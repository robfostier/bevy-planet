//! The nebula's gas: its style (`params`) and its cubemap of noise fields
//! (`NebulaCubemaps`), baked on the GPU with a compute shader
//! (`assets/shaders/sky/nebula/gas/bake.wgsl`); the sky composes the gas from
//! them every frame (`look`).
//!
//! - The noise is the `noise` crate's Perlin / Fbm / RidgedMulti, reproduced
//!   exactly: its permutation tables are generated on the CPU (one per field
//!   and octave, exactly as `Fbm`/`RidgedMulti` seed them) and uploaded.
//! - The bake only runs for a new seed (generation counter).

mod look;
mod noise;
mod params;
mod pipeline;
mod queue;
mod uniforms;

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

pub(crate) use look::GpuNebulaLook;
pub(crate) use params::NebulaStyle;
pub(crate) use pipeline::NebulaBakeState;

const SHADER_ASSET_PATH: &str = "shaders/sky/nebula/gas/bake.wgsl";
const WORKGROUP_SIZE: u32 = 8;
/// Texels on a side of the cubemaps' faces: the fields' (the gas's detail)
/// and the density's (the globules' light and placement, at their scale).
const RESOLUTION: u32 = 1024;
const DENSITY_RESOLUTION: u32 = 256;

/// The render-graph system that bakes the cubemaps (what reads them in the
/// render graph runs after it).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NebulaBakeSet;

/// The nebula's cubemaps, reserved at plugin-build (so they exist before any
/// system reads them) and never replaced: allocated here, baked at startup and
/// for every new seed (`bake.wgsl`).
/// - `fields`: the noise fields the sky composes the gas of, every frame;
/// - `density`: the style's density (the globules' light and placement).
#[derive(Resource)]
pub(crate) struct NebulaCubemaps {
    pub(crate) fields: Handle<Image>,
    pub(crate) density: Handle<Image>,
}

impl FromWorld for NebulaCubemaps {
    fn from_world(world: &mut World) -> Self {
        let images = world.resource_mut::<Assets<Image>>();
        Self {
            fields: images.reserve_handle(),
            density: images.reserve_handle(),
        }
    }
}

/// One bake request, extracted to the render world.
#[derive(Resource, Clone, ExtractResource)]
struct NebulaBakeJob {
    params: GpuNebulaParams,
    tables: Vec<u32>,
    generation: u64,
    /// The cubemaps to bake: fields, density.
    images: [Handle<Image>; 2],
}

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
    let _ = images.insert(cubemaps.density.id(), cubemap_image(DENSITY_RESOLUTION));
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<NebulaCubemaps>()
        .add_plugins(ExtractResourcePlugin::<NebulaBakeJob>::default())
        .add_systems(Startup, init_cubemaps)
        .add_systems(Update, queue_bake);

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
