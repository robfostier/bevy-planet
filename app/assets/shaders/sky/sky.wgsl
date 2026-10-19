// The sky: the nebula's gas, composed every frame from its baked fields
// (`nebula/gas/look.wgsl`). Drawn on a unit sphere re-centred on the camera in
// the vertex shader and pushed to the far plane (reverse-Z depth 0), so it
// sits behind every object, whatever the drawing order.

#import bevy_pbr::mesh_view_bindings::view
#import "shaders/sky/nebula/gas/look.wgsl"::{NebulaLook, cube_texel, gas_color}

// Mirrors `GpuSky`.
struct Sky {
    // Display brightness of the baked nebula, in cd/m^2.
    nebula_brightness: f32,
    nebula_look: NebulaLook,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sky: Sky;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var nebula: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var nebula_sampler: sampler;

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
    let scale = sky.nebula_brightness * view.exposure;
    return vec4(gas_color(cube_texel(nebula, nebula_sampler, dir), sky.nebula_look) * scale, 1.0);
}
