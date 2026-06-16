use bevy::prelude::*;

#[derive(Component)]
struct Orbital {
    radius: f32,
    angular_speed: f32,
    phase: f32,
}

fn update_orbital(time: Res<Time>, mut query: Query<(&mut Transform, &Orbital)>) {
    for (mut transform, orbital) in &mut query {
        let angle = orbital.angular_speed * time.elapsed_secs() + orbital.phase;
        let x = orbital.radius * angle.cos();
        let z = orbital.radius * angle.sin();
        transform.translation = Vec3::new(x, 0.0, z);
    }
}

#[derive(Component)]
struct Spin {
    angular_speed: f32,
    axis: Dir3,
}

fn update_spin(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>) {
    for (mut transform, spin) in &mut query {
        transform.rotate_axis(spin.axis, spin.angular_speed * time.delta_secs());
    }
}

fn spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // star
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: Color::WHITE.into(),
            ..default()
        })),
        PointLight {
            intensity: 1e6,
            range: 10.0,
            shadows_enabled: true,
            ..default()
        },
        Spin {
            angular_speed: 1.0,
            axis: Dir3::Y,
        },
    ));

    // planet
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.0, 0.0, 1.0).into(),
            ..default()
        })),
        Orbital {
            radius: 6.0,
            angular_speed: 0.25,
            phase: 0.0,
        },
        Spin {
            angular_speed: 2.0,
            axis: Dir3::Y,
        },
    ));
}

pub struct StarSystemPlugin;

impl Plugin for StarSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_orbital, update_spin))
            .add_systems(Startup, spawn_system);
    }
}
