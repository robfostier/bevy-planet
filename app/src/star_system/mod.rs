use bevy::prelude::*;

/// Handles to the key bodies of the system, published once `spawn_system` has run
/// so other plugins can reference them by entity.
#[derive(Resource)]
pub(crate) struct SystemBodies {
    /// The star at the center of the system.
    pub(crate) star: Entity,
}

/// Marks an entity as an orbital body and carries its radius.
#[derive(Component)]
#[require(Pickable)]
pub(crate) struct CelestialBody {
    /// Body radius in world units.
    pub(crate) radius: f32,
}

/// Marker for the system's central star.
#[derive(Component)]
struct Star;

/// Marker for a planet.
#[derive(Component)]
struct Planet;

/// Makes an entity revolve around the world origin in the XZ plane. This is
/// origin-relative, not parent-relative: it assumes the orbited body sits at the
/// origin, so it does not yet support moons orbiting a moving planet.
#[derive(Component)]
struct Orbital {
    /// Orbit radius in world units.
    radius: f32,
    /// Angular velocity in radians per second.
    angular_speed: f32,
    /// Starting angle in radians, to stagger bodies around their orbits.
    phase: f32,
}

/// Advances each `Orbital` body along its circular orbit around the origin.
fn update_orbital(time: Res<Time>, mut query: Query<(&mut Transform, &Orbital)>) {
    for (mut transform, orbital) in &mut query {
        let angle = orbital.angular_speed * time.elapsed_secs() + orbital.phase;
        let x = orbital.radius * angle.cos();
        let z = orbital.radius * angle.sin();
        transform.translation = Vec3::new(x, 0.0, z);
    }
}

/// Spins an entity around its own axis, independently of any orbital motion.
#[derive(Component)]
struct Spin {
    /// Rotation speed in radians per second.
    angular_speed: f32,
    /// Local axis to rotate about.
    axis: Dir3,
}

/// Rotates each `Spin` body about its own axis every frame.
fn update_spin(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>) {
    for (mut transform, spin) in &mut query {
        transform.rotate_axis(spin.axis, spin.angular_speed * time.delta_secs());
    }
}

/// Spawns the initial star system and publishes their entity handles in `SystemBodies`.
fn spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // star
    let star_radius = 2.0;
    let star_entity = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(star_radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: (Srgba::rgb(1.0, 0.95, 0.88) * 10.0).into(),
                ..default()
            })),
            CelestialBody {
                radius: star_radius,
            },
            Star,
            Spin {
                angular_speed: 0.1,
                axis: Dir3::Y,
            },
        ))
        .id();

    commands.insert_resource(SystemBodies { star: star_entity });

    commands.spawn((
        ChildOf(star_entity),
        PointLight {
            intensity: 9e7,
            range: 50.0,
            shadow_maps_enabled: true,
            ..default()
        },
    ));

    // planet
    let planet_radius = 0.25;
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(planet_radius))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.3, 0.25, 0.25).into(),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        })),
        CelestialBody {
            radius: planet_radius,
        },
        Planet,
        Orbital {
            radius: 8.6,
            angular_speed: 0.1,
            phase: 0.0,
        },
        Spin {
            angular_speed: 0.1,
            axis: Dir3::Y,
        },
    ));
}

/// Groups the star system's per-frame motion, so other plugins can order
/// themselves after the bodies have moved this frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct StarSystemSet;

/// Builds the star system and keeps its bodies moving every frame.
pub struct StarSystemPlugin;

impl Plugin for StarSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_system).add_systems(
            Update,
            (update_orbital, update_spin).chain().in_set(StarSystemSet),
        );
    }
}
