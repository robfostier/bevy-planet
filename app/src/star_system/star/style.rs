//! The star's look, as `StarStyle` and its `Tone`s (a color with an HDR
//! intensity). Its default is the art direction's base star, a Sun-like star of
//! one solar mass; other stars derive from it through their mass (`physics`).

use bevy::prelude::*;

/// A color with an intensity multiplier, in display-linear HDR units.
#[derive(Clone, Copy)]
pub(crate) struct Tone {
    pub(crate) color: Vec3,
    pub(crate) intensity: f32,
}

impl Tone {
    const fn new(r: f32, g: f32, b: f32, intensity: f32) -> Self {
        Self {
            color: Vec3::new(r, g, b),
            intensity,
        }
    }

    pub(super) fn gpu(&self) -> Vec4 {
        self.color.extend(self.intensity)
    }
}

/// Every setting of the star's look.
#[derive(Clone)]
pub(crate) struct StarStyle {
    /// Disc centre.
    pub(crate) core: Tone,
    /// Between centre and limb.
    pub(crate) mid: Tone,
    /// Darkened, reddened edge (limb darkening).
    pub(crate) limb: Tone,
    /// Bright granulation patches.
    pub(crate) spot: Tone,
    /// Dark ridges between the patches.
    pub(crate) crack: Tone,
    /// Granulation frequency on the unit sphere.
    pub(crate) granule_scale: f32,
    /// Crack frequency on the unit sphere.
    pub(crate) crack_scale: f32,
    /// Flow speed of the surface pattern.
    pub(crate) flow_speed: f32,
    /// Domain-warp amount of the surface pattern.
    pub(crate) warp: f32,
    /// Granulation strength seen from afar, where it recedes so the
    /// core-to-limb gradient reads as a smooth, painted blend (1 = as up close).
    pub(crate) far_granulation: f32,
    /// How far the surface is pushed in and out, as a fraction of the radius,
    /// so the silhouette is not a perfect circle.
    pub(crate) outline_amplitude: f32,
    /// Frequency of the silhouette's irregularities on the unit sphere.
    pub(crate) outline_frequency: f32,
    pub(crate) outline_speed: f32,
    /// Thin bright rim at the very edge, past the darkened limb; the corona
    /// rises from it and takes its color.
    pub(crate) chromosphere: Tone,
    /// Width of the chromosphere, as a fraction of the star's radius.
    pub(crate) chromosphere_width: f32,
    /// Exponent of the radial limb gradient: higher keeps the centre bright longer.
    pub(crate) limb_exponent: f32,
    /// Granulation threshold: higher = fewer, smaller bright patches.
    pub(crate) spot_threshold: f32,
    pub(crate) spot_strength: f32,
    pub(crate) crack_width: f32,
    /// Where cracks may appear (relative to the granulation): higher = fewer.
    pub(crate) crack_coverage: f32,
    /// Size of the corona quad, in star radii.
    pub(crate) corona_extent: f32,
    /// Glow falloff distance, in star radii.
    pub(crate) glow_falloff: f32,
    pub(crate) glow_intensity: f32,
    /// Color of the star's light on the planets.
    pub(crate) light: Vec3,
}

impl Default for StarStyle {
    /// The base star (one solar mass): hot, slightly orange just beyond white,
    /// with a warm limb, fine granulation and a bright chromosphere.
    fn default() -> Self {
        Self {
            core: Tone::new(1.0, 0.75, 0.36, 3.6),
            mid: Tone::new(1.0, 0.45, 0.1, 2.2),
            limb: Tone::new(1.0, 0.2, 0.03, 1.5),
            spot: Tone::new(1.0, 0.72, 0.3, 4.0),
            crack: Tone::new(0.85, 0.2, 0.03, 1.6),
            granule_scale: 9.0,
            crack_scale: 13.0,
            flow_speed: 0.04,
            warp: 1.0,
            far_granulation: 0.8,
            outline_amplitude: 0.03,
            outline_frequency: 3.0,
            outline_speed: 0.05,
            chromosphere: Tone::new(1.0, 0.5, 0.14, 3.0),
            chromosphere_width: 0.01,
            limb_exponent: 2.5,
            spot_threshold: 0.05,
            spot_strength: 0.5,
            crack_width: 0.1,
            crack_coverage: 0.5,
            corona_extent: 2.0,
            glow_falloff: 0.07,
            glow_intensity: 0.8,
            light: Vec3::new(1.0, 0.92, 0.82),
        }
    }
}
