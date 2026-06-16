use bevy::prelude::*;
use bevy_planet::star_system::StarSystemPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StarSystemPlugin))
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
