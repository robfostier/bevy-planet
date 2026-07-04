//! Nebula generation: a scalar density field sampled per direction. Domain
//! warping (after Inigo Quilez, <https://iquilezles.org/articles/warp/>) turns
//! plain FBM blobs into swirling filaments before the final density lookup.

use bevy::{math::DVec3, prelude::*};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use super::params::{NoiseParams, SkyboxParams};

/// Builds a configured `Fbm<Perlin>` from a `NoiseParams`.
fn build_fbm(params: &NoiseParams) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(params.seed)
        .set_octaves(params.octaves)
        .set_frequency(params.frequency)
        .set_lacunarity(params.lacunarity)
        .set_persistence(params.persistence)
}

/// A ready-to-sample nebula generator, built once per rebuild from `SkyboxParams`:
/// the heavy `Fbm` instances are configured up front and reused for every pixel.
pub(super) struct NebulaField {
    /// Detailed field, read at the warped position for the final density.
    density_noise: Fbm<Perlin>,
    /// Smooth field, used to build the domain-warp displacement.
    warp_noise: Fbm<Perlin>,
    /// Displacement strength of the domain warp.
    warp_strength: f64,
}

impl NebulaField {
    /// Builds the generator from the tunable parameters.
    pub(super) fn new(params: &SkyboxParams) -> Self {
        Self {
            density_noise: build_fbm(&params.density),
            warp_noise: build_fbm(&params.warp),
            warp_strength: params.warp_strength,
        }
    }
}

/// Domain-warps a sampling position: offsets it by a displacement built from the
/// smooth `warp_noise`, in two nested levels (after Inigo Quilez). The final noise
/// lookup happens in `nebula_density` at the returned position, so the full chain
/// is three noise levels deep.
///
/// `warp_strength` scales how far the position is pushed.
fn domain_warp(point: DVec3, field: &NebulaField) -> DVec3 {
    let q: DVec3 = DVec3::new(
        field
            .warp_noise
            .get([point.x + 0.0, point.y + 0.0, point.z + 0.0]),
        field
            .warp_noise
            .get([point.x + 5.2, point.y + 1.3, point.z + 7.1]),
        field
            .warp_noise
            .get([point.x + 8.5, point.y + 2.8, point.z + 3.4]),
    );

    let r = DVec3::new(
        field.warp_noise.get([
            point.x + 0.0 + field.warp_strength * q.x,
            point.y + 0.0 + field.warp_strength * q.y,
            point.z + 0.0 + field.warp_strength * q.z,
        ]),
        field.warp_noise.get([
            point.x + 1.3 + field.warp_strength * q.x,
            point.y + 7.1 + field.warp_strength * q.y,
            point.z + 2.8 + field.warp_strength * q.z,
        ]),
        field.warp_noise.get([
            point.x + 9.2 + field.warp_strength * q.x,
            point.y + 3.4 + field.warp_strength * q.y,
            point.z + 5.2 + field.warp_strength * q.z,
        ]),
    );
    point + field.warp_strength * r
}

/// Nebula density in a given direction, in `[0, 1]`. Domain-warps the direction,
/// samples `density_noise` there, and remaps the noise crate's native `[-1, 1]`
/// range to `[0, 1]`. Still a raw density -- contrast, palette and stars come later.
pub(super) fn nebula_density(direction: Vec3, field: &NebulaField) -> f32 {
    let point = domain_warp(direction.as_dvec3(), field);
    let raw = field.density_noise.get(point.to_array());
    ((raw + 1.0) * 0.5) as f32
}
