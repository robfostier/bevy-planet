//! The globule atlas (`assets/shaders/sky/nebula/globules/atlas.wgsl`): TILES
//! x TILES globule shapes, each with its exact signed distance field, baked on
//! the GPU at startup and as the system ages (their edges deform: a slice of
//! its rows per frame, in turn). The sky reads a globule's tile instead of
//! evaluating its shape per pixel, and draws rims from the distance.

mod bake;
mod pipeline;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use bake::{AtlasBakeState, prepare_atlas_bake};
use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::RenderGraph,
    },
    shader::ShaderDefVal,
};
use pipeline::{init_atlas_pipeline, run_atlas_bake};

use super::GlobuleStyle;
use crate::system::SystemParams;

const SHADER_ASSET_PATH: &str = "shaders/sky/nebula/globules/atlas.wgsl";
/// Texels on a side of one tile, and tiles on a side of the atlas (passed to
/// the shaders as shader defs, see `shader_defs`).
const TILE: u32 = 256;
const TILES: u32 = 8;
const SIZE: u32 = TILE * TILES;
/// The slices of the atlas' rows (whole rows of tiles) baked in turn as the
/// age changes: a quarter of the cost per frame while a slider is dragged.
const SLICES: u32 = 4;
/// Texel rows in a slice (its scratch textures are that tall).
const SLICE_ROWS: u32 = SIZE / SLICES;
/// How far from a shape's edge its distances are exact, in texels (the jump
/// flood starts there: a third of the steps a whole tile would need); farther,
/// they are clamped to it, still a bound the sky's rim tracing can step by.
const REACH: u32 = 32;

/// The atlas' sizes, for every shader that reads or writes it (`atlas.wgsl`,
/// `tile.wgsl`).
pub(crate) fn shader_defs() -> Vec<ShaderDefVal> {
    vec![
        ShaderDefVal::UInt("GLOBULE_TILE".into(), TILE),
        ShaderDefVal::UInt("GLOBULE_TILES".into(), TILES),
        ShaderDefVal::UInt("GLOBULE_REACH".into(), REACH),
    ]
}

/// The atlas image, reserved at plugin-build and baked by `pipeline`.
#[derive(Resource)]
pub(crate) struct GlobuleAtlas(pub(crate) Handle<Image>);

impl FromWorld for GlobuleAtlas {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource_mut::<Assets<Image>>().reserve_handle())
    }
}

/// The generation of the atlas' last bake, written by the render world: the
/// next slice waits for it (one asked for before is never lost).
#[derive(Resource, Clone, Default, ExtractResource)]
struct AtlasBakedGeneration(Arc<AtomicU64>);

/// One bake request, extracted to the render world.
#[derive(Resource, Clone, ExtractResource)]
struct AtlasBakeJob {
    buoyancy: f32,
    slide: f32,
    /// The slices to bake: the first, and how many.
    slices: [u32; 2],
    generation: u64,
    image: Handle<Image>,
}

fn init_atlas(atlas: Res<GlobuleAtlas>, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_uninit(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    let _ = images.insert(atlas.0.id(), image);
}

/// How far the edges' tearing noise slides over a whole life (noise units: 1
/// is a fully new tearing).
const SLIDE_SPAN: f32 = 1.5;

/// Where the atlas stands against the system's age.
#[derive(Default)]
struct Baked {
    age: Option<f32>,
    /// The slices still to bake at that age, and the next one.
    stale: u32,
    next: u32,
}

/// Bakes the atlas of the style's shapes, their edges slid with the system's
/// age: whole at startup; as the age changes, a slice per frame, in turn,
/// until all of them are at the age.
fn queue_bake(
    mut commands: Commands,
    atlas: Res<GlobuleAtlas>,
    system: Res<SystemParams>,
    done: Res<AtlasBakedGeneration>,
    mut baked: Local<Baked>,
    mut generation: Local<u64>,
) {
    if done.0.load(Ordering::Relaxed) < *generation {
        return;
    }
    let slices = match baked.age {
        None => [0, SLICES],
        Some(age) => {
            if age != system.age {
                baked.stale = SLICES;
            }
            if baked.stale == 0 {
                return;
            }
            baked.stale -= 1;
            let slice = baked.next;
            baked.next = (slice + 1) % SLICES;
            [slice, 1]
        }
    };
    baked.age = Some(system.age);
    *generation += 1;
    commands.insert_resource(AtlasBakeJob {
        buoyancy: GlobuleStyle::default().buoyancy,
        slide: (system.age - 0.5) * SLIDE_SPAN,
        slices,
        generation: *generation,
        image: atlas.0.clone(),
    });
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<GlobuleAtlas>()
        .init_resource::<AtlasBakedGeneration>()
        .add_plugins((
            ExtractResourcePlugin::<AtlasBakeJob>::default(),
            ExtractResourcePlugin::<AtlasBakedGeneration>::default(),
        ))
        .add_systems(Startup, init_atlas)
        .add_systems(Update, queue_bake);

    app.sub_app_mut(RenderApp)
        .init_resource::<AtlasBakeState>()
        .add_systems(RenderStartup, init_atlas_pipeline)
        .add_systems(
            Render,
            prepare_atlas_bake.in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(RenderGraph, run_atlas_bake.before(camera_driver));
}
