//! `CAPTURE_SKY`: the whole sky (nebula and stars, no bodies) rendered as six
//! 90-degree faces, for look development on the full panorama.

use bevy::{
    camera::{Exposure, Hdr, RenderTarget, visibility::RenderLayers},
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{TextureFormat, TextureUsages},
        view::screenshot::{Screenshot, save_to_disk},
    },
};

use super::{SETTLE_TIME, WRITE_TIME, env_numbers};
use crate::sky::SKY_LAYER;

/// The LDR and HDR render targets of one sky face.
type FaceTargets = (Handle<Image>, Handle<Image>);

/// Renders the whole sky into six 90-degree faces, twice, into `CAPTURE_SKY`:
/// - `face_N.png`: through the game's tonemapping, without bloom (bloom is a
///   screen-space blur: it would seam at every face edge);
/// - `face_N.f16`: linear HDR radiance (exposure applied, no tonemapping),
///   raw little-endian RGBA16F rows.
///
/// `faces.json` records each face camera's exact basis, so the faces can be
/// reprojected (e.g. to an equirectangular panorama) without guessing any
/// cubemap convention. The default 1440 px faces have the pixel angle of a
/// 720p window at the 45-degree FOV.
pub(super) fn capture_sky(
    mut commands: Commands,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut faces: Local<Vec<FaceTargets>>,
    mut taken_at: Local<Option<f32>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(dir) = std::env::var("CAPTURE_SKY") else {
        return;
    };
    let [res] = env_numbers::<u32, 1>("CAPTURE_SKY_RES", ',').unwrap_or([1440]);
    let t = time.elapsed_secs();

    if faces.is_empty() {
        // (forward, up) of each face camera.
        let bases = [
            (Vec3::X, Vec3::Y),
            (Vec3::NEG_X, Vec3::Y),
            (Vec3::Y, Vec3::Z),
            (Vec3::NEG_Y, Vec3::NEG_Z),
            (Vec3::Z, Vec3::Y),
            (Vec3::NEG_Z, Vec3::Y),
        ];
        let mut meta = Vec::new();
        for (i, (forward, up)) in bases.into_iter().enumerate() {
            let transform = Transform::default().looking_to(forward, up);
            let (r, u, f) = (transform.right(), transform.up(), transform.forward());
            meta.push(format!(
                "{{\"face\":{i},\"right\":[{},{},{}],\"up\":[{},{},{}],\"forward\":[{},{},{}]}}",
                r.x, r.y, r.z, u.x, u.y, u.z, f.x, f.y, f.z
            ));
            let ldr = images.add(Image::new_target_texture(
                res,
                res,
                TextureFormat::Rgba8UnormSrgb,
                None,
            ));
            let mut hdr = Image::new_target_texture(res, res, TextureFormat::Rgba16Float, None);
            hdr.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            let hdr = images.add(hdr);
            for (k, (target, tonemapping)) in [
                (ldr.clone(), Tonemapping::TonyMcMapface),
                (hdr.clone(), Tonemapping::None),
            ]
            .into_iter()
            .enumerate()
            {
                commands.spawn((
                    Camera3d::default(),
                    Camera {
                        order: -20 - (i * 2 + k) as isize,
                        ..default()
                    },
                    RenderTarget::Image(target.into()),
                    Projection::Perspective(PerspectiveProjection {
                        fov: std::f32::consts::FRAC_PI_2,
                        aspect_ratio: 1.0,
                        ..default()
                    }),
                    transform,
                    RenderLayers::layer(SKY_LAYER),
                    Hdr,
                    Exposure { ev100: 15.0 },
                    tonemapping,
                ));
            }
            faces.push((ldr, hdr));
        }
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            format!("{dir}/faces.json"),
            format!("{{\"resolution\":{res},\"faces\":[{}]}}", meta.join(",")),
        );
        return;
    }

    match *taken_at {
        None if t > SETTLE_TIME => {
            for (i, (ldr, hdr)) in faces.iter().enumerate() {
                commands
                    .spawn(Screenshot::image(ldr.clone()))
                    .observe(save_to_disk(format!("{dir}/face_{i}.png")));
                let path = format!("{dir}/face_{i}.f16");
                commands.spawn(Readback::texture(hdr.clone())).observe(
                    move |event: On<ReadbackComplete>, mut commands: Commands| {
                        let _ = std::fs::write(&path, &event.data);
                        commands.entity(event.entity).despawn();
                    },
                );
            }
            *taken_at = Some(t);
        }
        Some(at) if t > at + WRITE_TIME => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
