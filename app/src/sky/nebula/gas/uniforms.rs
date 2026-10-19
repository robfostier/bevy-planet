//! The bake's uniforms, mirroring `NebulaParams` in `bake.wgsl`: the style's
//! noise fields (see `noise`), its warp and the shaping of its gas (for the
//! globules), and how far the warp's noise has slid with the system's age.

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
    /// x warp strength, y ridge attenuation, z how far the warp's noise has slid
    shape: Vec4,
    /// The style's shaping, for the globules' gas: x ridge gamma, y voids
    /// floor, z voids ceil, w voids contrast.
    style: Vec4,
    /// Texels on a side of the fields', the carry's and the density's faces.
    resolution: u32,
    carry_resolution: u32,
    density_resolution: u32,
    /// The first face of the fields this bake writes (the others follow).
    first_face: u32,
}

impl GpuNebulaParams {
    /// The same, the warp's noise slid by `slide` (noise units).
    pub(super) fn slid(mut self, slide: f32) -> Self {
        self.shape.z = slide;
        self
    }

    /// The same, the fields' faces written from `first_face`.
    pub(super) fn starting_at_face(mut self, first_face: u32) -> Self {
        self.first_face = first_face;
        self
    }

    /// The same, the fields baked at `resolution` texels a side.
    pub(super) fn at_resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }
}

/// The bake's uniforms and permutation tables; `base_seed`: the gas's seed
/// (derived from the system seed), which every field offsets by its own;
/// `resolutions`: the fields', the carry's and the density's.
pub(super) fn build_gpu_params(
    p: &NebulaStyle,
    base_seed: u32,
    [resolution, carry_resolution, density_resolution]: [u32; 3],
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
        style: Vec4::new(p.ridge.gamma, p.mask.floor, p.mask.ceil, p.mask.contrast),
        resolution,
        carry_resolution,
        density_resolution,
        ..default()
    };
    (params, tables)
}
