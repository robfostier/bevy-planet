// The sky: the nebula's gas, composed every frame from its baked fields
// (`nebula/gas/look.wgsl`), and the Bok globules backlit by it
// (`nebula/globules/globules.wgsl`). Drawn on a unit sphere re-centred on the camera in
// the vertex shader and pushed to the far plane (reverse-Z depth 0), so it
// sits behind every object, whatever the drawing order.

#import bevy_pbr::mesh_view_bindings::view
#import "shaders/sky/nebula/gas/look.wgsl"::{NebulaLook, cube_texel, gas_color}
#import "shaders/sky/nebula/globules/globules.wgsl"::globules
#import "shaders/sky/nebula/globules/look.wgsl"::GlobuleLook

// Mirrors `GpuSky`.
struct Sky {
    // Display brightness of the baked nebula, in cd/m^2.
    nebula_brightness: f32,
    nebula_look: NebulaLook,
    globule_look: GlobuleLook,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sky: Sky;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var nebula: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var nebula_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var atlas: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var atlas_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var density: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var globule_mask: texture_cube<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) direction: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var clip = view.clip_from_world * vec4(view.world_position + vertex.position, 1.0);
    // Reverse-Z: depth 0 is the far plane (infinitely far).
    clip.z = 0.0;
    out.clip_position = clip;
    out.direction = vertex.position;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.direction);
    // Angular size of one pixel, from the screen-space derivatives (computed in
    // uniform control flow, before any branching).
    let pixel_angle = max(length(dpdx(dir)), length(dpdy(dir)));
    let scale = sky.nebula_brightness * view.exposure;
    let gas = gas_color(cube_texel(nebula, nebula_sampler, dir), sky.nebula_look) * scale;
    let dust = globules(nebula, nebula_sampler, atlas, atlas_sampler, density, globule_mask, scale, dir, gas, pixel_angle, sky.nebula_look, sky.globule_look);
    return vec4(dust.color, 1.0);
}
