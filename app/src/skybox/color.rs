//! Nebula palette: maps the scalar density field to a display color.

use bevy::prelude::*;

use crate::skybox::SkyboxParams;

/// The nebula's color ramp -- edge and core colors plus a global intensity --
/// built from `PaletteParams`.
pub(super) struct Palette {
    /// Whether to colorize; when off, the density is written as grayscale.
    pub(super) enabled: bool,
    /// Color at full density (the nebula's bright core).
    pub(super) core_color: Vec3,
    /// Color at low density (the nebula's faint edge).
    pub(super) edge_color: Vec3,
    /// Global multiplier on the final color (dims or brightens the whole nebula).
    pub(super) intensity: f32,
}

impl Palette {
    /// Builds the palette from the tunable parameters.
    pub(super) fn new(params: &SkyboxParams) -> Self {
        Self {
            enabled: params.palette.enabled,
            core_color: params.palette.core_color,
            edge_color: params.palette.edge_color,
            intensity: params.palette.intensity,
        }
    }
}

/// Maps a scalar density to a color: lerps edge -> core by density, then scales by
/// density and intensity so faint regions stay dim.
pub(super) fn nebula_color(density: f32, palette: &Palette) -> Vec3 {
    palette.edge_color.lerp(palette.core_color, density) * density * palette.intensity
}
