//! Procedural nebula skybox: a CPU-baked cubemap sampling a domain-warped noise
//! field, exposed as `SkyboxPlugin` and rebuilt live from `SkyboxParams`. The
//! generation lives in `field`, the cubemap assembly in `cubemap`, the tunable
//! parameters in `params`.

mod cubemap;
mod field;
mod params;

pub(crate) use params::{NoiseParams, SkyboxParams};

use bevy::{image::Image, prelude::*};
use cubemap::build_skybox_cubemap_image;
use field::NebulaField;

/// Resolution, per face, of the baked cubemap. Kept low while iterating; raise it
/// for a sharper sky at the cost of a longer bake.
const SKYBOX_RES: u32 = 256;

/// Regenerates the skybox cubemap whenever `SkyboxParams` changes and writes the
/// fresh image into the pre-reserved `SkyboxCubemap` handle, so the orbit camera's
/// `Skybox` picks it up without being re-spawned. The `resource_changed` run
/// condition keeps it idle except on parameter edits (and once at startup, when
/// the resource is first inserted.)
fn rebuild_skybox_cubemap(
    params: Res<SkyboxParams>,
    cubemap: Res<SkyboxCubemap>,
    mut images: ResMut<Assets<Image>>,
) {
    let field = NebulaField::new(&params);
    let image = build_skybox_cubemap_image(SKYBOX_RES, &field);
    let _ = images.insert(cubemap.0.id(), image);
}

/// Holds the shared `Handle<Image>` for the skybox cubemap.
/// The image is generated once at startup and whenever `SkyboxParams` changes,
/// and the handle is never replaced.
#[derive(Resource)]
pub(crate) struct SkyboxCubemap(pub(crate) Handle<Image>);

/// Reserves and empty `Handle<Image>` at plugin-build, so it exists before any
/// camera reads it. The pixels are baked later by `rebuild_skybox_cubemap`.
impl FromWorld for SkyboxCubemap {
    fn from_world(world: &mut World) -> Self {
        let images = world.resource_mut::<Assets<Image>>();
        let handle = images.reserve_handle();
        Self(handle)
    }
}

/// Wires up the procedural skybox: the tunable `SkyboxParams`, the shared
/// `SkyboxCubemap` handle, and the system that rebuilds the cubemap whenever the
/// parameters change.
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyboxParams>()
            .init_resource::<SkyboxCubemap>()
            .add_systems(
                Update,
                rebuild_skybox_cubemap.run_if(resource_changed::<SkyboxParams>),
            );
    }
}
