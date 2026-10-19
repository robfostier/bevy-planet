//! The sky's uniforms, mirroring `Sky` in `sky.wgsl`: the gas's look and the
//! nebula's brightness.

use bevy::render::render_resource::ShaderType;

use super::{NEBULA_BRIGHTNESS, nebula::gas::GpuNebulaLook};

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuSky {
    nebula_brightness: f32,
    nebula_look: GpuNebulaLook,
}

impl GpuSky {
    pub(super) fn new() -> Self {
        Self {
            nebula_brightness: NEBULA_BRIGHTNESS,
            nebula_look: GpuNebulaLook::new(),
        }
    }
}
