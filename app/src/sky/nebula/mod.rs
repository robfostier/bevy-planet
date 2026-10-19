//! The nebula: its gas (`gas`, a cubemap baked on the GPU) and the dust in it,
//! the Bok globules (`globules`, backlit by the gas), both evolved from their
//! base styles by the nebula's properties (`properties`).

pub(crate) mod gas;
pub(crate) mod globules;
pub(crate) mod properties;

use bevy::prelude::*;

pub(super) fn build(app: &mut App) {
    properties::build(app);
    gas::build(app);
    globules::build(app);
}
