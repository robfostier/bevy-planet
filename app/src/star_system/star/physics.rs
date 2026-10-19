//! A star's physics. Its structure is set by its mass, its composition and its
//! age (the Vogt-Russell theorem): the player picks the mass; the age and the
//! composition are the system's (`SystemParams`). Simple fits give its
//! temperature, luminosity and size; at the defaults
//! (one solar mass, the base system: the Sun's age and composition) they are
//! exactly the Sun's, i.e. the base style (`appearance` turns them into a look).

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

use crate::system::SystemParams;

/// The Sun's effective temperature, in Kelvin: the base style's temperature.
pub(super) const SUN_TEMPERATURE: f32 = 5778.0;
/// The share of a star's life spent on its main sequence; the rest of it, the
/// star is a red giant (compressed for display: really about a tenth).
const GIANT_ONSET: f32 = 0.77;
/// Effective temperature of a red giant, in Kelvin.
const GIANT_TEMPERATURE: f32 = 3900.0;

/// The lightest and heaviest stars the player can pick, in solar masses: a red
/// dwarf and a blue giant.
const LIGHTEST: f32 = 0.1;
const HEAVIEST: f32 = 20.0;

/// The star's own input (its age and composition are the system's).
#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct StarParams {
    /// Mass, from the lightest star (0, a red dwarf) to the heaviest (1, a
    /// blue giant), on a log scale; the Sun's: 0.43. The player's choice.
    #[inspector(min = 0.0, max = 1.0, speed = 0.002)]
    pub(crate) mass: f32,
}

impl Default for StarParams {
    /// The Sun.
    fn default() -> Self {
        Self::of_suns(1.0)
    }
}

impl StarParams {
    /// A star of `suns` solar masses.
    pub(crate) fn of_suns(suns: f32) -> Self {
        Self {
            mass: (suns / LIGHTEST).ln() / (HEAVIEST / LIGHTEST).ln(),
        }
    }

    /// The mass in solar masses, as the fits need it.
    fn suns(&self) -> f32 {
        LIGHTEST * (HEAVIEST / LIGHTEST).powf(self.mass)
    }
}

/// What follows from the inputs, for display (recomputed on every change;
/// edits here are overwritten).
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub(crate) struct StarProperties {
    /// Effective temperature, in Kelvin.
    pub(crate) temperature: f32,
    /// Luminosity, in solar luminosities.
    pub(crate) luminosity: f32,
    /// Displayed size relative to the base star (compressed, diorama scale).
    pub(crate) size: f32,
}

/// A star's luminosity on the main sequence, at the Sun's age, in Suns.
fn main_sequence_luminosity(mass: f32) -> f32 {
    if mass < 0.43 {
        0.23 * mass.powf(2.3)
    } else {
        mass.powf(3.5)
    }
}

impl StarProperties {
    /// How bright the star is, from the faintest star the player can pick (0)
    /// to the brightest (1), on a log scale.
    pub(crate) fn brightness(&self) -> f32 {
        let (faintest, brightest) = (
            main_sequence_luminosity(LIGHTEST).ln(),
            main_sequence_luminosity(HEAVIEST).ln(),
        );
        ((self.luminosity.ln() - faintest) / (brightest - faintest)).clamp(0.0, 1.0)
    }

