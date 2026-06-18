//! Continuous, per-frame input: dragging the mouse rotates the camera
//! around its target, the scroll wheel zooms. No notion of target
//! selection lives here -- see `picking` for that.

use super::{MAX_CAMERA_DISTANCE, MIN_CAMERA_DISTANCE, MOUSE_SENSITIVITY, OrbitCamera};

use crate::star_system::CelestialBody;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use std::f32::consts::FRAC_PI_4;

/// Hides and locks the cursor while the middle mouse button is held, so
/// dragging to orbit doesn't fight the OS cursor against the window edges.
pub(super) fn grab_cursor_on_mmb(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Middle) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if mouse.just_released(MouseButton::Middle) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

/// Rotates every orbit camera by the mouse delta while the middle button is
/// held. Elevation is clamped to +/- 45 degrees to avoid flipping over the
/// poles.
pub(super) fn drag_orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    mut evr_motion: MessageReader<MouseMotion>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    for ev in evr_motion.read() {
        if buttons.pressed(MouseButton::Middle) {
            for mut orbit_camera in &mut cameras {
                orbit_camera.azimuth += ev.delta.x * MOUSE_SENSITIVITY;
                orbit_camera.elevation += ev.delta.y * MOUSE_SENSITIVITY;
                orbit_camera.elevation = orbit_camera.elevation.clamp(-FRAC_PI_4, FRAC_PI_4);
            }
        }
    }
}

/// Zooms every orbit camera in or out on scroll, clamped to a distance
/// range derived from the target body's own radius so a camera never ends
/// up inside the body it orbits.
pub(super) fn zoom_orbit_camera(
    mut evr_scroll: MessageReader<MouseWheel>,
    mut cameras: Query<&mut OrbitCamera>,
    bodies: Query<&CelestialBody>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    for ev in evr_scroll.read() {
        for mut orbit_camera in &mut cameras {
            let Some(camera_target) = orbit_camera.target else {
                continue;
            };
            let Ok(body) = bodies.get(camera_target) else {
                continue;
            };

            // scroll wheel and trackpad treatment similar for now
            let delta = match ev.unit {
                MouseScrollUnit::Line => ev.y,
                MouseScrollUnit::Pixel => ev.y,
            };

            orbit_camera.radius = (orbit_camera.radius - delta).clamp(
                body.radius + MIN_CAMERA_DISTANCE,
                body.radius + MAX_CAMERA_DISTANCE,
            );
        }
    }
}
