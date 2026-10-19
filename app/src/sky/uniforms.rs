//! The sky's uniforms, mirroring `Sky` in `sky.wgsl`: the gas's look, the
//! globules' own uniforms (from their module), and the nebula's brightness.

use bevy::render::render_resource::ShaderType;

use crate::system::SystemParams;

use super::{
    NEBULA_BRIGHTNESS,
    nebula::{
        gas::GpuNebulaLook,
        globules::{self, GlobuleStyle, GpuGlobules},
        properties::NebulaProperties,
    },
};

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuSky {
    nebula_brightness: f32,
    nebula_look: GpuNebulaLook,
    globule_look: GpuGlobules,
}

impl GpuSky {
    pub(super) fn new(
        globules: &GlobuleStyle,
        system: &SystemParams,
        nebula: &NebulaProperties,
    ) -> Self {
        Self {
            nebula_brightness: NEBULA_BRIGHTNESS,
            nebula_look: nebula.look(),
            globule_look: GpuGlobules::new(
                globules,
                system.derive(globules::SEED_PATH),
                nebula.gas_left(),
            ),
        }
    }
}
