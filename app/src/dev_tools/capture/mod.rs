//! Image capture for look development, driven by environment variables so a
//! run can be scripted (each one exits once its capture is written):
//! - `CAPTURE_WINDOW=WxH`: resizes the window at startup;
//! - `CAPTURE_SYSTEM=seed,age,metallicity`: sets the system's inputs;
//! - `CAPTURE_CAMERA=px,py,pz,tx,ty,tz`: pins the game camera at a position,
//!   looking at a target (overrides the orbit camera);
//! - `CAPTURE_SCREENSHOT=path.png`: saves the game camera's image after a few
//!   seconds: the window, or an offscreen image of `CAPTURE_SIZE=WxH` (e.g.
//!   2560x1440, beyond the window's size; without the inspector overlay).

use bevy::{
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, save_to_disk},
    },
    transform::TransformSystems,
};

/// Time left for the GPU bake and the pipelines before capturing, in seconds.
const SETTLE_TIME: f32 = 6.0;
/// Time left to write the files before exiting, in seconds.
const WRITE_TIME: f32 = 3.0;

/// Parses an environment variable holding exactly `N` numbers.
fn env_numbers<T: std::str::FromStr, const N: usize>(key: &str, separator: char) -> Option<[T; N]> {
    let value = std::env::var(key).ok()?;
    let numbers: Vec<T> = value
        .split(separator)
        .filter_map(|s| s.parse().ok())
        .collect();
    numbers.try_into().ok()
}

fn resize_window(mut windows: Query<&mut Window>) {
    let Some([w, h]) = env_numbers::<u32, 2>("CAPTURE_WINDOW", 'x') else {
        return;
    };
    for mut window in &mut windows {
        window.resolution.set_physical_resolution(w, h);
    }
}

fn set_system(mut system: ResMut<crate::system::SystemParams>) {
    if let Some([seed, age, metallicity]) = env_numbers::<f32, 3>("CAPTURE_SYSTEM", ',') {
        system.seed = seed as u32;
        system.age = age;
        system.metallicity = metallicity;
    }
}

/// Runs after the orbit camera, so the pinned transform wins.
fn pin_camera(mut cameras: Query<&mut Transform, With<MeshPickingCamera>>) {
    let Some([px, py, pz, tx, ty, tz]) = env_numbers::<f32, 6>("CAPTURE_CAMERA", ',') else {
        return;
    };
    for mut transform in &mut cameras {
        *transform = Transform::from_xyz(px, py, pz).looking_at(Vec3::new(tx, ty, tz), Vec3::Y);
    }
}

/// With `CAPTURE_SIZE`, renders the game camera into an offscreen image of
/// that size instead of the window.
fn render_offscreen(
    mut commands: Commands,
    cameras: Query<Entity, Added<MeshPickingCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut target: Local<Option<Handle<Image>>>,
    mut offscreen: ResMut<OffscreenTarget>,
) {
    let Some([w, h]) = env_numbers::<u32, 2>("CAPTURE_SIZE", 'x') else {
        return;
    };
    for camera in &cameras {
        let image = target
            .get_or_insert_with(|| {
                images.add(Image::new_target_texture(
                    w,
                    h,
                    TextureFormat::Rgba8UnormSrgb,
                    None,
                ))
            })
            .clone();
        commands
            .entity(camera)
            .insert(RenderTarget::Image(image.clone().into()));
        offscreen.0 = Some(image);
    }
}

/// The offscreen image the game camera renders into, if any.
#[derive(Resource, Default)]
struct OffscreenTarget(Option<Handle<Image>>);

fn screenshot(
    mut commands: Commands,
    time: Res<Time>,
    offscreen: Res<OffscreenTarget>,
    mut taken_at: Local<Option<f32>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(path) = std::env::var("CAPTURE_SCREENSHOT") else {
        return;
    };
    let t = time.elapsed_secs();
    match *taken_at {
        None if t > SETTLE_TIME => {
            let screenshot = match &offscreen.0 {
                Some(image) => Screenshot::image(image.clone()),
                None => Screenshot::primary_window(),
            };
            commands.spawn(screenshot).observe(save_to_disk(path));
            *taken_at = Some(t);
        }
        Some(at) if t > at + WRITE_TIME => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<OffscreenTarget>()
        .add_systems(Startup, (resize_window, set_system))
        // After the frame's game logic, in a fixed order: the offscreen target
        // first, then the captures (which may end the run).
        .add_systems(
            PostUpdate,
            (
                pin_camera.before(TransformSystems::Propagate),
                (render_offscreen, screenshot).chain(),
            ),
        );
}
