//! The render-world side of the mask bake: its compute pipeline, the bind
//! group of each pending bake, and its dispatch.

use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_cube, texture_storage_2d_array, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

use super::{GpuGlobules, MaskBakeJob, SHADER_ASSET_PATH};
use crate::sky::nebula::gas::NebulaBakeState;

const WORKGROUP_SIZE: u32 = 8;

/// Mirrors `MaskBake` in `mask.wgsl`: the globules' scatter, the density
/// below which they never show, and the mask's resolution.
#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuMaskBake {
    count_large: f32,
    count_small: f32,
    size: f32,
    variation: f32,
    draw_start: f32,
    gas_faint: f32,
    resolution: u32,
    pad: u32,
}

impl GpuMaskBake {
    pub(super) fn new(g: &GpuGlobules, resolution: u32) -> Self {
        Self {
            count_large: g.count_large,
            count_small: g.count_small,
            size: g.size,
            variation: g.size_variation,
            draw_start: g.draw_start,
            gas_faint: g.gas_faint,
            resolution,
            pad: 0,
        }
    }
}

#[derive(Resource)]
pub(super) struct MaskPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
    /// Filters the style's density.
    sampler: Sampler,
}

#[derive(Resource, Default)]
pub(super) struct MaskBakeState {
    /// Bind group ready to dispatch, with its resolution and generation.
    pending: Option<(BindGroup, u32, u64)>,
    baked_generation: u64,
}

pub(super) fn init_mask_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    device: Res<RenderDevice>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "globule_mask",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d_array(
                    TextureFormat::Rgba8Unorm,
                    StorageTextureAccess::WriteOnly,
                ),
                uniform_buffer::<GpuMaskBake>(false),
                texture_cube(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("globule_mask".into()),
        layout: vec![layout.clone()],
        shader: asset_server.load(SHADER_ASSET_PATH),
        entry_point: Some(Cow::from("bake_mask")),
        ..default()
    });
    let sampler = device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });
    commands.insert_resource(MaskPipeline {
        layout,
        pipeline,
        sampler,
    });
}

pub(super) fn prepare_mask_bake(
    job: Option<Res<MaskBakeJob>>,
    mut state: ResMut<MaskBakeState>,
    pipeline: Res<MaskPipeline>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let Some(job) = job else { return };
    if job.generation == state.baked_generation
        || state
            .pending
            .as_ref()
            .is_some_and(|p| p.2 == job.generation)
    {
        return;
    }
    let (Some(mask), Some(density)) = (gpu_images.get(&job.image), gpu_images.get(&job.density))
    else {
        return;
    };
    let mut uniform = UniformBuffer::from(job.bake);
    uniform.write_buffer(&device, &queue);
    let view = mask.texture.create_view(&TextureViewDescriptor {
        label: Some("globule_mask_storage_view"),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    let bind_group = device.create_bind_group(
        "globule_mask",
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((&view, &uniform, &density.texture_view, &pipeline.sampler)),
    );
    state.pending = Some((bind_group, job.bake.resolution, job.generation));
}

pub(super) fn run_mask_bake(
    mut render_context: RenderContext,
    mut state: ResMut<MaskBakeState>,
    pipeline: Res<MaskPipeline>,
    pipeline_cache: Res<PipelineCache>,
    nebula: Res<NebulaBakeState>,
) {
    // It reads the style's density: not before the gas's bake is done.
    if !nebula.is_current() {
        return;
    }
    let Some(compute) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
        return;
    };
    let Some((bind_group, res, generation)) = state.pending.take() else {
        return;
    };
    let groups = res.div_ceil(WORKGROUP_SIZE);
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("globule_mask"),
            ..default()
        });
    pass.set_pipeline(compute);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(groups, groups, 6);
    state.baked_generation = generation;
}
