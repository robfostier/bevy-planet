//! The star's uniforms, mirroring the structs of `star.wesl` and
//! `corona.wesl`.

use bevy::{prelude::*, render::render_resource::ShaderType};

use super::style::StarStyle;

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuStar {
    core: Vec4,
    mid: Vec4,
    limb: Vec4,
    spot: Vec4,
    crack: Vec4,
    noise: Vec4,
    style: Vec4,
    shape: Vec4,
    chromosphere: Vec4,
    outline: Vec4,
    texture: Vec4,
}

impl From<&StarStyle> for GpuStar {
    fn from(s: &StarStyle) -> Self {
        Self {
            core: s.core.gpu(),
            mid: s.mid.gpu(),
            limb: s.limb.gpu(),
            spot: s.spot.gpu(),
            crack: s.crack.gpu(),
            noise: Vec4::new(s.granule_scale, s.crack_scale, s.flow_speed, s.warp),
            style: Vec4::new(s.chromosphere_width, s.limb_exponent, 0.0, 0.0),
            shape: Vec4::new(
                s.spot_threshold,
                s.crack_width,
                s.crack_coverage,
                s.spot_strength,
            ),
            chromosphere: s.chromosphere.gpu(),
            texture: Vec4::new(s.far_granulation, 0.0, 0.0, 0.0),
            outline: Vec4::new(
                s.outline_amplitude,
                s.outline_frequency,
                s.outline_speed,
                0.0,
            ),
        }
    }
}

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuCorona {
    color: Vec4,
    shape: Vec4,
}

impl GpuCorona {
    pub(super) fn new(s: &StarStyle, radius: f32) -> Self {
        Self {
            color: s.chromosphere.gpu(),
            shape: Vec4::new(radius, s.corona_extent, s.glow_falloff, s.glow_intensity),
        }
    }
}
