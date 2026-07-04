//! Live-tunable parameters for the nebula skybox, edited through the egui
//! inspector in `dev_tools` and consumed by the generation in `field`.

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

/// The classic FBM (fractional Brownian motion) controls for one noise field.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct NoiseParams {
    /// Chooses the noise pattern; two fields with different seeds are independent.
    pub(crate) seed: u32,
    /// Base scale of the first octave.
    pub(crate) frequency: f64,
    /// Octaves summed together; more octaves add finer detail (and cost).
    #[inspector(min = 1, max = 8)]
    pub(crate) octaves: usize,
    /// Frequency multiplier between successive octaves (~2.0).
    pub(crate) lacunarity: f64,
    /// Amplitude falloff between octaves (~0.5): higher is rougher, lower smoother.
    #[inspector(min = 0.0, max = 1.0)]
    pub(crate) persistence: f64,
}

/// Live-tunable parameters for the procedural skybox, edited at runtime through
/// the egui inspector and consumed by `rebuild_skybox_cubemap`.
#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct SkyboxParams {
    /// The detailed field, sampled at the warped position.
    pub(crate) density: NoiseParams,
    /// The smooth field that displaces the sampling position.
    pub(crate) warp: NoiseParams,
    /// How far the warp displaces the sampling position.
    pub(crate) warp_strength: f64,
}

impl Default for SkyboxParams {
    fn default() -> Self {
        Self {
            density: NoiseParams {
                seed: 0,
                frequency: 2.0,
                octaves: 5,
                lacunarity: 2.0,
                persistence: 0.55,
            },
            warp: NoiseParams {
                seed: 1,
                frequency: 1.0,
                octaves: 2,
                lacunarity: 2.0,
                persistence: 0.5,
            },
            warp_strength: 0.4,
        }
    }
}
