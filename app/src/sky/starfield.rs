//! The starfield (`assets/shaders/sky/starfield.wgsl`): its tunables
//! (`StarfieldStyle`, edited live in the inspector) and their uniforms.

use bevy::{prelude::*, render::render_resource::ShaderType};

/// One star population: its cell size, how many cells hold a star, and how bright.
#[derive(Clone, Copy)]
pub(crate) struct StarLayerStyle {
    pub(crate) enabled: bool,
    /// Cells per unit length on the direction sphere: higher = denser, smaller spacing.
    pub(crate) scale: f32,
    /// Probability that a cell holds a star.
    pub(crate) density: f32,
    /// Peak brightness of the layer's brightest star.
    pub(crate) brightness: f32,
    /// Brightness distribution exponent: higher = fewer bright stars.
    pub(crate) falloff: f32,
    /// Extra star radius, in arcminutes, on top of `star_size_arcmin`.
    pub(crate) size_arcmin: f32,
    /// How much the nebula hides this layer (multiplies the global `occlusion`).
    pub(crate) occlusion: f32,
}

/// The starfield.
#[derive(Clone, Copy)]
pub(crate) struct StarfieldStyle {
    pub(crate) enabled: bool,
    /// Brightness multiplier of the stars, in cd/m^2 (scaled by the exposure).
    pub(crate) intensity: f32,
    /// Gaussian sigma of a point star, in arcminutes: a fixed angle, so stars
    /// behave like a resized image whatever the window. (0.84' is 0.45 px of a
    /// 1440 px tall view at the camera's 45-degree FOV.)
    pub(crate) star_size_arcmin: f32,
    /// 0 = white stars, 1 = full blackbody colors.
    pub(crate) saturation: f32,
    pub(crate) temp_min: f32,
    pub(crate) temp_max: f32,
    /// Temperature distribution exponent: above 1 favours cool (orange/red) stars.
    pub(crate) temp_bias: f32,
    /// Offset of the stars' draws from the system seed.
    pub(crate) seed_offset: u32,
    /// Extinction of the stars by the displayed nebula: they fade as
    /// `exp(-occlusion * layer.occlusion * nebula_luminance)`. The displayed
    /// luminance is only ~0.005 to 0.05, hence values around 100. 0 disables it.
    pub(crate) occlusion: f32,
    /// Four-point sparkles on the brightest stars (0 = off).
    pub(crate) sparkle: f32,
    /// Star brightness above which a star sparkles: keep it high, so only a
    /// handful of the brightest stars do.
    pub(crate) sparkle_threshold: f32,
    /// Sparkle arm length at the threshold, in arcminutes (grows with sqrt(brightness)).
    pub(crate) sparkle_length_arcmin: f32,
    pub(crate) small: StarLayerStyle,
    pub(crate) medium: StarLayerStyle,
    /// Rare, bright stars that bloom.
    pub(crate) hero: StarLayerStyle,
}

impl Default for StarfieldStyle {
    fn default() -> Self {
        let layer = |scale, density, brightness, falloff, occlusion| StarLayerStyle {
            enabled: true,
            scale,
            density,
            brightness,
            falloff,
            size_arcmin: 0.0,
            occlusion,
        };
        Self {
            enabled: true,
            intensity: 1.5e4,
            star_size_arcmin: 0.84,
            saturation: 1.0,
            temp_min: 2500.0,
            temp_max: 14000.0,
            temp_bias: 1.4,
            seed_offset: 0,
            occlusion: 120.0,
            sparkle: 0.15,
            sparkle_threshold: 7.5,
            sparkle_length_arcmin: 5.0,
            // Calibrated on full-sky captures against the Chernoschokov
            // references: star counts per brightness bin and bright-star size
            // within ~20% of the references.
            small: layer(110.0, 0.55, 0.03, 1.8, 1.0),
            medium: layer(70.0, 0.6, 0.4, 3.5, 1.2),
            hero: layer(32.0, 0.9, 8.0, 4.5, 2.0),
        }
    }
}

/// Mirrors `StarLayer` in `starfield.wgsl`.
#[derive(Clone, Copy, Default, ShaderType)]
pub(crate) struct GpuStarLayer {
    scale: f32,
    density: f32,
    brightness: f32,
    falloff: f32,
    /// Radians.
    size: f32,
    enabled: u32,
    occlusion: f32,
    _pad0: u32,
}

/// Mirrors `StarLook` in `starfield.wgsl`.
#[derive(Clone, Copy, Default, ShaderType)]
pub(crate) struct GpuStarLook {
    /// Radians.
    star_size: f32,
    saturation: f32,
    temp_min: f32,
    temp_max: f32,
    temp_bias: f32,
    seed: u32,
    min_sigma_px: f32,
    /// Share of each layer's stars shown, relative to the style (1).
    thinning: f32,
    sparkle: Vec4,
}

/// The share of stars a system gains when newborn (still in its star-forming
/// region) and loses at the end of its star's life, relative to the style,
/// which shows a system at mid-life.
const STAR_SWING: f32 = 0.6;

/// The share of the style's stars shown at the system's `age` (0..1): from
/// 1 + STAR_SWING newborn, through 1 at mid-life, to 1 - STAR_SWING at the end.
pub(crate) fn thinning(age: f32) -> f32 {
    1.0 + (1.0 - 2.0 * age) * STAR_SWING
}

fn arcmin_to_radians(arcmin: f32) -> f32 {
    (arcmin / 60.0).to_radians()
}

impl StarfieldStyle {
    /// The three layers' uniforms, faintest first.
    pub(crate) fn gpu_layers(&self) -> [GpuStarLayer; 3] {
        [&self.small, &self.medium, &self.hero].map(|l| GpuStarLayer {
            scale: l.scale,
            density: l.density,
            brightness: l.brightness,
            falloff: l.falloff,
            size: arcmin_to_radians(l.size_arcmin),
            enabled: l.enabled as u32,
            occlusion: l.occlusion,
            ..default()
        })
    }

    /// The look's uniforms; `base_seed`: the starfield's seed (derived from
    /// the system seed), offset by `seed_offset`; `thinning`: the share of
    /// stars still shown (see `thinning`).
    pub(crate) fn gpu_look(&self, base_seed: u32, thinning: f32) -> GpuStarLook {
        GpuStarLook {
            star_size: arcmin_to_radians(self.star_size_arcmin),
            saturation: self.saturation,
            temp_min: self.temp_min,
            temp_max: self.temp_max,
            temp_bias: self.temp_bias,
            seed: base_seed.wrapping_add(self.seed_offset),
            min_sigma_px: 0.4,
            thinning,
            sparkle: Vec4::new(
                self.sparkle,
                self.sparkle_threshold,
                arcmin_to_radians(self.sparkle_length_arcmin),
                0.0,
            ),
        }
    }
}
