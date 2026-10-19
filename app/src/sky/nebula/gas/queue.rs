//! When to bake the nebula's cubemaps. As the age changes (a slider being
//! dragged), a low-resolution preview of the fields and the carry, every
//! frame; once it settles, the full fields, a face per frame, the preview shown
//! until the last. A new seed goes the same way, its density baked with its
//! first preview.

use std::sync::atomic::Ordering;

use bevy::prelude::*;

use super::{
    CARRY_RESOLUTION, DENSITY_RESOLUTION, Dispatches, NebulaBakeJob, NebulaBakedGeneration,
    NebulaCubemaps, NebulaStyle, PREVIEW_RESOLUTION, RESOLUTION, uniforms::GpuNebulaParams,
    uniforms::build_gpu_params,
};
use crate::system::SystemParams;

/// The gas's path in the system, from which its seed derives.
const SEED_PATH: &str = "sky/nebula/gas";
/// How far the warp's noise slides over a whole life (noise units; the style
/// at mid-life): the gas flows and reshapes a little as the system ages.
const SLIDE_SPAN: f32 = 0.15;
/// Frames the age must stay put before the full fields replace a preview.
const SETTLE_FRAMES: u32 = 6;
/// A cubemap's faces.
const FACES: u32 = 6;

/// Where the fields stand against the system's age.
#[derive(Clone, Copy)]
enum Phase {
    /// A preview shown: the frames since the age last changed.
    Previewing(u32),
    /// The age settled: the next full face to bake.
    Slicing(u32),
    Done,
}

/// The bake's uniforms and tables for a seed, kept: only the slide changes as
/// the system ages.
pub(super) struct Baked {
    seed: u32,
    age: f32,
    params: GpuNebulaParams,
    tables: Vec<u32>,
    phase: Phase,
}

/// What a bake writes.
enum Bake {
    /// The fields' preview and the carry (and the density, for a new seed).
    Preview { density: bool },
    /// One face of the full fields.
    Face(u32),
}

pub(super) fn queue_bake(
    mut commands: Commands,
    system: Res<SystemParams>,
    mut cubemaps: ResMut<NebulaCubemaps>,
    done: Res<NebulaBakedGeneration>,
    mut baked: Local<Option<Baked>>,
    mut generation: Local<u64>,
) {
    if done.0.load(Ordering::Relaxed) < *generation {
        return;
    }
    let bake = match baked.as_mut() {
        Some(b) if b.seed == system.seed && b.age != system.age => {
            b.age = system.age;
            b.phase = Phase::Previewing(0);
            Bake::Preview { density: false }
        }
        Some(b) if b.seed == system.seed => match b.phase {
            Phase::Previewing(frames) if frames + 1 < SETTLE_FRAMES => {
                b.phase = Phase::Previewing(frames + 1);
                return;
            }
            Phase::Previewing(_) => {
                b.phase = Phase::Slicing(1);
                Bake::Face(0)
            }
            Phase::Slicing(face) => {
                b.phase = if face + 1 < FACES {
                    Phase::Slicing(face + 1)
                } else {
                    Phase::Done
                };
                Bake::Face(face)
            }
            Phase::Done => return,
        },
        _ => {
            let (params, tables) = build_gpu_params(
                &NebulaStyle::default(),
                system.derive(SEED_PATH),
                [RESOLUTION, CARRY_RESOLUTION, DENSITY_RESOLUTION],
            );
            *baked = Some(Baked {
                seed: system.seed,
                age: system.age,
                params,
                tables,
                // Its faces follow right after its preview.
                phase: Phase::Previewing(SETTLE_FRAMES),
            });
            Bake::Preview { density: true }
        }
    };
    let Some(b) = baked.as_ref() else { return };
    let (fields, resolution, faces, carry, density) = match bake {
        Bake::Preview { density } => {
            let preview = cubemaps.preview.clone();
            (preview, PREVIEW_RESOLUTION, [0, FACES], true, density)
        }
        Bake::Face(face) => (cubemaps.fields.clone(), RESOLUTION, [face, 1], false, false),
    };
    *generation += 1;
    commands.insert_resource(NebulaBakeJob {
        params: b
            .params
            .slid((system.age - 0.5) * SLIDE_SPAN)
            .at_resolution(resolution)
            .starting_at_face(faces[0]),
        tables: b.tables.clone(),
        generation: *generation,
        images: [
            fields.clone(),
            cubemaps.carry.clone(),
            cubemaps.density.clone(),
        ],
        dispatches: Dispatches {
            resolutions: [resolution, CARRY_RESOLUTION, DENSITY_RESOLUTION],
            faces,
            carry,
            density,
        },
    });
    // The sky reads the preview, or the full fields once their last face is
    // baked.
    let shown = match bake {
        Bake::Face(face) if face + 1 < FACES => return,
        _ => fields,
    };
    if cubemaps.shown != shown {
        cubemaps.shown = shown;
    }
}
