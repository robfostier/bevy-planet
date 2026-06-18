use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

#[derive(Resource)]
pub(crate) struct SystemBodies {
    pub(crate) star: Entity,
}

#[derive(Component)]
#[require(Pickable)]
pub(crate) struct CelestialBody {
    pub(crate) radius: f32,
}

#[derive(Component)]
#[require(NoFrustumCulling)]
struct Star;

#[derive(Component)]
struct Planet;

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
    let star_radius = 1.0;
    let star_entity = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(star_radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: Color::WHITE.into(),
                ..default()
            })),
            CelestialBody {
                radius: star_radius,
            },
            Star,
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
        ))
        .id();

    commands.insert_resource(SystemBodies { star: star_entity });

    // planet
    let planet_radius = 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(planet_radius))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.0, 0.0, 1.0).into(),
            ..default()
        })),
        CelestialBody {
            radius: planet_radius,
        },
        Planet,
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