    pub(crate) fn new(params: &StarParams, system: &SystemParams) -> Self {
        let (mass, z) = (params.suns(), system.metallicity_dex());
        // How far along its main sequence the star is (1: its core runs out),
        // and the Sun's, the base.
        let ms = (system.age / GIANT_ONSET).min(1.0);
        let sun_ms = SystemParams::default().age / GIANT_ONSET;
        // On the main sequence, at the Sun's age.
        let mut luminosity = main_sequence_luminosity(mass);
        let mut temperature = main_sequence_temperature(mass);
        // Real radii grow as M^0.8; compressed so a giant fits the orrery.
        let mut size = mass.powf(0.35);

        // Composition: fewer metals = a more transparent, hotter, brighter star.
        temperature *= 10f32.powf(-0.04 * z);
        luminosity *= 10f32.powf(-0.15 * z);

        // Main-sequence ageing (normalised to the Sun's): the star slowly
        // brightens, warms a little and grows (the Sun: +30% since birth).
        let brighten = |f: f32| 0.7 + 0.4 * f + f * f;
        luminosity *= brighten(ms) / brighten(sun_ms);
        temperature *= (0.97 + 0.065 * ms) / (0.97 + 0.065 * sun_ms);
        let grow = |f: f32| 0.9 + 0.22 * f + 0.25 * f * f;
        size *= (grow(ms) / grow(sun_ms)).sqrt();

        // Past the main sequence: the core contracts, the envelope swells into
        // a red giant -- cool, huge, very luminous (compressed for display).
        let giant = ((system.age - GIANT_ONSET) / (1.0 - GIANT_ONSET)).clamp(0.0, 1.0);
        let giant = giant * giant * (3.0 - 2.0 * giant);
        temperature += (GIANT_TEMPERATURE - temperature) * giant;
        luminosity *= 10f32.powf(3.0 * giant);
        size *= 1.0 + 1.5 * giant;

        Self {
            temperature,
            luminosity,
            size,
        }
    }
}

/// Effective temperature on the main sequence, interpolated in log-mass between
/// reference stars (M dwarf to O star).
fn main_sequence_temperature(mass: f32) -> f32 {
    const TABLE: [(f32, f32); 8] = [
        (0.1, 2900.0),
        (0.5, 3800.0),
        (0.8, 5000.0),
        (1.0, SUN_TEMPERATURE),
        (2.0, 9000.0),
        (5.0, 15000.0),
        (10.0, 25000.0),
        (20.0, 35000.0),
    ];
    let m = mass.clamp(TABLE[0].0, TABLE[TABLE.len() - 1].0);
    for pair in TABLE.windows(2) {
        let ((m0, t0), (m1, t1)) = (pair[0], pair[1]);
        if m <= m1 {
            let k = (m.ln() - m0.ln()) / (m1.ln() - m0.ln());
            return t0 + (t1 - t0) * k;
        }
    }
    TABLE[TABLE.len() - 1].1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() <= 1e-4 * b.abs().max(1.0)
    }

    #[test]
    fn defaults_are_the_sun() {
        let sun = StarProperties::new(&StarParams::default(), &SystemParams::default());
        assert!(close(sun.temperature, SUN_TEMPERATURE));
        assert!(close(sun.luminosity, 1.0));
        assert!(close(sun.size, 1.0));
    }

    #[test]
    fn heavier_stars_burn_hotter_and_brighter() {
        // In a young system: the heavy star is still on its main sequence.
        let young = SystemParams {
            age: 0.05,
            ..default()
        };
        let sun = StarProperties::new(&StarParams::default(), &young);
        let heavy = StarProperties::new(&StarParams::of_suns(5.0), &young);
        assert!(heavy.temperature > sun.temperature);
        assert!(heavy.luminosity > sun.luminosity);
    }

    #[test]
    fn an_old_star_becomes_a_red_giant() {
        let old = SystemParams {
            age: 1.0,
            ..default()
        };
        let giant = StarProperties::new(&StarParams::default(), &old);
        assert!(close(giant.temperature, GIANT_TEMPERATURE));
        assert!(giant.luminosity > 500.0);
        assert!(giant.size > 2.0);
    }

    #[test]
    fn main_sequence_temperature_hits_its_reference_stars() {
        assert!(close(main_sequence_temperature(1.0), SUN_TEMPERATURE));
        assert!(close(main_sequence_temperature(5.0), 15000.0));
        // Clamped beyond the table.
        assert!(close(main_sequence_temperature(0.01), 2900.0));
        assert!(close(main_sequence_temperature(50.0), 35000.0));
    }
}
