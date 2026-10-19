//! The nebula's gas: its style (`params`), its cubemaps (`NebulaCubemaps`),
//! baked on the GPU with a compute shader
//! (`assets/shaders/sky/nebula/gas/bake.wesl`).
//!
//! - The noise is the `noise` crate's Perlin / Fbm / RidgedMulti, reproduced
//!   exactly: its permutation tables are generated on the CPU (one per field
//!   and octave, exactly as `Fbm`/`RidgedMulti` seed them) and uploaded.
//! - Its look is the base style (`NebulaStyle`), evolved by the nebula's
//!   properties (`NebulaProperties`); the bake only runs when they change
//!   (generation counter).

mod look;
mod noise;
mod params;
mod pipeline;
mod queue;
mod uniforms;

use std::sync::{Arc, atomic::AtomicU64};

use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::RenderGraph,
    },
};
use pipeline::{init_nebula_pipeline, prepare_nebula_bind_group, run_nebula_bake};
use queue::queue_bake;
use uniforms::GpuNebulaParams;

pub(crate) use look::{Evolution, GpuNebulaLook};
pub(crate) use params::NebulaStyle;
pub(crate) use pipeline::NebulaBakeState;

const SHADER_ASSET_PATH: &str = "shaders/sky/nebula/gas/bake.wesl";
const WORKGROUP_SIZE: u32 = 8;
/// Texels on a side of the cubemaps' faces: the fields' (the gas's detail),
/// the carry's (a smooth displacement) and the density's (the globules' light
/// and placement, at their scale).
const RESOLUTION: u32 = 1024;
const CARRY_RESOLUTION: u32 = 128;
const DENSITY_RESOLUTION: u32 = 256;
/// The fields' while the age keeps changing (a slider being dragged): a quick
/// preview, until the age settles.
const PREVIEW_RESOLUTION: u32 = 256;

/// The system that queues the cubemaps' bakes (what reads `NebulaCubemaps`
/// runs after it).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NebulaQueueSet;

/// The render-graph system that bakes the cubemaps (what reads them in the
/// render graph runs after it).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NebulaBakeSet;

/// The nebula's cubemaps, reserved at plugin-build (so they exist before any
/// system reads them) and never replaced: allocated here, baked at startup and
/// whenever the system's seed or age changes (`bake.wesl`).
/// - `fields`: the noise fields the sky composes the gas of, every frame;
/// - `preview`: the same at a low resolution, while the age keeps changing;
/// - `shown`: which of the two the sky reads;
/// - `carry`: where the style's gas now shows (the globules ride it);
/// - `density`: the style's density (the globules' light and placement; baked
///   only for a new seed, it never changes with the age).
#[derive(Resource)]
pub(crate) struct NebulaCubemaps {
    pub(crate) fields: Handle<Image>,
    pub(crate) preview: Handle<Image>,
    pub(crate) shown: Handle<Image>,
    pub(crate) carry: Handle<Image>,
    pub(crate) density: Handle<Image>,
}

impl FromWorld for NebulaCubemaps {
    fn from_world(world: &mut World) -> Self {
        let images = world.resource_mut::<Assets<Image>>();
        let fields = images.reserve_handle();
        Self {
            shown: fields.clone(),
            fields,
            preview: images.reserve_handle(),
            carry: images.reserve_handle(),
            density: images.reserve_handle(),
        }
    }
}

/// One bake request, extracted to the render world.
#[derive(Resource, Clone, ExtractResource)]
#[extract_app(RenderApp)]
struct NebulaBakeJob {
    params: GpuNebulaParams,
    tables: Vec<u32>,
    generation: u64,
    /// The cubemaps to bake: fields (or their preview), carry, density.
    images: [Handle<Image>; 3],
    dispatches: Dispatches,
}

/// What a bake writes: the fields' faces (the first, and how many), the carry
/// and the density, each at its resolution (fields, carry, density).
#[derive(Clone, Copy)]
struct Dispatches {
    resolutions: [u32; 3],
    faces: [u32; 2],
    carry: bool,
    density: bool,
}

/// The generation of the last bake, written by the render world: the next
/// one waits for it (a face asked for before is never lost).
#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(RenderApp)]
struct NebulaBakedGeneration(Arc<AtomicU64>);

/// An (uninitialised) storage cubemap, `resolution` texels a side.
fn cubemap_image(resolution: u32) -> Image {
    let mut image = Image::new_uninit(
        Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image
}

/// Allocates the nebula's cubemaps.
fn init_cubemaps(cubemaps: Res<NebulaCubemaps>, mut images: ResMut<Assets<Image>>) {
    let _ = images.insert(cubemaps.fields.id(), cubemap_image(RESOLUTION));
    let _ = images.insert(cubemaps.preview.id(), cubemap_image(PREVIEW_RESOLUTION));
    let _ = images.insert(cubemaps.carry.id(), cubemap_image(CARRY_RESOLUTION));
    let _ = images.insert(cubemaps.density.id(), cubemap_image(DENSITY_RESOLUTION));
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<NebulaCubemaps>()
        .init_resource::<NebulaBakedGeneration>()
        .add_plugins((
            ExtractResourcePlugin::<NebulaBakeJob>::default(),
            ExtractResourcePlugin::<NebulaBakedGeneration>::default(),
        ))
        .add_systems(Startup, init_cubemaps)
        .add_systems(Update, queue_bake.in_set(NebulaQueueSet));

    app.sub_app_mut(RenderApp)
        .init_resource::<NebulaBakeState>()
        .add_systems(RenderStartup, init_nebula_pipeline)
        .add_systems(
            Render,
            prepare_nebula_bind_group.in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            RenderGraph,
            run_nebula_bake.in_set(NebulaBakeSet).before(camera_driver),
        );
}
