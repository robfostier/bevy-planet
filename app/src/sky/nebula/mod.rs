//! The nebula: its gas (`gas`, a cubemap baked on the GPU).

pub(crate) mod gas;

use bevy::prelude::*;

pub(super) fn build(app: &mut App) {
    gas::build(app);
}
