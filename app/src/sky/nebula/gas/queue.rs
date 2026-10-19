//! When to bake the nebula's fields: at startup, and for every new seed.

use bevy::prelude::*;

use super::{NebulaBakeJob, NebulaCubemaps, NebulaStyle, RESOLUTION, uniforms::build_gpu_params};
use crate::system::SystemParams;

/// The gas's path in the system, from which its seed derives.
const SEED_PATH: &str = "sky/nebula/gas";

pub(super) fn queue_bake(
    mut commands: Commands,
    system: Res<SystemParams>,
    cubemaps: Res<NebulaCubemaps>,
    mut baked_seed: Local<Option<u32>>,
    mut generation: Local<u64>,
) {
    if *baked_seed == Some(system.seed) {
        return;
    }
    *baked_seed = Some(system.seed);
    let (params, tables) = build_gpu_params(
        &NebulaStyle::default(),
        system.derive(SEED_PATH),
        RESOLUTION,
    );
    *generation += 1;
    commands.insert_resource(NebulaBakeJob {
        params,
        tables,
        generation: *generation,
        image: cubemaps.fields.clone(),
    });
}
