//! The gas's look at runtime (`GpuNebulaLook`, mirroring `NebulaLook` in
//! `sky/nebula/gas.wgsl`): the style's shaping and palette, evolved by the
//! nebula's properties. The sky composes the gas from the baked fields with it
//! every frame, so the system's evolution costs no bake.

use bevy::{prelude::*, render::render_resource::ShaderType};

use super::NebulaStyle;

#[derive(Clone, Copy, Default, ShaderType)]
pub(crate) struct GpuNebulaLook {
    /// The palette's two colours, in Oklab (w unused).
    primary: Vec4,
    secondary: Vec4,
    gamma: f32,
    evaporation: f32,
    mask_floor: f32,
    mask_ceil: f32,
    mask_contrast: f32,
    hue_floor: f32,
    hue_ceil: f32,
    hue_contrast: f32,
    drift_degrees: f32,
    drift_center: f32,
    saturation: f32,
    intensity: f32,
}

fn oklab(color: Vec3) -> Vec4 {
    let lab: Oklaba = LinearRgba::rgb(color.x, color.y, color.z).into();
    Vec4::new(lab.lightness, lab.a, lab.b, 0.0)
}

/// How the style evolves: the share of its filaments' thin edges evaporated,
/// the share its voids close by, and its hue field's shift towards the warm
/// colour.
pub(crate) struct Evolution {
    pub(crate) evaporation: f32,
    pub(crate) filling: f32,
    pub(crate) warm_shift: f32,
}

impl GpuNebulaLook {
    pub(crate) fn new(evolution: &Evolution) -> Self {
        let style = NebulaStyle::default();
        let (pal, mask, hue) = (&style.palette, &style.mask, &style.palette.hue);
        Self {
            primary: oklab(pal.primary_color),
            secondary: oklab(pal.secondary_color),
            // Evaporated, the filaments thin out too.
            gamma: style.ridge.gamma * (1.0 + evolution.evaporation),
            evaporation: evolution.evaporation,
            mask_floor: mask.floor * (1.0 - evolution.filling),
            mask_ceil: mask.ceil,
            mask_contrast: mask.contrast,
            hue_floor: hue.floor - evolution.warm_shift,
            hue_ceil: hue.ceil - evolution.warm_shift,
            hue_contrast: hue.contrast,
            drift_degrees: pal.drift.degrees,
            drift_center: pal.drift.center,
            saturation: pal.saturation,
            intensity: pal.intensity,
        }
    }
}
