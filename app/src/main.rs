use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};

use bevy_planet::{
    camera::CameraPlugin, dev_tools::DevToolsPlugin, skybox::SkyboxPlugin,
    star_system::StarSystemPlugin,
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SkyboxPlugin,
            StarSystemPlugin,
            CameraPlugin,
            DevToolsPlugin,
        ))
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run()
}
