use crate::star_system::{CelestialBody, SystemBodies};
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    light::cluster::ClusterConfig,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use std::f32::consts::FRAC_PI_4;

const MOUSE_SENSITIVITY: f32 = 0.01;
const MIN_CAMERA_DISTANCE: f32 = 2.0;
const MAX_CAMERA_DISTANCE: f32 = 16.0;

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    azimuth: f32,
    elevation: f32,
    target: Option<Entity>,
}

fn spawn_orbit_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        ClusterConfig::Single,
        MeshPickingCamera,
        OrbitCamera {
            radius: 9.0,
            azimuth: 0.0,
            elevation: 0.0,
            target: None,
        },
    ));
}

fn bind_orbit_camera_target(
    mut cameras: Query<&mut OrbitCamera>,
    system_bodies: Res<SystemBodies>,
) {
    for mut orbit_camera in &mut cameras {
        orbit_camera.target.get_or_insert(system_bodies.star);
    }
}

fn pick_orbit_camera_target(
    click: On<Pointer<Click>>,
    bodies: Query<&CelestialBody>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    if click.button != PointerButton::Primary {
        return;
    };

    if !bodies.contains(click.entity) {
        return;
    };

    for mut orbit_camera in &mut cameras {
        orbit_camera.target = Some(click.entity);
    }
}

fn grab_cursor_on_mmb(
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

fn drag_orbit_camera(
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

fn zoom_orbit_camera(
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

fn sync_orbit_camera_transform(
    mut cameras: Query<(&mut Transform, &OrbitCamera)>,
    bodies: Query<&Transform, (With<CelestialBody>, Without<OrbitCamera>)>,
) {
    for (mut transform, orbit_camera) in &mut cameras {
        let Some(camera_target) = orbit_camera.target else {
            continue;
        };
        let Ok(body_transform) = bodies.get(camera_target) else {
            continue;
        };

        let x = (orbit_camera.radius * orbit_camera.elevation.cos() * orbit_camera.azimuth.sin())
            + body_transform.translation.x;
        let y = (orbit_camera.radius * orbit_camera.elevation.sin()) + body_transform.translation.y;
        let z = (orbit_camera.radius * orbit_camera.elevation.cos() * orbit_camera.azimuth.cos())
            + body_transform.translation.z;

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(body_transform.translation, Vec3::Y);
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .insert_resource(MeshPickingSettings {
                require_markers: true,
                ..default()
            })
            .add_systems(Startup, spawn_orbit_camera)
            .add_observer(pick_orbit_camera_target)
            .add_systems(
                Update,
                (
                    bind_orbit_camera_target,
                    grab_cursor_on_mmb,
                    drag_orbit_camera,
                    zoom_orbit_camera,
                    sync_orbit_camera_transform,
                )
                    .chain(),
            );
    }
}
