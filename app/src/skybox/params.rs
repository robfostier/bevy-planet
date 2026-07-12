//! Live-tunable parameters for the nebula skybox, edited through the egui
//! inspector in `dev_tools` and consumed by the generation in `field`.

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

/// Fractal-noise controls for one noise field, shared by the FBM and ridged generators.
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

/// A levels remap: rescales `[floor, ceil]` onto `[0, 1]`, clamping outside.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct LevelsParams {
    /// Whether the remap is applied.
    pub(crate) enabled: bool,
    /// Input value that maps to 0; everything below is clamped to 0.
    #[inspector(min = 0.0, max = 1.0, speed = 0.01)]
    pub(crate) floor: f32,
    /// Input value that maps to 1; everything above is clamped to 1.
    #[inspector(min = 0.0, max = 1.0, speed = 0.01)]
    pub(crate) ceil: f32,
}

/// A gamma curve applied to a value (`value.powf(gamma)`): below 1 brightens, above 1 darkens.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct GammaParams {
    /// Whether the gamma curve is applied.
    pub(crate) enabled: bool,
    /// The exponent fed to `value.powf(..)`.
    #[inspector(min = 0.1, max = 5.0, speed = 0.01)]
    pub(crate) value: f32,
}

/// An S-curve contrast around the midpoint: higher `value` pushes tones toward 0 or 1.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct ContrastParams {
    /// Whether the contrast curve is applied.
    pub(crate) enabled: bool,
    /// Strength of the S-curve: higher is punchier; 1.0 leaves the value unchanged.
    pub(crate) value: f32,
}

/// Domain-warp stage: how far the sampling position is pushed, the noise that
/// drives it, and a mask that modulates the warp amplitude per region.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct WarpParams {
    /// Whether the direction is domain-warped before sampling.
    pub(crate) enabled: bool,
    /// How far the sampling position is pushed by the warp.
    #[inspector(min = 0.0, max = 4.0, speed = 0.01)]
    pub(crate) strength: f64,
    /// The noise field that drives the displacement.
    pub(crate) noise: NoiseParams,
    /// Modulates the warp amplitude per region (calm vs turbulent zones).
    pub(crate) mask: MaskParams,
}

/// Ridged-noise structure stage: the fractal that forms the nebula's filaments,
/// with an optional gamma to shape its falloff.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct RidgeParams {
    /// Whether the ridged-noise structure is multiplied into the density.
    pub(crate) enabled: bool,
    /// The ridged fractal noise forming the filaments.
    pub(crate) noise: NoiseParams,
    /// Optional gamma shaping the ridge's falloff.
    pub(crate) gamma: GammaParams,
}

/// A mask: a noise field shaped by optional levels and contrast. Reused both as
/// the global density mask and as the warp-amplitude mask.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct MaskParams {
    /// Whether the mask is multiplied into its target.
    pub(crate) enabled: bool,
    /// The noise field sampled for the mask.
    pub(crate) noise: NoiseParams,
    /// Optional contrast shaping the mask.
    pub(crate) contrast: ContrastParams,
    /// Optional levels remap shaping the mask.
    pub(crate) levels: LevelsParams,
}

/// Spatial hue-drift stage: a mask (shaped by contrast/levels) whose value rotates the
/// whole palette's hue across the sky, plus how far it rotates and its direction bias.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct DriftParams {
    /// Whether the palette hue drift is applied.
    pub(crate) enabled: bool,
    /// The noise field driving the drift.
    pub(crate) noise: NoiseParams,
    /// Optional contrast shaping the drift mask.
    pub(crate) contrast: ContrastParams,
    /// Optional levels remap shaping the drift mask.
    pub(crate) levels: LevelsParams,
    /// Maximum hue rotation, in degrees, spanned by the drift.
    #[inspector(min = 0.0, max = 360.0)]
    pub(crate) degrees: f32,
    /// Direction bias: 0 centers the rotation (both ways along the hue wheel), -1 and
    /// +1 send it all one way or the other. Which tint each way lands on depends on
    /// the chosen colors.
    #[inspector(min = -1.0, max = 1.0, speed = 0.01)]
    pub(crate) center: f32,
}

