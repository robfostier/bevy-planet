//! Click-to-select: double-clicking a celestial body switches the orbit
//! camera's target to it and starts a `CameraTransition` to glide there.

use super::{
    CAMERA_ANIMATION_DURATION, CameraTransition, DEFAULT_CAMERA_RADIUS, DOUBLE_CLICK_WINDOW,
    OrbitCamera,
};
use crate::star_system::CelestialBody;
use bevy::prelude::*;
use std::time::Duration;

/// Observer on `Pointer<Click>`: detects a double-click on a celestial body
/// and switches every orbit camera's target to it.
///
/// Double-click detection is a small debounce state machine kept in a
/// `Local`: the first click on a body is just remembered (entity + time);
/// if a second click lands on the *same* body within `DOUBLE_CLICK_WINDOW`,
/// the target switches. Any click on a different body, on empty space, or
/// with a button other than the primary one resets the state instead of
/// counting towards a double-click.
pub(super) fn pick_orbit_camera_target(
    mut click: On<Pointer<Click>>,
    bodies: Query<(&CelestialBody, &Transform)>,
    mut cameras: Query<(&mut OrbitCamera, &Transform)>,
    mut state: Local<Option<(Entity, Duration)>>,
    time: Res<Time<Real>>,
) {
    // avoid a second trigger on the window (click bubbles by default)
    click.propagate(false);

    if click.button != PointerButton::Primary || !bodies.contains(click.entity) {
        *state = None;
        return;
    };

    let Some((last_entity, last_click_time)) = *state else {
        *state = Some((click.entity, time.elapsed()));
        return;
    };

    if last_entity == click.entity && time.elapsed() - last_click_time <= DOUBLE_CLICK_WINDOW {
        *state = None;

        for (mut orbit_camera, transform) in &mut cameras {
            // capture where the camera currently is/looks before retargeting,
            // so sync_orbit_camera_transform can glide from there
            if let Some(current_target) = orbit_camera.target
                && let Ok((_, current_target_transform)) = bodies.get(current_target)
            {
                orbit_camera.transition = Some(CameraTransition {
                    timer: Timer::from_seconds(
                        CAMERA_ANIMATION_DURATION.as_secs_f32(),
                        TimerMode::Once,
                    ),
                    start_position: transform.translation,
                    start_look_at: current_target_transform.translation,
                });
            };

            orbit_camera.radius = DEFAULT_CAMERA_RADIUS;
            orbit_camera.target = Some(click.entity);
        }
    } else {
        *state = Some((click.entity, time.elapsed()));
    }
}
