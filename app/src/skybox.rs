use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin};
use noise::{Fbm, NoiseFn, Perlin};

/// Maps a pixel position `(u, v)` (each in `[-1, 1]`) on cubemap face
/// `face_index` to the world-space direction it represents.
fn face_uv_to_direction(face_index: usize, u: f32, v: f32) -> Vec3 {
    match face_index {
        0 => Vec3::new(1.0, -v, -u),  // +X
        1 => Vec3::new(-1.0, -v, u),  // -X
        2 => Vec3::new(u, 1.0, v),    // +Y
        3 => Vec3::new(u, -1.0, -v),  // -Y
        4 => Vec3::new(u, -v, 1.0),   // +Z
        5 => Vec3::new(-u, -v, -1.0), // -Z
        _ => unreachable!("face_index must be in 0..6"),
    }
    .normalize()
}

/// Samples the nebula's noise field in the given direction and remaps the
/// result from the noise crate's native `[-1, 1]` range to a `[0, 1]`
/// density value.
///
/// `frequency` controls how many nebula "blobs" fit around the sky: since
/// `direction` is a unit vector, all samples lie on the unit sphere --
/// a low frequency stretches the noise field over that whole sphere
/// (few, large blobs), a high frequency packs more variation into it
/// (many, smaller blobs).
fn nebula_density(direction: Vec3, noise: &Fbm<Perlin>, frequency: f64) -> f32 {
    let point = (direction.as_dvec3() * frequency).to_array();
    let raw = noise.get(point);
    ((raw + 1.0) * 0.5) as f32
}

/// Renders one cubemap face into raw RGBA32-float pixel bytes by sampling
/// `nebula_density` once per pixel. This is a placeholder grayscale
/// rendering -- just enough to confirm the pipeline works end-to-end.
fn build_face_pixels(
    face_index: usize,
    resolution: u32,
    noise: &Fbm<Perlin>,
    frequency: f64,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(resolution as usize * resolution as usize * 16);
    for y in 0..resolution {
        for x in 0..resolution {
            let u = (x as f32 + 0.5) / resolution as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / resolution as f32 * 2.0 - 1.0;
            let direction = face_uv_to_direction(face_index, u, v);
            // Negate Z so the noise is sampled in the cubemap's left-handed frame,
            // matching the ray.z negation done in skybox.wgsl's fragment shader.
            let sample_direction = direction * Vec3::new(1.0, 1.0, -1.0);
            let density = nebula_density(sample_direction, noise, frequency);
            for channel in [density, density, density, 1.0] {
                bytes.extend_from_slice(&channel.to_le_bytes());
            }
        }
    }
    bytes
}

/// Builds the full 6-face cubemap `Image`, ready to be assigned to a
/// `Skybox`'s `image` field.
pub(crate) fn build_skybox_cubemap_image(
    resolution: u32,
    noise: &Fbm<Perlin>,
    frequency: f64,
) -> Image {
    let data = (0..6)
        .flat_map(|face_index| build_face_pixels(face_index, resolution, noise, frequency))
        .collect();

    Image {
        texture_view_descriptor: Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        }),
        ..Image::new(
            Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 6,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

/// Regenerates the skybox cubemap whenever `SkyboxParams` changes and writes the
/// fresh image into the pre-reserved `SkyboxCubemap` handle, so the orbit camera's
/// `Skybox` picks it up without being re-spawned. The `resource_changed` run
/// condition keeps it idle except on parameter edits (and once at startup, when
/// the resource is first inserted.)
pub(crate) fn rebuild_skybox_cubemap(
    params: Res<SkyboxParams>,
    cubemap: Res<SkyboxCubemap>,
    mut images: ResMut<Assets<Image>>,
) {
    let noise = Fbm::<Perlin>::new(params.seed);
    let image = build_skybox_cubemap_image(256, &noise, params.frequency);
    let _ = images.insert(cubemap.0.id(), image);
}

/// Live-tunable parameters for the procedural skybox, edited at runtime through
/// the egui inspector and consumed by `rebuild_skybox_cubemap`.
#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct SkyboxParams {
    /// Seed for the procedural noise generator.
    pub(crate) seed: u32,
    /// Frequency of the procedural noise generator.
    #[inspector(min = 0.1, max = 10.0, speed = 0.1)]
    pub(crate) frequency: f64,
}

impl Default for SkyboxParams {
    fn default() -> Self {
        Self {
            seed: 0,
            frequency: 2.0,
        }
    }
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

/// Wires up the procedural skybox: the tunable `SkyboxParams`, its egui
/// inspector panel, the shared `SkyboxCubemap` handle, and the system that
/// rebuilds the cubemap whenever the parameters change.
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyboxParams>()
            .register_type::<SkyboxParams>()
            .add_plugins(ResourceInspectorPlugin::<SkyboxParams>::default())
            .init_resource::<SkyboxCubemap>()
            .add_systems(
                Update,
                rebuild_skybox_cubemap.run_if(resource_changed::<SkyboxParams>),
            );
    }
}
