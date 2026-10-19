//! The bake's uniforms, mirroring `NebulaParams` in `bake.wgsl`: the style's
//! noise fields (see `noise`) and its warp.

use bevy::{prelude::*, render::render_resource::ShaderType};

use super::noise::{FIELD_COUNT, GpuNoiseField, MAX_OCTAVES, RIDGE_ATTENUATION, noise_field};
use super::params::{NebulaStyle, NoiseParams};

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuNebulaParams {
    warp: GpuNoiseField,
    warp_mask: GpuNoiseField,
    ridge: GpuNoiseField,
    mask: GpuNoiseField,
    hue: GpuNoiseField,
    drift: GpuNoiseField,
    /// x warp strength, y ridge attenuation
    shape: Vec4,
    /// Texels on a side of the fields' faces.
    resolution: u32,
    _pad_a: u32,
    _pad_b: u32,
    _pad_c: u32,
}

/// The bake's uniforms and permutation tables; `base_seed`: the gas's seed
/// (derived from the system seed), which every field offsets by its own.
pub(super) fn build_gpu_params(
    p: &NebulaStyle,
    base_seed: u32,
    resolution: u32,
) -> (GpuNebulaParams, Vec<u32>) {
    let mut tables = vec![0u32; FIELD_COUNT * MAX_OCTAVES * 256];
    let pal = &p.palette;
    // The hue follows the ridge's own noise, over its first octaves.
    let hue_noise = NoiseParams {
        octaves: pal.hue.octaves,
        ..p.ridge.noise
    };
    let params = GpuNebulaParams {
        warp: noise_field(&p.warp.noise, base_seed, 0, false, &mut tables),
        warp_mask: noise_field(&p.warp.mask, base_seed, 1, false, &mut tables),
        ridge: noise_field(&p.ridge.noise, base_seed, 2, true, &mut tables),
        mask: noise_field(&p.mask.noise, base_seed, 3, false, &mut tables),
        hue: noise_field(&hue_noise, base_seed, 4, false, &mut tables),
        drift: noise_field(&pal.drift.noise, base_seed, 5, false, &mut tables),
        shape: Vec4::new(p.warp.strength as f32, RIDGE_ATTENUATION as f32, 0.0, 0.0),
        resolution,
        ..default()
    };
    (params, tables)
}
