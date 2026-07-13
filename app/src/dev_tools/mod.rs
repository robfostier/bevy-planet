use bevy::{
    camera::Hdr,
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin,
    bevy_egui::{
        EguiContext, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
        egui::{CollapsingHeader, ScrollArea, Window},
    },
    bevy_inspector,
};

use crate::skybox::{
    ContrastParams, DriftParams, GammaParams, LevelsParams, MaskParams, NoiseParams, PaletteParams,
    RidgeParams, SkyboxParams, WarpParams,
};

/// Spawns a dedicated 2D overlay camera for the egui inspector, so egui stops
/// piggybacking on the HDR orbit camera (where the skybox pass drew over it). It
/// renders after the orbit camera without clearing, and carries `PrimaryEguiContext`
/// so bevy_egui targets it.
fn spawn_dev_tools_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Name::new("Dev Tools Camera"),
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

fn dev_tools_ui(world: &mut World) {
    let Ok(ctx) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };

    let mut egui_context = ctx.clone();

    Window::new("Dev Tools UI").show(egui_context.get_mut(), |ui| {
        ScrollArea::both().show(ui, |ui| {
            CollapsingHeader::new("Skybox").show(ui, |ui| {
                bevy_inspector::ui_for_resource::<SkyboxParams>(world, ui);
            });
        });
    });
}

/// Dev-only tooling, grouped so it can be gated out of release builds in one
/// place: the diagnostic loggers, the inspector's overlay camera, and disabling
/// bevy_egui's automatic primary-context creation (which would otherwise attach a
/// second context to the 3D camera and clash with ours).
pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NoiseParams>()
            .register_type::<ContrastParams>()
            .register_type::<LevelsParams>()
            .register_type::<GammaParams>()
            .register_type::<RidgeParams>()
            .register_type::<WarpParams>()
            .register_type::<MaskParams>()
            .register_type::<DriftParams>()
            .register_type::<PaletteParams>()
            .register_type::<SkyboxParams>()
            .add_plugins((
                EguiPlugin::default(),
                DefaultInspectorConfigPlugin,
                LogDiagnosticsPlugin::default(),
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                SystemInformationDiagnosticsPlugin,
            ))
            .add_systems(Startup, spawn_dev_tools_camera)
            .add_systems(EguiPrimaryContextPass, dev_tools_ui);

        // Disable auto-creation of the primary Egui context, since we spawn our own inspector camera.
        app.world_mut()
            .resource_mut::<EguiGlobalSettings>()
            .auto_create_primary_context = false;
    }
}
