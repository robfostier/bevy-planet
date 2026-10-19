//! Nebula generation: a scalar density field sampled per direction. The density
//! is a ridged-noise structure multiplied by a smooth density mask, then shaped by
//! a global gamma and levels pass. Before sampling, the direction is optionally
//! domain-warped (after Inigo Quilez, <https://iquilezles.org/articles/warp/>) to
//! turn plain blobs into swirling filaments. Alongside the density, each sample
//! carries a hue (a separate warped noise) that feeds the palette in `color`.

use bevy::{math::DVec3, prelude::*};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti};

use crate::utils::{contrast, gamma, levels, smoothstep};

use super::params::{NoiseParams, SkyboxParams};

/// Builds a configured `Fbm<Perlin>` from a `NoiseParams`, its seed offset
/// by `base_seed`.
fn build_fbm(params: &NoiseParams, base_seed: u32) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(params.seed.wrapping_add(base_seed))
        .set_octaves(params.octaves)
        .set_frequency(params.frequency)
        .set_lacunarity(params.lacunarity)
        .set_persistence(params.persistence)
}

/// Builds a configured `RidgedMulti<Perlin>` from a `NoiseParams`: sharp-ridged
/// fractal noise, used to carve the nebula's filament structure.
fn build_ridged(params: &NoiseParams, base_seed: u32) -> RidgedMulti<Perlin> {
    RidgedMulti::<Perlin>::new(params.seed.wrapping_add(base_seed))
        .set_octaves(params.octaves)
        .set_frequency(params.frequency)
        .set_lacunarity(params.lacunarity)
        .set_persistence(params.persistence)
}

/// A ready-to-sample nebula generator, built once per rebuild from `SkyboxParams`:
/// the noise generators (ridge, density mask, warp) are configured up front and
/// reused for every pixel, alongside the flattened filter parameters they feed.
pub(super) struct NebulaField {
    /// Ridged fractal noise forming the filament structure.
    ridged_noise: RidgedMulti<Perlin>,
    /// Whether the ridge is multiplied into the density.
    ridge_enabled: bool,
    /// Gamma exponent shaping the ridge's falloff.
    ridge_gamma: f32,
    /// Whether the ridge gamma is applied.
    ridge_gamma_enabled: bool,
    /// Noise for the global density mask.
    mask_noise: Fbm<Perlin>,
    /// Whether the density mask is applied.
    mask_enabled: bool,
    /// Contrast strength for the density mask.
    mask_contrast: f32,
    /// Whether the mask contrast is applied.
    mask_contrast_enabled: bool,
    /// Levels `(floor, ceil)` remap for the density mask.
    mask_levels: (f32, f32),
    /// Whether the mask levels remap is applied.
    mask_levels_enabled: bool,
    /// Noise driving the domain-warp displacement.
    warp_noise: Fbm<Perlin>,
    /// How far the warp pushes the sampling position.
    warp_strength: f64,
    /// Whether the direction is domain-warped.
    warp_enabled: bool,
    /// Noise for the warp-amplitude mask.
    warp_mask_noise: Fbm<Perlin>,
    /// Whether the warp mask modulates the displacement amplitude.
    warp_mask_enabled: bool,
    /// Contrast strength for the warp mask.
    warp_mask_contrast: f32,
    /// Whether the warp mask contrast is applied.
    warp_mask_contrast_enabled: bool,
    /// Levels `(floor, ceil)` remap for the warp mask.
    warp_mask_levels: (f32, f32),
    /// Whether the warp mask levels remap is applied.
    warp_mask_levels_enabled: bool,
    /// Global gamma exponent on the final density.
    gamma: f32,
    /// Whether the global gamma is applied.
    gamma_enabled: bool,
    /// Global levels `(floor, ceil)` remap on the final density.
    levels: (f32, f32),
    /// Whether the global levels remap is applied.
    levels_enabled: bool,
    /// Noise driving the hue fed to the palette lookup.
    hue_noise: Fbm<Perlin>,
    /// Whether the hue varies across space; when off, a constant hue is used.
    hue_enabled: bool,
    /// Contrast strength for the hue.
    hue_contrast: f32,
    /// Whether the hue contrast is applied.
    hue_contrast_enabled: bool,
    /// Levels `(floor, ceil)` remap stretching the hue's narrow native range.
    hue_levels: (f32, f32),
    /// Whether the hue levels remap is applied.
    hue_levels_enabled: bool,
    /// Noise driving the spatial hue drift of the palette.
    drift_noise: Fbm<Perlin>,
    /// Whether the palette hue drift is applied.
    drift_enabled: bool,
    /// Contrast strength for the drift mask.
    drift_contrast: f32,
    /// Whether the drift contrast is applied.
    drift_contrast_enabled: bool,
    /// Levels `(floor, ceil)` remap for the drift mask.
    drift_levels: (f32, f32),
    /// Whether the drift levels remap is applied.
    drift_levels_enabled: bool,
}

pub(super) struct NebulaSample {
    pub(super) density: f32,
    pub(super) hue: f32,
    /// Spatial value in `[0, 1]` driving the palette hue drift.
    pub(super) drift: f32,
}

