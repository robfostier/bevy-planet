//! The noise fields' uniforms: the `noise` crate's permutation tables (one per
//! field and octave, seeded exactly as `Fbm`/`RidgedMulti` do) and the scale
//! factors that bring their sums back to [-1, 1].

use bevy::{prelude::*, render::render_resource::ShaderType};
use noise::permutationtable::{NoiseHasher, PermutationTable};

use super::params::NoiseParams;

/// Octave cap for the permutation-table upload (the inspector caps at 8 too).
pub(super) const MAX_OCTAVES: usize = 8;
/// Number of noise fields: warp, warp mask, ridge, mask, hue, drift.
pub(super) const FIELD_COUNT: usize = 6;
/// `RidgedMulti::DEFAULT_ATTENUATION`.
pub(super) const RIDGE_ATTENUATION: f64 = 2.0;

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuNoiseField {
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    scale_factor: f32,
    octaves: u32,
    table_base: u32,
    _pad0: u32,
    _pad1: u32,
}

/// `Fbm::calc_scale_factor`.
fn fbm_scale_factor(persistence: f64, octaves: usize) -> f64 {
    1.0 / (1..=octaves).fold(0.0, |acc, x| acc + persistence.powi(x as i32))
}

/// `RidgedMulti::calc_scale_factor` (note the `octaves + 1` terms, as upstream).
fn ridged_scale_factor(persistence: f64, attenuation: f64, octaves: usize) -> f64 {
    let mut amplitude = 1.0;
    let mut signal: f64 = 1.0;
    let mut denom = signal;
    for x in 1..=octaves {
        amplitude *= persistence;
        let weight = (signal / attenuation.powi(x as i32)).clamp(0.0, 1.0);
        signal = weight * amplitude;
        denom += signal;
    }
    2.0 / denom
}

/// Appends the permutation tables of one field (`seed + octave`, as
/// `fractals::build_sources` does; its seed: the gas's `base_seed` plus the
/// field's own offset) and returns its GPU descriptor.
pub(super) fn noise_field(
    params: &NoiseParams,
    base_seed: u32,
    index: usize,
    ridged: bool,
    tables: &mut [u32],
) -> GpuNoiseField {
    let octaves = params.octaves.clamp(1, MAX_OCTAVES);
    let base = index * MAX_OCTAVES * 256;
    for octave in 0..octaves {
        let table = PermutationTable::new(
            params
                .seed_offset
                .wrapping_add(base_seed)
                .wrapping_add(octave as u32),
        );
        for i in 0..256 {
            tables[base + octave * 256 + i] = table.hash(&[i as isize]) as u32;
        }
    }
    let scale_factor = if ridged {
        ridged_scale_factor(params.persistence, RIDGE_ATTENUATION, octaves)
    } else {
        fbm_scale_factor(params.persistence, octaves)
    };
    GpuNoiseField {
        frequency: params.frequency as f32,
        lacunarity: params.lacunarity as f32,
        persistence: params.persistence as f32,
        scale_factor: scale_factor as f32,
        octaves: octaves as u32,
        table_base: base as u32,
        ..default()
    }
}
