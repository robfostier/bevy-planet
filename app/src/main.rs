use bevy::{
    camera::Hdr,
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};

use bevy_planet::{camera::CameraPlugin, skybox::SkyboxPlugin, star_system::StarSystemPlugin};

/// Spawns a dedicated 2D overlay camera for the egui inspector, so egui stops
/// piggybacking on the HDR orbit camera (where the skybox pass drew over it). It
/// renders after the orbit camera without clearing, and carries `PrimaryEguiContext`
/// so bevy_egui targets it.
fn spawn_inspector_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Name::new("Inspector Camera"),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        PrimaryEguiContext,
        // Hdr here is NOT for HDR rendering, but to make this camera compatible with the 3D camera (which is also Hdr) :
        // it forces target_format = Rgba16Float so this camera shares the same ViewTarget as the 3D camera.
        // Without it, the MSAA writeback composites egui over black.
        Hdr,
    ));
}

/// Debug-only tooling, grouped so it can be gated out of release builds in one
/// place: the diagnostic loggers, the inspector's overlay camera, and disabling
/// bevy_egui's automatic primary-context creation (which would otherwise attach a
/// second context to the 3D camera and clash with ours).
pub fn debug_plugin(app: &mut App) {
    app.add_plugins((
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        SystemInformationDiagnosticsPlugin,
    ))
    .add_systems(Startup, spawn_inspector_camera);

    // Disable auto-creation of the primary Egui context, since we spawn our own inspector camera.
    app.world_mut()
        .resource_mut::<EguiGlobalSettings>()
        .auto_create_primary_context = false;
}

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            SkyboxPlugin,
            StarSystemPlugin,
            CameraPlugin,
        ))
        .add_plugins(debug_plugin)
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run()
}
