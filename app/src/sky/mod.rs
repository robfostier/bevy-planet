//! The sky, exposed as `SkyPlugin`: a sphere (`assets/shaders/sky/sky.wgsl`)
//! that composites its parts, each a module mirroring its shaders:
//! - `nebula`: the gas (a cubemap baked on the GPU).
//!
//! Drawn as an opaque unit sphere re-centred on the camera and pushed to the far
//! plane, so it always sits behind everything.

pub(crate) mod nebula;
mod uniforms;

use bevy::{
    camera::visibility::{NoFrustumCulling, RenderLayers},
    light::NotShadowCaster,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use nebula::gas::NebulaCubemaps;
use uniforms::GpuSky;

const SHADER_ASSET_PATH: &str = "shaders/sky/sky.wgsl";

/// Display brightness of the baked nebula, in cd/m^2 (scaled by the exposure).
const NEBULA_BRIGHTNESS: f32 = 1e5;

/// Render layer seen by the sky-capture cameras (`dev_tools::capture`): the
/// sky is on it, the bodies are not.
pub(crate) const SKY_LAYER: usize = 1;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct SkyMaterial {
    #[uniform(0)]
    params: GpuSky,
    /// The nebula's baked noise fields, the gas is composed of.
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    nebula: Handle<Image>,
}

impl Material for SkyMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    /// The sky is only ever seen, never in a depth prepass or a shadow map.
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
        // Seen from inside the sphere.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

fn spawn_sky(
    mut commands: Commands,
    cubemaps: Res<NebulaCubemaps>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let material = materials.add(SkyMaterial {
        params: GpuSky::new(),
        nebula: cubemaps.fields.clone(),
    });
    commands.spawn((
        Name::new("Sky"),
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(4).unwrap())),
        MeshMaterial3d(material),
        Transform::default(),
        NoFrustumCulling,
        NotShadowCaster,
        Pickable::IGNORE,
        RenderLayers::from_layers(&[0, SKY_LAYER]),
    ));
}

/// Wires up the sky: its parts (`nebula`) and the sphere that draws them.
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        nebula::build(app);
        app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(Startup, spawn_sky);
    }
}
