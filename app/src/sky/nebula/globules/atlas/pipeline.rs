//! The render-world side of the atlas bake: its three compute pipelines
//! (shape, jump-flood step, resolve), and their dispatches
//! (once per bake request, prepared by `bake`).

use std::{borrow::Cow, sync::atomic::Ordering};

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{texture_2d, texture_storage_2d, uniform_buffer},
            *,
        },
        renderer::RenderContext,
    },
};

use super::{
    AtlasBakedGeneration, SHADER_ASSET_PATH, SIZE, SLICE_ROWS,
    bake::{AtlasBakeState, GpuBake},
    shader_defs,
};

const WORKGROUP_SIZE: u32 = 8;
/// The stages, in dispatch order (the entry points of `atlas.wgsl`).
const STAGES: [&str; 3] = ["shape", "flood", "resolve"];

#[derive(Resource)]
pub(super) struct AtlasPipeline {
    pub(super) layouts: [BindGroupLayoutDescriptor; 3],
    pipelines: [CachedComputePipelineId; 3],
}

pub(super) fn init_atlas_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let read = || texture_2d(TextureSampleType::Float { filterable: false });
    let read_seeds = || texture_2d(TextureSampleType::Uint);
    let write = |format| texture_storage_2d(format, StorageTextureAccess::WriteOnly);
    let layout = |stage: usize, entries: &[BindGroupLayoutEntry]| {
        BindGroupLayoutDescriptor::new(format!("globule_atlas_{}", STAGES[stage]), entries)
    };
    let stages = ShaderStages::COMPUTE;
    let layouts = [
        layout(
            0,
            &BindGroupLayoutEntries::with_indices(
                stages,
                (
                    (0, write(TextureFormat::R32Float)),
                    (1, write(TextureFormat::Rg32Uint)),
                    (2, uniform_buffer::<GpuBake>(false)),
                ),
            ),
        ),
        layout(
            1,
            &BindGroupLayoutEntries::with_indices(
                stages,
                (
                    (3, read_seeds()),
                    (4, write(TextureFormat::Rg32Uint)),
                    (5, uniform_buffer::<GpuBake>(false)),
                ),
            ),
        ),
        layout(
            2,
            &BindGroupLayoutEntries::with_indices(
                stages,
                (
                    (6, read()),
                    (7, read_seeds()),
                    (8, write(TextureFormat::Rgba16Float)),
                    (9, uniform_buffer::<GpuBake>(false)),
                ),
            ),
        ),
    ];
    let shader = asset_server.load(SHADER_ASSET_PATH);
    let pipelines = [0, 1, 2].map(|i| {
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(format!("globule_atlas_{}", STAGES[i]).into()),
            layout: vec![layouts[i].clone()],
            shader: shader.clone(),
            shader_defs: shader_defs(),
            entry_point: Some(Cow::from(STAGES[i])),
            ..default()
        })
    });
    commands.insert_resource(AtlasPipeline { layouts, pipelines });
}

pub(super) fn run_atlas_bake(
    mut render_context: RenderContext,
    mut state: ResMut<AtlasBakeState>,
    pipeline: Res<AtlasPipeline>,
    pipeline_cache: Res<PipelineCache>,
    done: Option<Res<AtlasBakedGeneration>>,
) {
    let Some([shape, flood, resolve]) = pipeline
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
    let (columns, rows) = (
        SIZE.div_ceil(WORKGROUP_SIZE),
        SLICE_ROWS.div_ceil(WORKGROUP_SIZE),
    );
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("globule_atlas"),
            ..default()
        });
    let mut dispatch = |pipeline: &ComputePipeline, group: &BindGroup| {
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(columns, rows, 1);
    };
    // Slice by slice, through the same scratch textures.
    for (shape_group, resolve_group) in bake.shapes.iter().zip(&bake.resolves) {
        dispatch(shape, shape_group);
        for group in &bake.floods {
            dispatch(flood, group);
        }
        dispatch(resolve, resolve_group);
    }
    state.baked_generation = bake.generation;
    if let Some(done) = done {
        done.0.store(bake.generation, Ordering::Relaxed);
    }
}
