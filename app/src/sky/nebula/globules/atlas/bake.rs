//! A bake's scratch textures and bind groups, prepared when a new bake is
//! requested (then dispatched by `pipeline`). A bake goes slice by slice (whole
//! rows of tiles), through scratch textures one slice tall.

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

use super::{AtlasBakeJob, REACH, SIZE, SLICE_ROWS, pipeline::AtlasPipeline};

/// Mirrors `Bake` in `atlas.wgsl`.
#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuBake {
    buoyancy: f32,
    step: i32,
    slide: f32,
    row_offset: i32,
}

/// The field (r32) and the two jump-flood ping-pong textures (rg32uint: two
/// packed texel positions).
struct Scratch {
    _textures: [Texture; 3],
    views: [TextureView; 3],
}

/// The bind groups of a pending bake: per slice, its shape and resolve; the
/// flood steps, the same for every slice.
pub(super) struct Bake {
    pub(super) shapes: Vec<BindGroup>,
    pub(super) floods: Vec<BindGroup>,
    pub(super) resolves: Vec<BindGroup>,
    pub(super) generation: u64,
}

#[derive(Resource, Default)]
pub(super) struct AtlasBakeState {
    scratch: Option<Scratch>,
    pub(super) pending: Option<Bake>,
    pub(super) baked_generation: u64,
}

fn scratch_texture(device: &RenderDevice, format: TextureFormat) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("globule_atlas_scratch"),
        size: Extent3d {
            width: SIZE,
            height: SLICE_ROWS,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn uniform(device: &RenderDevice, queue: &RenderQueue, bake: GpuBake) -> UniformBuffer<GpuBake> {
    let mut buffer = UniformBuffer::from(bake);
    buffer.write_buffer(device, queue);
    buffer
}

pub(super) fn prepare_atlas_bake(
    job: Option<Res<AtlasBakeJob>>,
    mut state: ResMut<AtlasBakeState>,
    pipeline: Res<AtlasPipeline>,
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
            .is_some_and(|p| p.generation == job.generation)
    {
        return;
    }
    let Some(atlas) = gpu_images.get(&job.image) else {
        return;
    };
    let scratch = state.scratch.get_or_insert_with(|| {
        let (r32, seeds) = (TextureFormat::R32Float, TextureFormat::Rg32Uint);
        let textures = [r32, seeds, seeds].map(|f| scratch_texture(&device, f));
        let views = [0, 1, 2].map(|i| textures[i].create_view(&TextureViewDescriptor::default()));
        Scratch {
            _textures: textures,
            views,
        }
    });
    let [field, a, b] = &scratch.views;
    let group = |stage: usize, entries: &[BindGroupEntry]| {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.layouts[stage]);
        device.create_bind_group("globule_atlas", &layout, entries)
    };
    let bake = |step, slice: u32| GpuBake {
        buoyancy: job.buoyancy,
        step,
        slide: job.slide,
        row_offset: (slice * SLICE_ROWS) as i32,
    };
    let [first, count] = job.slices;
    let slices = first..first + count;

    let shapes = slices
        .clone()
        .map(|slice| {
            let buffer = uniform(&device, &queue, bake(0, slice));
            group(
                0,
                &BindGroupEntries::with_indices(((0, field), (1, a), (2, &buffer))),
            )
        })
        .collect();
    // Steps REACH .. 1, then 1 again (fixes the rare misses), ping-ponging
    // between the two flood textures from `a`.
    let mut steps = Vec::new();
    let mut step = REACH as i32;
    while step >= 1 {
        steps.push(step);
        step /= 2;
    }
    steps.push(1);
    let floods: Vec<BindGroup> = steps
        .iter()
        .enumerate()
        .map(|(i, &step)| {
            let (from, to) = if i % 2 == 0 { (a, b) } else { (b, a) };
            let buffer = uniform(&device, &queue, bake(step, 0));
            group(
                1,
                &BindGroupEntries::with_indices(((3, from), (4, to), (5, &buffer))),
            )
        })
        .collect();
    let seeds = if steps.len() % 2 == 1 { b } else { a };
    let resolves = slices
        .map(|slice| {
            let buffer = uniform(&device, &queue, bake(0, slice));
            group(
                2,
                &BindGroupEntries::with_indices((
                    (6, field),
                    (7, seeds),
                    (8, &atlas.texture_view),
                    (9, &buffer),
                )),
            )
        })
        .collect();
    state.pending = Some(Bake {
        shapes,
        floods,
        resolves,
        generation: job.generation,
    });
}
