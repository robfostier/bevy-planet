//! The nebula: its gas (`gas`, a cubemap baked on the GPU) and the dust in it,
//! the Bok globules (`globules`, backlit by the gas).

pub(crate) mod gas;
pub(crate) mod globules;

use bevy::prelude::*;

pub(super) fn build(app: &mut App) {
    gas::build(app);
    globules::build(app);
}