impl NebulaField {
    /// Builds the generator from the tunable parameters; `base_seed`: the
    /// nebula's seed (derived from the system seed), every field's offset.
    pub(super) fn new(params: &SkyboxParams, base_seed: u32) -> Self {
        Self {
            ridged_noise: build_ridged(&params.ridge.noise, base_seed),
            ridge_enabled: params.ridge.enabled,
            ridge_gamma: params.ridge.gamma.value,
            ridge_gamma_enabled: params.ridge.gamma.enabled,
            mask_noise: build_fbm(&params.mask.noise, base_seed),
            mask_enabled: params.mask.enabled,
            mask_contrast: params.mask.contrast.value,
            mask_contrast_enabled: params.mask.contrast.enabled,
            mask_levels: (params.mask.levels.floor, params.mask.levels.ceil),
            mask_levels_enabled: params.mask.levels.enabled,
            warp_noise: build_fbm(&params.warp.noise, base_seed),
            warp_strength: params.warp.strength,
            warp_enabled: params.warp.enabled,
            warp_mask_noise: build_fbm(&params.warp.mask.noise, base_seed),
            warp_mask_enabled: params.warp.mask.enabled,
            warp_mask_contrast: params.warp.mask.contrast.value,
            warp_mask_contrast_enabled: params.warp.mask.contrast.enabled,
            warp_mask_levels: (params.warp.mask.levels.floor, params.warp.mask.levels.ceil),
            warp_mask_levels_enabled: params.warp.mask.levels.enabled,
            gamma: params.gamma.value,
            gamma_enabled: params.gamma.enabled,
            levels: (params.levels.floor, params.levels.ceil),
            levels_enabled: params.levels.enabled,
            hue_noise: build_fbm(&params.palette.hue.noise, base_seed),
            hue_enabled: params.palette.hue.enabled,
            hue_contrast: params.palette.hue.contrast.value,
            hue_contrast_enabled: params.palette.hue.contrast.enabled,
            hue_levels: (
                params.palette.hue.levels.floor,
                params.palette.hue.levels.ceil,
            ),
            hue_levels_enabled: params.palette.hue.levels.enabled,
            drift_noise: build_fbm(&params.palette.drift.noise, base_seed),
            drift_enabled: params.palette.drift.enabled,
            drift_contrast: params.palette.drift.contrast.value,
            drift_contrast_enabled: params.palette.drift.contrast.enabled,
            drift_levels: (
                params.palette.drift.levels.floor,
                params.palette.drift.levels.ceil,
            ),
            drift_levels_enabled: params.palette.drift.levels.enabled,
        }
    }
}

/// Domain-warps a sampling position: offsets it by a displacement built from the
/// smooth `warp_noise`, in two nested levels (after Inigo Quilez). `warp_strength`
/// scales how far the position is pushed.
///
/// When the warp mask is enabled, its noise -- sampled at the original position and
/// shaped by optional levels and contrast -- modulates the displacement amplitude,
/// so the warp can rage in some regions and stay calm in others. The final density
/// lookup happens back in `sample_nebula`, at the returned position.
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

    // mask
    if field.warp_mask_enabled {
        let mut mask =
            ((field.warp_mask_noise.get([point.x, point.y, point.z]) + 1.0) * 0.5) as f32;
        if field.warp_mask_levels_enabled {
            mask = smoothstep(levels(
                mask,
                field.warp_mask_levels.0,
                field.warp_mask_levels.1,
            ));
        }
        if field.warp_mask_contrast_enabled {
            mask = contrast(mask, field.warp_mask_contrast);
        }
        point + field.warp_strength * r * (mask as f64)
    } else {
        point + field.warp_strength * r
    }
}

/// Samples the nebula in a given direction, returning a `NebulaSample`: a density in
/// `[0, 1]` and a hue. Optionally domain-warps the direction, then builds the density
/// by multiplying a ridged-noise structure by a smooth density mask (both sampled at
/// the warped point), shaped by a global gamma and levels pass. The hue -- a separate
/// noise sampled at the same warped point, stretched by its own levels -- feeds the
/// palette lookup in `color::nebula_color`.
pub(super) fn sample_nebula(direction: Vec3, field: &NebulaField) -> NebulaSample {
    // 1. apply domain warp
    let point = if field.warp_enabled {
        domain_warp(direction.as_dvec3(), field)
    } else {
        direction.as_dvec3()
    };

    let mut density = 1.0;

    // 2. sample the ridged noise
    if field.ridge_enabled {
        let mut ridge = ((field.ridged_noise.get(point.to_array()) + 1.0) * 0.5) as f32;
        if field.ridge_gamma_enabled {
            ridge = gamma(ridge, field.ridge_gamma);
        }
        density *= ridge;
    }

    // 3. apply global density mask
    if field.mask_enabled {
        let mut mask = ((field.mask_noise.get(point.to_array()) + 1.0) * 0.5) as f32;
        if field.mask_levels_enabled {
            mask = smoothstep(levels(mask, field.mask_levels.0, field.mask_levels.1));
        }
        if field.mask_contrast_enabled {
            mask = contrast(mask, field.mask_contrast);
        }
        density *= mask;
    }

    if field.gamma_enabled {
        density = gamma(density, field.gamma);
    }

    if field.levels_enabled {
        density = levels(density, field.levels.0, field.levels.1);
    }

    let hue = if field.hue_enabled {
        let mut hue = ((field.hue_noise.get(point.to_array()) + 1.0) * 0.5) as f32;
        if field.hue_levels_enabled {
            hue = levels(hue, field.hue_levels.0, field.hue_levels.1);
        }
        if field.hue_contrast_enabled {
            hue = contrast(hue, field.hue_contrast);
        }
        hue
    } else {
        0.5
    };

    let drift = if field.drift_enabled {
        let mut d = ((field.drift_noise.get(point.to_array()) + 1.0) * 0.5) as f32;
        if field.drift_levels_enabled {
            d = smoothstep(levels(d, field.drift_levels.0, field.drift_levels.1));
        }
        if field.drift_contrast_enabled {
            d = contrast(d, field.drift_contrast);
        }
        d
    } else {
        0.0
    };

    NebulaSample {
        density,
        hue,
        drift,
    }
}
