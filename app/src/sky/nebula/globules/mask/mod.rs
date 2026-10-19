//! The globules' mask (`assets/shaders/sky/nebula/globules/mask.wgsl`): a
//! low-resolution cubemap of where a globule may show at any age (with the
//! most gas a cloud keeps), baked on the GPU for each system seed, after the
//! gas (it reads the style's density). The sky skips its search for globules
//! wherever it is 0.

mod pipeline;

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
use pipeline::{GpuMaskBake, MaskBakeState, init_mask_pipeline, prepare_mask_bake, run_mask_bake};

use super::{GlobuleStyle, GpuGlobules, SEED_PATH};
use crate::sky::nebula::gas::{NebulaBakeSet, NebulaCubemaps};
use crate::sky::nebula::properties::MOST_GAS_LEFT;
use crate::system::SystemParams;

const SHADER_ASSET_PATH: &str = "shaders/sky/nebula/globules/mask.wgsl";
/// Texels on a side of a face: a fraction of the smallest globules' reach.
const RESOLUTION: u32 = 512;

/// The mask cubemap, reserved at plugin-build and baked by `pipeline`.
#[derive(Resource)]
pub(crate) struct GlobuleMask(pub(crate) Handle<Image>);

impl FromWorld for GlobuleMask {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource_mut::<Assets<Image>>().reserve_handle())
    }
}

/// One bake request, extracted to the render world.
#[derive(Resource, Clone, ExtractResource)]
struct MaskBakeJob {
    bake: GpuMaskBake,
    generation: u64,
    image: Handle<Image>,
    /// The style's density (`NebulaCubemaps`).
    density: Handle<Image>,
}

fn init_mask(mask: Res<GlobuleMask>, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_uninit(
        Extent3d {
            width: RESOLUTION,
            height: RESOLUTION,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    let _ = images.insert(mask.0.id(), image);
}

/// Bakes the mask of the globules' scatter for a new seed: with the most gas
/// a cloud keeps, so it holds whatever the age (no bake as it changes).
fn queue_bake(
    mut commands: Commands,
    mask: Res<GlobuleMask>,
    cubemaps: Res<NebulaCubemaps>,
    system: Res<SystemParams>,
    mut baked_seed: Local<Option<u32>>,
    mut generation: Local<u64>,
) {
    if *baked_seed == Some(system.seed) {
        return;
    }
    *baked_seed = Some(system.seed);
    let globules = GpuGlobules::new(
        &GlobuleStyle::default(),
        system.derive(SEED_PATH),
        MOST_GAS_LEFT,
    );
    *generation += 1;
    commands.insert_resource(MaskBakeJob {
        bake: GpuMaskBake::new(&globules, RESOLUTION),
        generation: *generation,
        image: mask.0.clone(),
        density: cubemaps.density.clone(),
    });
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<GlobuleMask>()
        .add_plugins(ExtractResourcePlugin::<MaskBakeJob>::default())
        .add_systems(Startup, init_mask)
        .add_systems(Update, queue_bake.run_if(resource_changed::<SystemParams>));

    app.sub_app_mut(RenderApp)
        .init_resource::<MaskBakeState>()
        .add_systems(RenderStartup, init_mask_pipeline)
        .add_systems(
            Render,
            prepare_mask_bake.in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            RenderGraph,
            run_mask_bake.after(NebulaBakeSet).before(camera_driver),
        );
}
