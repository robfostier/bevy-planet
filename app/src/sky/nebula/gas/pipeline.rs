//! The render-world side of the bake: the compute pipeline, the bind group of
//! each pending bake job, and its dispatch (when the seed changes).

use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{storage_buffer_read_only, texture_storage_2d_array, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

use super::{NebulaBakeJob, SHADER_ASSET_PATH, WORKGROUP_SIZE, uniforms::GpuNebulaParams};

#[derive(Resource)]
pub(super) struct NebulaPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

/// A bake ready to dispatch.
struct Pending {
    bind_group: BindGroup,
    resolution: u32,
    generation: u64,
}

#[derive(Resource, Default)]
pub(super) struct NebulaBakeState {
    pending: Option<Pending>,
    baked_generation: u64,
}

pub(super) fn init_nebula_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    // Fields, uniforms, permutation tables.
    let layout = BindGroupLayoutDescriptor::new(
        "nebula_bake",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d_array(
                    TextureFormat::Rgba16Float,
                    StorageTextureAccess::WriteOnly,
                ),
                uniform_buffer::<GpuNebulaParams>(false),
                storage_buffer_read_only::<Vec<u32>>(false),
            ),
        ),
    );
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("nebula_bake".into()),
        layout: vec![layout.clone()],
        shader: asset_server.load(SHADER_ASSET_PATH),
        entry_point: Some(Cow::from("bake")),
        ..default()
    });
    commands.insert_resource(NebulaPipeline { layout, pipeline });
}

pub(super) fn prepare_nebula_bind_group(
    job: Option<Res<NebulaBakeJob>>,
    mut state: ResMut<NebulaBakeState>,
    pipeline: Res<NebulaPipeline>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let Some(job) = job else { return };
    if job.generation == state.baked_generation
        || state
            .pending
            .as_ref()
            .is_some_and(|p| p.generation == job.generation)
    {
        return;
    }
    let Some(image) = gpu_images.get(&job.image) else {
        return;
    };
    let mut uniform = UniformBuffer::from(job.params);
    uniform.write_buffer(&render_device, &queue);
    let mut tables = StorageBuffer::from(job.tables.clone());
    tables.write_buffer(&render_device, &queue);
    let fields = image.texture.create_view(&TextureViewDescriptor {
        label: Some("nebula_bake_storage_view"),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    let bind_group = render_device.create_bind_group(
        "nebula_bake",
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((&fields, &uniform, &tables)),
    );
    state.pending = Some(Pending {
        bind_group,
        resolution: image.texture_descriptor.size.width,
        generation: job.generation,
    });
}

pub(super) fn run_nebula_bake(
    mut render_context: RenderContext,
    mut state: ResMut<NebulaBakeState>,
    pipeline: Res<NebulaPipeline>,
    pipeline_cache: Res<PipelineCache>,
) {
    let Some(compute) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
        return;
    };
    let Some(bake) = state.pending.take() else {
        return;
    };
    let groups = bake.resolution.div_ceil(WORKGROUP_SIZE);
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("nebula_bake"),
            ..default()
        });
    pass.set_pipeline(compute);
    pass.set_bind_group(0, &bake.bind_group, &[]);
    pass.dispatch_workgroups(groups, groups, 6);
    state.baked_generation = bake.generation;
}
