//! The nebula's properties (`NebulaProperties`): what follows, for this system,
//! from its star and its inputs. The nebula is the cloud the system was born
//! in: its star's light clears it over the system's age (`clearing`), and its
//! metallicity sets its colours (`warmth`: a metal-poor cloud is hotter, its
//! emission bluer; a metal-rich one cooler and dustier, redder). They evolve
//! the style, which shows a cloud halfway on both (0.5).

use bevy::prelude::*;

use super::gas::{Evolution, GpuNebulaLook};
use crate::{
    star_system::star::{StarProperties, derive_star_properties},
    system::SystemParams,
};

/// At most, the share by which the voids close in an untouched cloud
/// (`clearing` 0).
const MAX_FILLING: f32 = 0.27;
/// The most gas a cloud keeps, relative to the style (`gas_left` at its
/// fullest: a cloud no star has cleared).
pub(crate) const MOST_GAS_LEFT: f32 = 1.0 + MAX_FILLING;
/// At most, the share of the filaments' thin edges evaporated in a fully
/// cleared cloud (`clearing` 1).
const MAX_EVAPORATION: f32 = 0.4;
/// The hue field's shift between a metal-poor and a metal-rich cloud, as a
/// share of the field (half towards the cool colour, half the warm one).
const HUE_SPAN: f32 = 0.15;

/// What follows from the star and the system, for display (recomputed on
/// every change; edits here are overwritten).
#[derive(Resource, Reflect, Clone, Copy)]
#[reflect(Resource)]
pub(crate) struct NebulaProperties {
    /// How far the star has cleared its cloud, from untouched (0) to as far as
    /// it goes (1), by the starlight it has poured into it: its brightness
    /// over the system's age.
    pub(crate) clearing: f32,
    /// The gas's warmth, from all cool colour (0) to all warm (1): the
    /// metallicity.
    pub(crate) warmth: f32,
}

impl Default for NebulaProperties {
    /// The style's cloud: halfway on both.
    fn default() -> Self {
        Self {
            clearing: 0.5,
            warmth: 0.5,
        }
    }
}

impl NebulaProperties {
    pub(crate) fn new(star: &StarProperties, system: &SystemParams) -> Self {
        Self {
            // The starlight poured into the cloud: as much from the star's
            // brightness as from time, and nothing yet in a newborn system.
            clearing: (star.brightness() * system.age).sqrt(),
            warmth: system.metallicity,
        }
    }

    /// The share by which the voids close (0..1): a cloud cleared less than
    /// halfway.
    pub(crate) fn filling(&self) -> f32 {
        (1.0 - 2.0 * self.clearing).max(0.0) * MAX_FILLING
    }

    /// The share of the filaments' thin edges evaporated (0..1): a cloud
    /// cleared more than halfway.
    pub(crate) fn evaporation(&self) -> f32 {
        (2.0 * self.clearing - 1.0).max(0.0) * MAX_EVAPORATION
    }

    /// How much gas is left, relative to the style (1): the globules show as
    /// much.
    pub(crate) fn gas_left(&self) -> f32 {
        1.0 + self.filling() - self.evaporation()
    }

    /// The gas's hue field's shift towards the warm colour (a share of the
    /// field; 0 at warmth 0.5, the style).
    pub(crate) fn warm_shift(&self) -> f32 {
        (self.warmth - 0.5) * HUE_SPAN
    }

    /// The gas's look: cleared, its thin edges evaporate and its filaments
    /// thin out, the dense ones remaining; less cleared, its voids close.
    pub(crate) fn look(&self) -> GpuNebulaLook {
        GpuNebulaLook::new(&Evolution {
            evaporation: self.evaporation(),
            filling: self.filling(),
            warm_shift: self.warm_shift(),
        })
    }
}

pub(crate) fn derive_nebula_properties(
    star: Res<StarProperties>,
    system: Res<SystemParams>,
    mut properties: ResMut<NebulaProperties>,
) {
    *properties = NebulaProperties::new(&star, &system);
}

pub(super) fn build(app: &mut App) {
    app.register_type::<NebulaProperties>()
        .init_resource::<NebulaProperties>()
        .add_systems(
            Update,
            derive_nebula_properties
                .run_if(
                    resource_changed::<StarProperties>.or_else(resource_changed::<SystemParams>),
                )
                .after(derive_star_properties),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::star_system::star::StarParams;

    #[test]
    fn a_cloud_halfway_keeps_the_style() {
        let halfway = NebulaProperties::default();
        assert!(halfway.filling() == 0.0 && halfway.evaporation() == 0.0);
        assert!(halfway.warm_shift() == 0.0);
    }

    #[test]
    fn a_bright_star_clears_its_nebula_a_dim_one_keeps_it() {
        let nebula = |suns, age| {
            let system = SystemParams { age, ..default() };
            NebulaProperties::new(
                &StarProperties::new(&StarParams::of_suns(suns), &system),
                &system,
            )
        };
        assert!(nebula(1.6, 0.35).clearing > nebula(0.3, 0.35).clearing);
        assert!(nebula(0.3, 0.35).filling() > 0.0);
        assert!(nebula(8.0, 0.7).evaporation() > 0.0);
    }

    #[test]
    fn metals_warm_the_gas() {
        let star = StarProperties::new(&StarParams::default(), &SystemParams::default());
        let rich = SystemParams {
            metallicity: 1.0,
            ..default()
        };
        assert!(NebulaProperties::new(&star, &rich).warm_shift() > 0.0);
    }
}
