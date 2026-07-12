//! Nebula palette: maps a nebula sample (density and hue) to a display color.

use bevy::color::{LinearRgba, Oklcha};
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
    /// Whether the spatial hue drift is applied; when off, the base colors are used.
    pub(super) drift_enabled: bool,
    /// Maximum hue rotation, in degrees, spanned by the spatial drift.
    pub(super) drift_degrees: f32,
    /// Direction bias of the drift in `[-1, 1]` (see `DriftParams::center`).
    pub(super) drift_center: f32,
}

impl Palette {
    /// Builds the palette from the tunable parameters.
    pub(super) fn new(params: &SkyboxParams) -> Self {
        Self {
            enabled: params.palette.enabled,
            primary_color: params.palette.primary_color,
            secondary_color: params.palette.secondary_color,
            intensity: params.palette.intensity,
            drift_enabled: params.palette.drift.enabled,
            drift_degrees: params.palette.drift.degrees,
            drift_center: params.palette.drift.center,
        }
    }
}

/// Rotates a linear-RGB color's hue by `degrees` in OkLCH (perceptual), keeping its
/// lightness and chroma.
fn shift_hue(color: Vec3, degrees: f32) -> Vec3 {
    let mut c = Oklcha::from(LinearRgba::rgb(color.x, color.y, color.z));
    c.hue = (c.hue + degrees).rem_euclid(360.0);
    let l = LinearRgba::from(c);
    Vec3::new(l.red, l.green, l.blue)
}

/// Maps a sample to a color: lerps secondary -> primary by the hue, then scales by
/// density and intensity so voids stay dark. Both palette colors are first hue-drifted
/// by the sample's spatial `drift`, so the color scheme rotates across the sky (the
/// same shift applied to both, keeping the local two-color relationship intact).
pub(super) fn nebula_color(samples: NebulaSample, palette: &Palette) -> Vec3 {
    // The `drift` mask (0..1) is compared against a pivot set by `drift_center`: the
    // rotation is 0 at the pivot and spans up to `drift_degrees` away from it. At
    // center 0 the pivot sits mid-mask (rotates both ways); at -1/+1 it sits at an end,
    // so the whole mask rotates one way or the other. Amplitude stays constant. When
    // the drift is disabled, there is no rotation and the base colors are used.
    let shift = if palette.drift_enabled {
        let pivot = 0.5 - palette.drift_center * 0.5;
        (samples.drift - pivot) * palette.drift_degrees
    } else {
        0.0
    };
    let secondary = shift_hue(palette.secondary_color, shift);
    let primary = shift_hue(palette.primary_color, shift);

    secondary.lerp(primary, samples.hue) * samples.density * palette.intensity
}
