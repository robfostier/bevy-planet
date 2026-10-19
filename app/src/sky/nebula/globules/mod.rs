//! The Bok globules (`assets/shaders/sky/nebula/globules/`): dust clumps in the
//! nebula, backlit by its gas. Their style (`GlobuleStyle`, locked in code,
//! each setting changing a single thing), their uniforms
//! (`GpuGlobules`), their shapes' atlas (`atlas`) and the mask of where they
//! may show (`mask`).

mod atlas;
mod mask;

use bevy::{prelude::*, render::render_resource::ShaderType};

pub(crate) use atlas::{GlobuleAtlas, shader_defs};
pub(crate) use mask::GlobuleMask;

/// The globules' path in the system, from which their seed derives.
pub(crate) const SEED_PATH: &str = "sky/nebula/globules";

/// The Bok globules: dust clumps in front of the nebula, backlit by its gas.
#[derive(Clone, Copy)]
pub(crate) struct GlobuleStyle {
    /// Offset of the globules' draws (where they stand, their shapes, which
    /// are front-lit) from the system seed.
    pub(crate) seed_offset: u32,
    /// How many large globules over the whole sky (`size_degrees`).
    pub(crate) count_large: f32,
    /// How many small globules (`size_degrees * (1 - size_variation)`); a
    /// medium tier lies in between, in size and count.
    pub(crate) count_small: f32,
    /// How big: the head radius of the largest globules, in degrees.
    pub(crate) size_degrees: f32,
    /// How much smaller the small globules are than the large ones (0 = all
    /// the same size; 0.8 = a fifth of their size).
    pub(crate) size_variation: f32,
    /// How plump their lobes are (0 = torn only).
    pub(crate) buoyancy: f32,
    /// How opaque the globules are, rim included: 1 = a core that
    /// hides the gas behind, 0 = nothing (their edges stay thinner).
    pub(crate) opacity: f32,
    /// Density of the nebula around a globule (whatever its colour; about 0
    /// to 0.1) below which it is absent, and from which it is fully there (in between, the whole globule fades:
    /// they melt into dim gas).
    pub(crate) gas_faint: f32,
    pub(crate) gas_full: f32,
    /// Width of the lit rim on the edge facing the light, in pixels.
    pub(crate) rim: f32,
    /// How much of the rim fades out towards its inner end (0 = a hard band;
    /// 1 = a full gradient, from the edge to nothing `rim` pixels in).
    pub(crate) rim_falloff: f32,
    /// Exposure of the rim's light alone, in stops (each +1 doubles it;
    /// 0 = as bright as the gas behind).
    pub(crate) rim_intensity: f32,
    /// Share of the globules lit from the front (deep in the nebula, drawn at
    /// random): flats of light (`front_intensity`); the others stay backlit.
    pub(crate) front_lit: f32,
    /// Exposure of a front-lit globule's light, in stops (each +1 doubles it;
    /// 0 = as bright as the gas behind).
    pub(crate) front_intensity: f32,
    /// Gilding: the share of the lit edges laid with strokes of saturated
    /// gold (0 = none; 1 = all of them).
    pub(crate) gilding: f32,
}

impl Default for GlobuleStyle {
    fn default() -> Self {
        Self {
            seed_offset: 0,
            count_large: 50.0,
            count_small: 200.0,
            size_degrees: 2.0,
            size_variation: 0.66,
            buoyancy: 0.4,
            opacity: 1.0,
            gas_faint: 0.01,
            gas_full: 0.35,
            rim: 5.0,
            rim_falloff: 1.0,
            rim_intensity: 1.9,
            front_lit: 0.2,
            front_intensity: -0.6,
            gilding: 0.4,
        }
    }
}

/// Mirrors `GlobuleLook` in `globules/look.wgsl`.
#[derive(Clone, Copy, Default, ShaderType)]
pub(crate) struct GpuGlobules {
    count_large: f32,
    count_small: f32,
    /// Radians.
    size: f32,
    size_variation: f32,
    /// Where the draws start (0..1000), from the seed.
    draw_start: f32,
    opacity: f32,
    gas_faint: f32,
    gas_full: f32,
    rim: f32,
    rim_falloff: f32,
    rim_intensity: f32,
    front_lit: f32,
    front_intensity: f32,
    gilding: f32,
    /// The gas left, relative to the style (1).
    density: f32,
}

impl GpuGlobules {
    /// The globules' uniforms; `base_seed`: their seed (derived from the system
    /// seed), offset by `seed_offset`; `density`: the gas left, relative to the
    /// style (fades globules in or out).
    pub(crate) fn new(g: &GlobuleStyle, base_seed: u32, density: f32) -> Self {
        // Where the draws start, in 0..1000: bounded, so the shader's sine hash
        // keeps its precision whatever the seed.
        let seed = f64::from(base_seed.wrapping_add(g.seed_offset));
        let start = ((seed * 0.618_034).fract() * 1000.0) as f32;
        Self {
            count_large: g.count_large,
            count_small: g.count_small,
            size: g.size_degrees.to_radians(),
            size_variation: g.size_variation,
            draw_start: start,
            opacity: g.opacity,
            gas_faint: g.gas_faint,
            gas_full: g.gas_full,
            rim: g.rim,
            rim_falloff: g.rim_falloff,
            rim_intensity: g.rim_intensity,
            front_lit: g.front_lit,
            front_intensity: g.front_intensity,
            gilding: g.gilding,
            density,
        }
    }
}

pub(super) fn build(app: &mut App) {
    atlas::build(app);
    mask::build(app);
}
