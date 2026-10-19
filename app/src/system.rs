//! A star system's inputs (`SystemParams`): one set per system -- a save may
//! later hold several. Everything in the system follows from them (and from the
//! player's choices, such as the star's mass), so a system is reproducible:
//! - its seed: every part derives its own from it by its path in the system
//!   (`derive`), never from a shared random stream, so adding a part never
//!   shifts another's seed;
//! - its age, set by the eras, in its star's life: the star ages, and its
//!   light erodes the nebula;
//! - its metallicity, set by the cloud it was born in: the star's composition
//!   and the nebula's dust.

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct SystemParams {
    pub(crate) seed: u32,
    /// Age, through its star's life: 0 at the star's birth, 1 at its end (a
    /// red giant); the Sun's: 0.35. Everything that ages follows it.
    #[inspector(min = 0.0, max = 1.0, speed = 0.002_f32)]
    pub(crate) age: f32,
    /// Metallicity, from metal-poor (0) to metal-rich (1); the Sun's: 2/3.
    /// A metal-poor system has hotter, brighter stars and a bluer nebula; a
    /// metal-rich one, cooler stars and a redder nebula.
    #[inspector(min = 0.0, max = 1.0, speed = 0.002_f32)]
    pub(crate) metallicity: f32,
}

/// The base system's age: the Sun's, 0.46 of the way through its main
/// sequence, itself the first 0.77 of its life.
const SUN_AGE: f32 = 0.3542;

/// The metallicity's range, in dex of [Fe/H] (0 being the Sun's): from
/// metal-poor (an old, first-generation cloud) to metal-rich.
const METAL_POOR_DEX: f32 = -1.0;
const METAL_RICH_DEX: f32 = 0.5;
/// The base system's metallicity: the Sun's, 0 dex.
const SUN_METALLICITY: f32 = (0.0 - METAL_POOR_DEX) / (METAL_RICH_DEX - METAL_POOR_DEX);

impl Default for SystemParams {
    /// The base system: the Sun's age and composition.
    fn default() -> Self {
        Self {
            seed: 0,
            age: SUN_AGE,
            metallicity: SUN_METALLICITY,
        }
    }
}

impl SystemParams {
    /// The metallicity in dex of [Fe/H] (0 = the Sun's), as the star's
    /// physics needs it.
    pub(crate) fn metallicity_dex(&self) -> f32 {
        METAL_POOR_DEX + self.metallicity * (METAL_RICH_DEX - METAL_POOR_DEX)
    }

    /// The seed of the part at `path` (e.g. `"sky/nebula/gas"`): the path's
    /// FNV-1a hash mixed with the system seed by SplitMix64.
    pub(crate) fn derive(&self, path: &str) -> u32 {
        let path_hash = path.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        });
        (splitmix64(path_hash ^ u64::from(self.seed)) >> 32) as u32
    }
}

/// SplitMix64's output function (Steele, Lea and Flood 2014).
fn splitmix64(z: u64) -> u64 {
    let z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Holds the system's inputs; add it before the procedural plugins.
pub struct SystemPlugin;

impl Plugin for SystemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SystemParams>()
            .init_resource::<SystemParams>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded(seed: u32) -> SystemParams {
        SystemParams { seed, ..default() }
    }

    #[test]
    fn same_seed_and_path_give_the_same_seed() {
        let system = SystemParams {
            seed: 42,
            ..default()
        };
        assert_eq!(
            system.derive("sky/nebula/gas"),
            system.derive("sky/nebula/gas")
        );
    }

    #[test]
    fn paths_and_systems_draw_independently() {
        let system = SystemParams::default();
        assert_ne!(
            system.derive("sky/nebula/gas"),
            system.derive("sky/starfield")
        );
        assert_ne!(
            system.derive("sky/nebula/gas"),
            seeded(1).derive("sky/nebula/gas")
        );
    }

    #[test]
    fn derived_seeds_never_change() {
        // Saves and shared codes rely on these: a change here breaks them.
        assert_eq!(seeded(0).derive("sky/nebula/gas"), 2_297_493_520);
        assert_eq!(seeded(1).derive("sky/nebula/gas"), 1_031_798_039);
        assert_eq!(seeded(0).derive("sky/starfield"), 793_696_382);
    }
}
