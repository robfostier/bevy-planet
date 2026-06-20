use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::prelude::*;
use bevy_planet::camera::CameraPlugin;
use bevy_planet::star_system::StarSystemPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StarSystemPlugin, CameraPlugin))
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run()
}
