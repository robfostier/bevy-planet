//! The sky, exposed as `SkyPlugin`: a sphere (`assets/shaders/sky/sky.wgsl`)
//! that composites its parts, each a module mirroring its shaders:
//! - `nebula`: the gas (a cubemap baked on the GPU) and the Bok globules
//!   backlit by it.
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
use nebula::{
    gas::NebulaCubemaps,
    globules::{self, GlobuleAtlas, GlobuleMask, GlobuleStyle},
};
use uniforms::GpuSky;

use crate::system::SystemParams;

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
    /// The globule atlas (shapes and their distance fields).
    #[texture(3)]
    #[sampler(4)]
    atlas: Handle<Image>,
    /// The style's density: the globules' light and placement.
    #[texture(5, dimension = "cube")]
    density: Handle<Image>,
    /// Where a globule may show (the sky searches for them only there).
    #[texture(6, dimension = "cube")]
    globule_mask: Handle<Image>,
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
        // The globule atlas' sizes, for the shaders that read it.
        descriptor
            .vertex
            .shader_defs
            .extend(globules::shader_defs());
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.extend(globules::shader_defs());
        }
        Ok(())
    }
}

#[derive(Resource)]
struct SkyHandle(Handle<SkyMaterial>);

fn spawn_sky(
    mut commands: Commands,
    cubemaps: Res<NebulaCubemaps>,
    atlas: Res<GlobuleAtlas>,
    mask: Res<GlobuleMask>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let material = materials.add(SkyMaterial {
        // Filled by `sync_sky`, on its first run.
        params: GpuSky::default(),
        nebula: cubemaps.fields.clone(),
        atlas: atlas.0.clone(),
        density: cubemaps.density.clone(),
        globule_mask: mask.0.clone(),
    });
    commands.insert_resource(SkyHandle(material.clone()));
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

fn sync_sky(
    system: Res<SystemParams>,
    handle: Res<SkyHandle>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    if let Some(mut material) = materials.get_mut(&handle.0) {
        material.params = GpuSky::new(&GlobuleStyle::default(), &system);
    }
}

/// Wires up the sky: its parts (`nebula`) and the sphere that draws them.
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        nebula::build(app);
        app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(Startup, spawn_sky)
            .add_systems(Update, sync_sky.run_if(resource_changed::<SystemParams>));
    }
}
