//! Nebula palette: maps a nebula sample (density and hue) to a display color.

use bevy::prelude::*;

use crate::skybox::{SkyboxParams, field::NebulaSample};

/// The nebula's color ramp -- two colors interpolated by the sample's hue, plus a
/// global intensity -- built from `PaletteParams`.
pub(super) struct Palette {
    /// Whether to colorize; when off, the density is written as grayscale.
    pub(super) enabled: bool,
    /// Color at hue = 1 (one end of the ramp).
    pub(super) primary_color: Vec3,
    /// Color at hue = 0 (the other end of the ramp).
    pub(super) secondary_color: Vec3,
    /// Global multiplier on the final color (dims or brightens the whole nebula).
    pub(super) intensity: f32,
}

impl Palette {
    /// Builds the palette from the tunable parameters.
    pub(super) fn new(params: &SkyboxParams) -> Self {
        Self {
            enabled: params.palette.enabled,
            primary_color: params.palette.primary_color,
            secondary_color: params.palette.secondary_color,
            intensity: params.palette.intensity,
        }
    }
}

/// Maps a sample to a color: lerps secondary -> primary by the hue, then scales by
/// density and intensity so voids stay dark.
pub(super) fn nebula_color(samples: NebulaSample, palette: &Palette) -> Vec3 {
    palette
        .secondary_color
        .lerp(palette.primary_color, samples.hue)
        * samples.density
        * palette.intensity
}
