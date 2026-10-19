//! The gas's look at runtime (`GpuNebulaLook`, mirroring `NebulaLook` in
//! `sky/nebula/gas/look.wgsl`): the style's shaping and palette. The sky
//! composes the gas from the baked fields with it every frame.

use bevy::{prelude::*, render::render_resource::ShaderType};

use super::NebulaStyle;

#[derive(Clone, Copy, Default, ShaderType)]
pub(crate) struct GpuNebulaLook {
    /// The palette's two colours, in Oklab (w unused).
    primary: Vec4,
    secondary: Vec4,
    gamma: f32,
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

impl GpuNebulaLook {
    pub(crate) fn new() -> Self {
        let style = NebulaStyle::default();
        let (pal, mask, hue) = (&style.palette, &style.mask, &style.palette.hue);
        Self {
            primary: oklab(pal.primary_color),
            secondary: oklab(pal.secondary_color),
            gamma: style.ridge.gamma,
            mask_floor: mask.floor,
            mask_ceil: mask.ceil,
            mask_contrast: mask.contrast,
            hue_floor: hue.floor,
            hue_ceil: hue.ceil,
            hue_contrast: hue.contrast,
            drift_degrees: pal.drift.degrees,
            drift_center: pal.drift.center,
            saturation: pal.saturation,
            intensity: pal.intensity,
        }
    }
}
