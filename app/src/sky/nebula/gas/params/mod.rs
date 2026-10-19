//! Live-tunable parameters for the nebula's gas, edited through the egui
//! inspector in `dev_tools` and consumed by the GPU bake (`gas`).

mod stages;

use bevy::prelude::*;

pub(crate) use stages::{
    DriftParams, HueParams, MaskParams, NoiseParams, PaletteParams, RidgeParams, WarpParams,
};

/// Live-tunable parameters for the nebula's gas, edited at runtime through
/// the egui inspector and baked whenever they change.
#[derive(Clone, Copy)]
pub(crate) struct NebulaStyle {
    /// Domain-warp applied to the sampling direction, before anything else.
    pub(crate) warp: WarpParams,
    /// Ridged-noise structure: the nebula's filaments.
    pub(crate) ridge: RidgeParams,
    /// Global density mask carving the large-scale voids.
    pub(crate) mask: MaskParams,
    /// All color information: ramp colors, hue selection, and spatial drift.
    pub(crate) palette: PaletteParams,
}

/// A noise field with the given seed offset, frequency and octaves, and the usual
/// lacunarity (2) and persistence (0.5).
fn noise(seed_offset: u32, frequency: f64, octaves: usize) -> NoiseParams {
    NoiseParams {
        seed_offset,
        frequency,
        octaves,
        lacunarity: 2.0,
        persistence: 0.5,
    }
}

impl Default for NebulaStyle {
    fn default() -> Self {
        Self {
            warp: WarpParams {
                strength: 0.3,
                noise: NoiseParams {
                    lacunarity: 4.0,
                    persistence: 0.25,
                    ..noise(0, 3.0, 4)
                },
                mask: noise(0, 6.0, 3),
            },
            ridge: RidgeParams {
                noise: noise(0, 2.0, 3),
                gamma: 2.82,
            },
            mask: MaskParams {
                noise: noise(18, 1.0, 3),
                floor: 0.52,
                ceil: 1.0,
                contrast: 1.5,
            },
            palette: PaletteParams {
                primary_color: Vec3::new(0.4, 0.15, 0.0),
                secondary_color: Vec3::new(0.01, 0.12, 0.35),
                intensity: 0.55,
                saturation: 1.5,
                hue: HueParams {
                    octaves: 1,
                    floor: 0.15,
                    ceil: 1.0,
                    contrast: 6.0,
                },
                drift: DriftParams {
                    noise: noise(0, 0.5, 1),
                    degrees: 80.0,
                    center: -1.0,
                },
            },
        }
    }
}
