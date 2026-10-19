// The sky: the nebula's gas, composed every frame from its baked fields
// (`nebula/gas/look.wgsl`), the Bok globules backlit by it
// (`nebula/globules/globules.wgsl`) and a per-pixel procedural starfield
// (`starfield.wgsl`). Drawn on a unit sphere re-centred on the camera in the
// vertex shader and pushed to the far plane (reverse-Z depth 0), so it sits
// behind every object, whatever the drawing order.

#import bevy_pbr::mesh_view_bindings::view
#import "shaders/common/color.wgsl"::LUMA
#import "shaders/sky/nebula/gas/look.wgsl"::{NebulaLook, cube_texel, gas_color}
#import "shaders/sky/nebula/globules/carry.wgsl"::style_point
#import "shaders/sky/starfield.wgsl"::{StarLayer, StarLook, star_layer}
#import "shaders/sky/nebula/globules/globules.wgsl"::globules
#import "shaders/sky/nebula/globules/look.wgsl"::GlobuleLook

// Mirrors `GpuSky`.
struct Sky {
    layers: array<StarLayer, 3>,
    stars: StarLook,
    // Star brightness multiplier, in cd/m^2 (scaled by the exposure).
    intensity: f32,
    enabled: u32,
    // Extinction of the stars by the displayed nebula (0 = no masking).
    occlusion: f32,
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
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var carry: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var density: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var globule_mask: texture_cube<f32>;

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
    // The nebula's gas in this direction. Its luminance acts as an optical
    // depth: stars fade as exp(-occlusion * layer.occlusion * luminance) where
    // it shows. The globules live in the style's sky (`point`) and ride it.
    let scale = sky.nebula_brightness * view.exposure;
    let gas = gas_color(cube_texel(nebula, nebula_sampler, dir), sky.nebula_look) * scale;
    let point = style_point(carry, nebula_sampler, dir);
    let optical_depth = sky.occlusion * dot(gas, LUMA);
    let dust = globules(nebula, nebula_sampler, atlas, atlas_sampler, carry, density, globule_mask, scale, dir, gas, point, pixel_angle, sky.nebula_look, sky.globule_look);

    var stars = vec3(0.0);
    if (sky.enabled != 0u) {
        let right = normalize(view.world_from_view[0].xyz);
        let up = normalize(view.world_from_view[1].xyz);
        for (var i = 0u; i < 3u; i++) {
            let layer = sky.layers[i];
            if (layer.enabled != 0u) {
                stars += star_layer(layer, sky.stars, i, dir, pixel_angle, right, up)
                    * exp(-optical_depth * layer.occlusion) * dust.transmittance;
            }
        }
    }
    return vec4(dust.color + stars * sky.intensity * view.exposure, 1.0);
}
