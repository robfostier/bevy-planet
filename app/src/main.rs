use bevy::prelude::*;
use bevy_planet::camera::CameraPlugin;
use bevy_planet::star_system::StarSystemPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StarSystemPlugin, CameraPlugin))
        .run()
}