/// All the color information: the two ramp colors, their global intensity, the hue
/// field selecting each pixel's tint, and the spatial drift of the whole ramp.
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(crate) struct PaletteParams {
    /// Whether the palette is applied; when off, the density shows as grayscale.
    pub(crate) enabled: bool,
    /// Color at hue = 1 (one end of the ramp).
    pub(crate) primary_color: Vec3,
    /// Color at hue = 0 (the other end of the ramp).
    pub(crate) secondary_color: Vec3,
    /// Multiplies the final color, allowing the nebula to be dimmed or brightened.
    pub(crate) intensity: f32,
    /// Hue field selecting each pixel's tint along the ramp.
    pub(crate) hue: MaskParams,
    /// Spatial hue drift of the whole ramp across the sky.
    pub(crate) drift: DriftParams,
}

/// Live-tunable parameters for the procedural skybox, edited at runtime through
/// the egui inspector and consumed by `rebuild_skybox_cubemap`.
#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct SkyboxParams {
    /// Domain-warp applied to the sampling direction, before anything else.
    pub(crate) warp: WarpParams,
    /// Ridged-noise structure: the nebula's filaments.
    pub(crate) ridge: RidgeParams,
    /// Global density mask carving the large-scale voids.
    pub(crate) mask: MaskParams,
    /// All color information: ramp colors, hue selection, and spatial drift.
    pub(crate) palette: PaletteParams,
    /// Final gamma applied to the density.
    pub(crate) gamma: GammaParams,
    /// Final levels remap applied to the density.
    pub(crate) levels: LevelsParams,
}

impl Default for SkyboxParams {
    fn default() -> Self {
        Self {
            warp: WarpParams {
                enabled: false,
                strength: 0.3,
                noise: NoiseParams {
                    seed: 0,
                    frequency: 3.0,
                    octaves: 4,
                    lacunarity: 4.0,
                    persistence: 0.25,
                },
                mask: MaskParams {
                    enabled: true,
                    noise: NoiseParams {
                        seed: 0,
                        frequency: 6.0,
                        octaves: 3,
                        lacunarity: 2.0,
                        persistence: 0.5,
                    },
                    contrast: ContrastParams {
                        enabled: false,
                        value: 1.0,
                    },
                    levels: LevelsParams {
                        enabled: false,
                        floor: 0.0,
                        ceil: 1.0,
                    },
                },
            },
            ridge: RidgeParams {
                enabled: true,
                noise: NoiseParams {
                    seed: 0,
                    frequency: 4.0,
                    octaves: 3,
                    lacunarity: 2.0,
                    persistence: 0.5,
                },
                gamma: GammaParams {
                    enabled: true,
                    value: 1.5,
                },
            },
            mask: MaskParams {
                enabled: true,
                noise: NoiseParams {
                    seed: 18,
                    frequency: 1.0,
                    octaves: 3,
                    lacunarity: 2.0,
                    persistence: 0.5,
                },
                contrast: ContrastParams {
                    enabled: true,
                    value: 1.5,
                },
                levels: LevelsParams {
                    enabled: true,
                    floor: 0.61,
                    ceil: 1.0,
                },
            },
            palette: PaletteParams {
                enabled: false,
                primary_color: Vec3::new(0.4, 0.15, 0.0),
                secondary_color: Vec3::new(0.01, 0.12, 0.35),
                intensity: 0.55,
                hue: MaskParams {
                    enabled: true,
                    noise: NoiseParams {
                        seed: 0,
                        frequency: 4.0,
                        octaves: 1,
                        lacunarity: 2.0,
                        persistence: 0.5,
                    },
                    contrast: ContrastParams {
                        enabled: true,
                        value: 6.0,
                    },
                    levels: LevelsParams {
                        enabled: true,
                        floor: 0.15,
                        ceil: 1.0,
                    },
                },
                drift: DriftParams {
                    enabled: true,
                    noise: NoiseParams {
                        seed: 0,
                        frequency: 0.5,
                        octaves: 1,
                        lacunarity: 2.0,
                        persistence: 0.5,
                    },
                    contrast: ContrastParams {
                        enabled: false,
                        value: 1.0,
                    },
                    levels: LevelsParams {
                        enabled: false,
                        floor: 0.0,
                        ceil: 1.0,
                    },
                    degrees: 80.0,
                    center: -1.0,
                },
            },
            gamma: GammaParams {
                enabled: false,
                value: 1.0,
            },
            levels: LevelsParams {
                enabled: false,
                floor: 0.0,
                ceil: 1.0,
            },
        }
    }
}
