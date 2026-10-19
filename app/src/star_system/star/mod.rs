//! The star's look: a custom surface material (`assets/shaders/star_system/star/star.wgsl`:
//! core-to-limb gradient, granulation, cracks) and an additive corona billboard
//! (`assets/shaders/star_system/star/corona.wgsl`: a soft glow). The star is set by its
//! mass (`StarParams`) and the system's age and composition (`physics`); its look derives from
//! the base style (`style`) through `appearance`.

mod appearance;
mod physics;
mod style;
mod uniforms;

use bevy::{
    camera::visibility::NoFrustumCulling,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use uniforms::{GpuCorona, GpuStar};

use super::{CelestialBody, Star, StarSystemSet};
use crate::system::SystemParams;

pub(crate) use physics::{StarParams, StarProperties};
pub(crate) use style::StarStyle;

/// Radius of the base star (one solar mass), in world units.
pub(super) const BASE_RADIUS: f32 = 2.0;
/// Intensity of the base star's light, in lumens.
pub(super) const BASE_LIGHT_INTENSITY: f32 = 9e7;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct StarMaterial {
    #[uniform(0)]
    params: GpuStar,
}

impl Material for StarMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/star_system/star/star.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/star_system/star/star.wgsl".into()
    }

    /// Emissive: never in a depth prepass (whose default vertex shader would
    /// miss the displacement) nor in a shadow map.
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct CoronaMaterial {
    #[uniform(0)]
    params: GpuCorona,
}

impl Material for CoronaMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/star_system/star/corona.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/star_system/star/corona.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// A glow: never in a depth prepass or a shadow map.
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![
            layout
                .0
                .get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?,
        ];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

impl StarStyle {
    /// The star's surface and corona materials for this style.
    pub(super) fn materials(&self, radius: f32) -> (StarMaterial, CoronaMaterial) {
        (
            StarMaterial {
                params: GpuStar::from(self),
            },
            CoronaMaterial {
                params: GpuCorona::new(self, radius),
            },
        )
    }

    /// Color of the star's light on the planets.
    pub(super) fn light_color(&self) -> Color {
        Color::linear_rgb(self.light.x, self.light.y, self.light.z)
    }
}

/// The corona: a quad that the vertex shader turns to face the camera and sizes
/// around the star's silhouette (hence no frustum culling).
pub(super) fn corona(mesh: Handle<Mesh>, material: Handle<CoronaMaterial>) -> impl Bundle {
    (
        Name::new("Corona"),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        NoFrustumCulling,
        Pickable::IGNORE,
    )
}

/// Re-derives the star's properties from the player's `StarParams` and the
/// system's inputs.
pub(crate) fn derive_star_properties(
    params: Res<StarParams>,
    system: Res<SystemParams>,
    mut properties: ResMut<StarProperties>,
) {
    *properties = StarProperties::new(&params, &system);
}

/// Applies the star's properties: its look (materials and light) and its size.
fn apply_star_look(
    properties: Res<StarProperties>,
    mut surfaces: ResMut<Assets<StarMaterial>>,
    mut coronas: ResMut<Assets<CoronaMaterial>>,
    mut stars: Query<
        (
            &mut Transform,
            &mut CelestialBody,
            &MeshMaterial3d<StarMaterial>,
        ),
        With<Star>,
    >,
    corona_handles: Query<(&MeshMaterial3d<CoronaMaterial>, &ChildOf)>,
    mut lights: Query<(&mut PointLight, &ChildOf)>,
) {
    let style = appearance::appearance(&StarStyle::default(), &properties);
    let radius = BASE_RADIUS * properties.size;
    for (mut transform, mut body, surface) in &mut stars {
        transform.scale = Vec3::splat(properties.size);
        body.radius = radius;
        if let Some(mut material) = surfaces.get_mut(&surface.0) {
            material.params = GpuStar::from(&style);
        }
    }
    for (corona, child_of) in &corona_handles {
        if stars.contains(child_of.parent())
            && let Some(mut material) = coronas.get_mut(&corona.0)
        {
            material.params = GpuCorona::new(&style, radius);
        }
    }
    for (mut light, child_of) in &mut lights {
        if stars.contains(child_of.parent()) {
            light.color = style.light_color();
            // Brighter stars light their planets more (compressed for display).
            light.intensity = BASE_LIGHT_INTENSITY * properties.luminosity.powf(0.3);
        }
    }
}

pub(super) fn build(app: &mut App) {
    app.register_type::<StarParams>()
        .register_type::<StarProperties>()
        .init_resource::<StarParams>()
        .init_resource::<StarProperties>()
        .add_plugins((
            MaterialPlugin::<StarMaterial>::default(),
            MaterialPlugin::<CoronaMaterial>::default(),
        ))
        .add_systems(
            Update,
            (
                derive_star_properties.run_if(
                    resource_changed::<StarParams>.or_else(resource_changed::<SystemParams>),
                ),
                apply_star_look
                    .run_if(resource_changed::<StarProperties>)
                    .after(derive_star_properties)
                    // Before the bodies move and the camera follows them.
                    .before(StarSystemSet),
            ),
        );
}
