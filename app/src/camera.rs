//! Orbit camera: lets the player orbit, zoom, and switch target among the
//! system's celestial bodies. Input handling, target picking, and the final
//! `Transform` sync each live in their own submodule; this file only holds
//! the shared state (`OrbitCamera`, `CameraTransition`), the constants they
//! depend on, and the `Plugin` that wires everything together.

mod input;
mod picking;
mod sync;

use crate::star_system::SystemBodies;
use bevy::{light::cluster::ClusterConfig, prelude::*};
use std::time::Duration;

/// Radians of orbit per pixel of mouse motion while dragging.
const MOUSE_SENSITIVITY: f32 = 0.01;
/// Maximum delay between two clicks on the same body for it to count as a
/// double-click (and thus switch the camera's target).
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);
/// How long a target switch takes to glide from the old position/orientation
/// to the new one.
const CAMERA_ANIMATION_DURATION: Duration = Duration::from_millis(1500);
/// Orbit radius the camera resets to whenever it switches target.
const DEFAULT_CAMERA_RADIUS: f32 = 4.0;
/// Closest the camera is allowed to zoom in, added to the target's own radius.
const MIN_CAMERA_DISTANCE: f32 = 2.0;
/// Furthest the camera is allowed to zoom out, added to the target's own radius.
const MAX_CAMERA_DISTANCE: f32 = 16.0;

/// In-progress glide from a previous camera position/orientation to the
/// current target's, started whenever `OrbitCamera::target` changes.
struct CameraTransition {
    /// Drives the easing factor; finished once it elapses.
    timer: Timer,
    /// Camera position captured the instant the transition started.
    start_position: Vec3,
    /// Point the camera was looking at the instant the transition started.
    start_look_at: Vec3,
}

/// A camera that orbits a target body at a given distance, azimuth and
/// elevation, expressed in spherical coordinates around the target.
#[derive(Component)]
struct OrbitCamera {
    /// Distance from the target.
    radius: f32,
    /// Horizontal angle around the target, in radians.
    azimuth: f32,
    /// Vertical angle above/below the target's equatorial plane, in radians.
    elevation: f32,
    /// Body the camera orbits and looks at; `None` only before the first
    /// frame, before `bind_orbit_camera_target` assigns the default.
    target: Option<Entity>,
    /// `Some` while the camera is gliding to a newly picked target.
    transition: Option<CameraTransition>,
}

/// Spawns the single orbit camera, with no target yet (assigned next frame
/// by `bind_orbit_camera_target`).
fn spawn_orbit_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        ClusterConfig::Single,
        MeshPickingCamera,
        OrbitCamera {
            radius: DEFAULT_CAMERA_RADIUS,
            azimuth: 0.0,
            elevation: 0.0,
            target: None,
            transition: None,
        },
    ));
}

/// Gives every targetless orbit camera a default target (the system's
/// star), once `SystemBodies` exists.
fn bind_orbit_camera_target(
    mut cameras: Query<&mut OrbitCamera>,
    system_bodies: Res<SystemBodies>,
) {
    for mut orbit_camera in &mut cameras {
        orbit_camera.target.get_or_insert(system_bodies.star);
    }
}

/// Registers the orbit camera: spawn, mesh picking (for click-to-select),
/// input handling, and the system that syncs `OrbitCamera` state to the
/// actual `Transform` every frame.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .insert_resource(MeshPickingSettings {
                require_markers: true,
                ..default()
            })
            .add_systems(Startup, spawn_orbit_camera)
            .add_observer(picking::pick_orbit_camera_target)
            .add_systems(
                Update,
                (
                    bind_orbit_camera_target,
                    input::grab_cursor_on_mmb,
                    input::drag_orbit_camera,
                    input::zoom_orbit_camera,
                    sync::sync_orbit_camera_transform,
                )
                    .chain(),
            );
    }
}
