//! The building blocks of `NebulaStyle`: one struct per stage of the nebula
//! (noise, warp, ridge, mask, drift, palette).

use bevy::prelude::*;

/// Fractal-noise controls for one noise field, shared by the FBM and ridged generators.
#[derive(Clone, Copy)]
pub(crate) struct NoiseParams {
    /// Offset of the noise pattern from the system seed; two fields with
    /// different offsets are independent.
    pub(crate) seed_offset: u32,
    /// Base scale of the first octave.
    pub(crate) frequency: f64,
    /// Octaves summed together; more octaves add finer detail (and cost).
    pub(crate) octaves: usize,
    /// Frequency multiplier between successive octaves (~2.0).
    pub(crate) lacunarity: f64,
    /// Amplitude falloff between octaves (~0.5): higher is rougher, lower smoother.
    pub(crate) persistence: f64,
}

/// Domain warp: how far the sampling position is pushed, the noise that drives
/// it, and a mask that modulates its amplitude per region.
#[derive(Clone, Copy)]
pub(crate) struct WarpParams {
    /// How far the sampling position is pushed by the warp.
    pub(crate) strength: f64,
    /// The noise field that drives the displacement.
    pub(crate) noise: NoiseParams,
    /// Modulates the warp amplitude per region (calm vs turbulent zones).
    pub(crate) mask: NoiseParams,
}

/// The ridged fractal that forms the nebula's filaments.
#[derive(Clone, Copy)]
pub(crate) struct RidgeParams {
    pub(crate) noise: NoiseParams,
    /// Gamma shaping the ridge's falloff: above 1 thins the filaments.
    pub(crate) gamma: f32,
}

/// A noise field remapped by levels (`[floor, ceil]` onto `[0, 1]`) then an
/// S-curve contrast: the density mask.
#[derive(Clone, Copy)]
pub(crate) struct MaskParams {
    pub(crate) noise: NoiseParams,
    /// Input value that maps to 0; everything below is clamped to 0.
    pub(crate) floor: f32,
    /// Input value that maps to 1; everything above is clamped to 1.
    pub(crate) ceil: f32,
    /// Strength of the S-curve: higher is punchier; 1.0 leaves the value unchanged.
    pub(crate) contrast: f32,
}

/// The hue field: the ridge's own noise (same seed, frequency and octave
/// steps, so the colours follow the filaments) over its first `octaves`,
/// remapped by levels then an S-curve contrast, as a `MaskParams`.
#[derive(Clone, Copy)]
pub(crate) struct HueParams {
    /// How many of the ridge's octaves the hue follows (1: its broad shapes).
    pub(crate) octaves: usize,
    pub(crate) floor: f32,
    pub(crate) ceil: f32,
    pub(crate) contrast: f32,
}

/// Spatial hue drift: a noise field whose value rotates the whole palette's
/// hue across the sky.
#[derive(Clone, Copy)]
pub(crate) struct DriftParams {
    pub(crate) noise: NoiseParams,
    /// Maximum hue rotation, in degrees, spanned by the drift.
    pub(crate) degrees: f32,
    /// Direction bias: 0 centers the rotation (both ways along the hue wheel), -1 and
    /// +1 send it all one way or the other.
    pub(crate) center: f32,
}

/// All the color information: the two ramp colors, their global intensity, the hue
/// field selecting each pixel's tint, and the spatial drift of the whole ramp.
#[derive(Clone, Copy)]
pub(crate) struct PaletteParams {
    /// Color at hue = 1 (one end of the ramp).
    pub(crate) primary_color: Vec3,
    /// Color at hue = 0 (the other end of the ramp).
    pub(crate) secondary_color: Vec3,
    /// Multiplies the final color, allowing the nebula to be dimmed or brightened.
    pub(crate) intensity: f32,
    /// Chroma multiplier of the mixed colour (1 = as mixed, higher = more vivid).
    pub(crate) saturation: f32,
    /// Hue field selecting each pixel's tint along the ramp.
    pub(crate) hue: HueParams,
    /// Spatial hue drift of the whole ramp across the sky.
    pub(crate) drift: DriftParams,
}
