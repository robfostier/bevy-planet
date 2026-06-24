use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};

use bevy_planet::{camera::CameraPlugin, star_system::StarSystemPlugin};

pub fn debug_plugin(app: &mut App) {
    app.add_plugins((
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        SystemInformationDiagnosticsPlugin,
    ));
}

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StarSystemPlugin, CameraPlugin))
        .add_plugins(debug_plugin)
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run()
}
