use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use std::f32::consts::FRAC_PI_4;

const MOUSE_SENSITIVITY: f32 = 0.01;

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    azimuth: f32,
    elevation: f32,
}

fn spawn_orbit_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        OrbitCamera {
            radius: 12.0,
            azimuth: 0.0,
            elevation: 0.0,
        },
    ));
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
    mut query: Query<&mut OrbitCamera>,
) {
    for ev in evr_motion.read() {
        if buttons.pressed(MouseButton::Middle) {
            for mut orbit_camera in &mut query {
                orbit_camera.azimuth += ev.delta.x * MOUSE_SENSITIVITY;
                orbit_camera.elevation += ev.delta.y * MOUSE_SENSITIVITY;
                orbit_camera.elevation = orbit_camera.elevation.clamp(-FRAC_PI_4, FRAC_PI_4);
            }
        }
    }
}

fn zoom_orbit_camera(
    mut evr_scroll: MessageReader<MouseWheel>,
    mut query: Query<&mut OrbitCamera>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                for mut orbit_camera in &mut query {
                    orbit_camera.radius -= ev.y;
                }
            }
            MouseScrollUnit::Pixel => {
                for mut orbit_camera in &mut query {
                    orbit_camera.radius -= ev.y;
                }
            }
        }
    }
}

fn sync_orbit_camera_transform(mut query: Query<(&mut Transform, &OrbitCamera)>) {
    for (mut transform, orbit_camera) in &mut query {
        let x = orbit_camera.radius * orbit_camera.elevation.cos() * orbit_camera.azimuth.sin();
        let y = orbit_camera.radius * orbit_camera.elevation.sin();
        let z = orbit_camera.radius * orbit_camera.elevation.cos() * orbit_camera.azimuth.cos();

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_orbit_camera).add_systems(
            Update,
            (
                grab_cursor_on_mmb,
                drag_orbit_camera,
                zoom_orbit_camera,
                sync_orbit_camera_transform,
            )
                .chain(),
        );
    }
}
