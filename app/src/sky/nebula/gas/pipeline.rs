//! The render-world side of the bake: the compute pipeline, the bind group of
//! each pending bake job, and its dispatch (when the seed or the age changes).

use std::{borrow::Cow, sync::atomic::Ordering};

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

use super::{
    Dispatches, NebulaBakeJob, NebulaBakedGeneration, SHADER_ASSET_PATH, WORKGROUP_SIZE,
    uniforms::GpuNebulaParams,
};

#[derive(Resource)]
pub(super) struct NebulaPipeline {
    layout: BindGroupLayoutDescriptor,
    /// The fields' entry point (`bake`), the carry's (`bake_carry`) and the
    /// density's (`bake_density`).
    pipelines: [CachedComputePipelineId; 3],
}

/// A bake ready to dispatch.
struct Pending {
    bind_group: BindGroup,
    dispatches: Dispatches,
    generation: u64,
}

#[derive(Resource, Default)]
pub(crate) struct NebulaBakeState {
    pending: Option<Pending>,
    baked_generation: u64,
    /// The latest job's generation.
    latest_generation: u64,
}

impl NebulaBakeState {
    /// Whether the cubemaps hold the latest job's bake (what reads them, like
    /// the globules' mask, waits for it).
    pub(crate) fn is_current(&self) -> bool {
        self.latest_generation > 0 && self.baked_generation == self.latest_generation
    }
}

pub(super) fn init_nebula_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let storage =
        || texture_storage_2d_array(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly);
    // Fields, uniforms, permutation tables, carry, density.
    let layout = BindGroupLayoutDescriptor::new(
        "nebula_bake",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage(),
                uniform_buffer::<GpuNebulaParams>(false),
                storage_buffer_read_only::<Vec<u32>>(false),
                storage(),
                storage(),
            ),
        ),
    );
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let pipelines = ["bake", "bake_carry", "bake_density"].map(|entry| {
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(format!("nebula_{entry}").into()),
            layout: vec![layout.clone()],
            shader: shader.clone(),
            entry_point: Some(Cow::from(entry)),
            ..default()
        })
    });
    commands.insert_resource(NebulaPipeline { layout, pipelines });
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
    state.latest_generation = job.generation;
    if job.generation == state.baked_generation
        || state
            .pending
            .as_ref()
            .is_some_and(|p| p.generation == job.generation)
    {
        return;
    }
    let Some([fields, carry, density]) = job
        .images
        .each_ref()
        .map(|image| gpu_images.get(image))
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|images| <[&GpuImage; 3]>::try_from(images).ok())
    else {
        return;
    };
    let mut uniform = UniformBuffer::from(job.params);
    uniform.write_buffer(&render_device, &queue);
    let mut tables = StorageBuffer::from(job.tables.clone());
    tables.write_buffer(&render_device, &queue);
    let view = |image: &GpuImage| {
        image.texture.create_view(&TextureViewDescriptor {
            label: Some("nebula_bake_storage_view"),
            dimension: Some(TextureViewDimension::D2Array),
            ..default()
        })
    };
    let [fields, carry, density] = [fields, carry, density].map(view);
    let bind_group = render_device.create_bind_group(
        "nebula_bake",
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((&fields, &uniform, &tables, &carry, &density)),
    );
    state.pending = Some(Pending {
        bind_group,
        dispatches: job.dispatches,
        generation: job.generation,
    });
}

fn dispatch(
    pass: &mut ComputePass,
    pipeline: &ComputePipeline,
    bind_group: &BindGroup,
    res: u32,
    faces: u32,
) {
    let groups = res.div_ceil(WORKGROUP_SIZE);
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(groups, groups, faces);
}

pub(super) fn run_nebula_bake(
    mut render_context: RenderContext,
    mut state: ResMut<NebulaBakeState>,
    pipeline: Res<NebulaPipeline>,
    pipeline_cache: Res<PipelineCache>,
    done: Option<Res<NebulaBakedGeneration>>,
) {
    let Some([fields, carry, density]) = pipeline
        .pipelines
        .map(|id| pipeline_cache.get_compute_pipeline(id))
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|p| <[&ComputePipeline; 3]>::try_from(p).ok())
    else {
        return;
    };
    let Some(bake) = state.pending.take() else {
        return;
    };

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("nebula_bake"),
            ..default()
        });
    let d = bake.dispatches;
    let [fields_res, carry_res, density_res] = d.resolutions;
    let group = &bake.bind_group;
    dispatch(&mut pass, fields, group, fields_res, d.faces[1]);
    if d.carry {
        dispatch(&mut pass, carry, group, carry_res, 6);
    }
    if d.density {
        dispatch(&mut pass, density, group, density_res, 6);
    }
    state.baked_generation = bake.generation;
    if let Some(done) = done {
        done.0.store(bake.generation, Ordering::Relaxed);
    }
}
