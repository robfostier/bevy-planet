//! The sky's uniforms, mirroring `Sky` in `sky.wesl`: the starfield's and the
//! globules' own uniforms (each from its module), and the nebula's brightness.

use bevy::render::render_resource::ShaderType;

use crate::system::SystemParams;

use super::{
    NEBULA_BRIGHTNESS,
    nebula::{
        gas::GpuNebulaLook,
        globules::{self, GlobuleStyle, GpuGlobules},
        properties::NebulaProperties,
    },
    starfield::{self, GpuStarLayer, GpuStarLook, StarfieldStyle},
};

#[derive(Clone, Copy, Default, ShaderType)]
pub(super) struct GpuSky {
    layers: [GpuStarLayer; 3],
    stars: GpuStarLook,
    intensity: f32,
    enabled: u32,
    occlusion: f32,
    nebula_brightness: f32,
    nebula_look: GpuNebulaLook,
    globule_look: GpuGlobules,
}

impl GpuSky {
    pub(super) fn new(
        stars: &StarfieldStyle,
        globules: &GlobuleStyle,
        system: &SystemParams,
        nebula: &NebulaProperties,
    ) -> Self {
        Self {
            layers: stars.gpu_layers(),
            stars: stars.gpu_look(
                system.derive("sky/starfield"),
                starfield::thinning(system.age),
            ),
            intensity: stars.intensity,
            enabled: stars.enabled as u32,
            occlusion: stars.occlusion,
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
