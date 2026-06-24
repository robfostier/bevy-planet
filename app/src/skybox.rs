use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};
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
            // The cubemap's +Z face is flipped in the Y axis, so we flip it back here
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
pub(crate) fn build_nebula_cubemap_image(
    resolution: u32,
    noise: &Fbm<Perlin>,
    frequency: f64,
) -> Image {
    let data = (0..6)
        .into_iter()
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
