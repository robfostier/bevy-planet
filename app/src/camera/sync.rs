//! Applies `OrbitCamera` state to the actual `Transform` every frame,
//! including easing the position and look-at point while a
//! `CameraTransition` is in progress.

use super::OrbitCamera;
use crate::{star_system::CelestialBody, utils::ease_in_out_sine};
use bevy::prelude::*;

/// Computes the camera's position from its spherical coordinates around its
/// target, then either applies it directly or, while a transition is in
/// progress, eases from the transition's start position/look-at towards it.
pub(super) fn sync_orbit_camera_transform(
    mut cameras: Query<(&mut Transform, &mut OrbitCamera)>,
    bodies: Query<&Transform, (With<CelestialBody>, Without<OrbitCamera>)>,
    time: Res<Time>,
) {
    for (mut transform, mut orbit_camera) in &mut cameras {
        let Some(camera_target) = orbit_camera.target else {
            continue;
        };
        let Ok(body_transform) = bodies.get(camera_target) else {
            continue;
        };

        // spherical -> cartesian, centered on the target
        let x = (orbit_camera.radius * orbit_camera.elevation.cos() * orbit_camera.azimuth.sin())
            + body_transform.translation.x;
        let y = (orbit_camera.radius * orbit_camera.elevation.sin()) + body_transform.translation.y;
        let z = (orbit_camera.radius * orbit_camera.elevation.cos() * orbit_camera.azimuth.cos())
            + body_transform.translation.z;

        let next_position = Vec3::new(x, y, z);
        let next_look_at = body_transform.translation;

        if let Some(transition) = &mut orbit_camera.transition {
            transition.timer.tick(time.delta());

            let t = ease_in_out_sine(transition.timer.fraction());

            transform.translation = Vec3::lerp(transition.start_position, next_position, t);

            transform.look_at(
                Vec3::lerp(transition.start_look_at, next_look_at, t),
                Vec3::Y,
            );

            if transition.timer.is_finished() {
                orbit_camera.transition = None;
            }
        } else {
            transform.translation = next_position;
            transform.look_at(next_look_at, Vec3::Y);
        }
    }
}
