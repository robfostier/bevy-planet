// Nebula bake: the six faces of the nebula's two cubemaps, at startup and
// whenever the system's seed or age changes (about 45 ms). Reproduces
// the `noise` crate's Perlin / Fbm / RidgedMulti exactly, using the same
// permutation tables (uploaded from the CPU by `sky/nebula/gas`).
//
// As the system ages, the warp's noise slides a little (`shape.z`): the gas
// flows and reshapes. The sky composes the gas from the fields every frame
// (`look.wgsl`), so its other changes cost no bake.
// - fields (on every bake): r the ridge (0..1, before its gamma), g the voids
//   mask's noise, b the hue field's noise, a the drift's noise (all 0..1);
// - carry (on every bake, the globules'): xyz where the style's gas at a
//   direction now shows, minus the direction;
// - density (only on a new seed, the globules'): r the style's density.

#import "shaders/sky/nebula/gas/noise.wgsl"::{NoiseField, fbm, ridged}
#import "shaders/sky/nebula/gas/curves.wgsl"::{contrast, levels, powf, smooth_unit}
#import "shaders/common/cube.wgsl"::texel_direction

struct NebulaParams {
    warp: NoiseField,
    warp_mask: NoiseField,
    ridge: NoiseField,
    mask: NoiseField,
    hue: NoiseField,
    drift: NoiseField,
    // x warp strength, y ridge attenuation, z how far the warp's noise has slid
    shape: vec4<f32>,
    // The style's shaping, for the globules' gas: x ridge gamma, y voids
    // floor, z voids ceil, w voids contrast.
    style: vec4<f32>,
    // Texels on a side of the fields', the carry's and the density's faces.
    resolution: u32,
    carry_resolution: u32,
    density_resolution: u32,
    // The first face of the fields this bake writes (the others follow).
    first_face: u32,
}

@group(0) @binding(0) var fields_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(1) var<uniform> params: NebulaParams;
@group(0) @binding(3) var carry_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(4) var density_out: texture_storage_2d_array<rgba16float, write>;

// The way the warp's noise slides as the system ages.
const SLIDE: vec3<f32> = vec3(0.58, 0.31, -0.75);
// The warp's octaves that carry the globules: only its broad ones, so a
// globule rides the gas's large flow as one piece, its shape barely bent
// (the finer octaves would twist it as fast as the gas's fine detail).
const CARRY_OCTAVES: u32 = 1u;

// FBM remapped to [0, 1].
fn unit_fbm(f: NoiseField, p: vec3<f32>) -> f32 {
    return (fbm(f, p) + 1.0) * 0.5;
}

// The displacement of warp `w` at `point`, its noise slid by `slide` along
// SLIDE.
fn warp_offset(w: NoiseField, point: vec3<f32>, slide: f32) -> vec3<f32> {
    let k = params.shape.x;
    let n = point + SLIDE * slide;
    let q = vec3(
        fbm(w, n + vec3(0.0, 0.0, 0.0)),
        fbm(w, n + vec3(5.2, 1.3, 7.1)),
        fbm(w, n + vec3(8.5, 2.8, 3.4)),
    );
    let r = vec3(
        fbm(w, n + vec3(0.0, 0.0, 0.0) + k * q),
        fbm(w, n + vec3(1.3, 7.1, 2.8) + k * q),
        fbm(w, n + vec3(9.2, 3.4, 5.2) + k * q),
    );
    return k * r * unit_fbm(params.warp_mask, point);
}

// The globules' carry at the style's direction `d`: where its gas now shows
// (to first order: back by how far the warp's broad octaves have moved
// there), minus `d`.
fn carry(d: vec3<f32>) -> vec3<f32> {
    var broad = params.warp;
    broad.octaves = min(broad.octaves, CARRY_OCTAVES);
    return normalize(d - (warp_offset(broad, d, params.shape.z) - warp_offset(broad, d, 0.0))) - d;
}

// The style's density at the style's direction `d` (the globules' light and
// placement: it never changes with the system's age).
fn style_density(d: vec3<f32>) -> f32 {
    let point = d + warp_offset(params.warp, d, 0.0);
    let s = params.style;
    let ridge = (ridged(params.ridge, params.shape.y, point) + 1.0) * 0.5;
    let voids = contrast(smooth_unit(levels(unit_fbm(params.mask, point), s.y, s.z)), s.w);
    return powf(ridge, s.x) * voids;
}

@compute @workgroup_size(8, 8, 1)
fn bake(@builtin(global_invocation_id) id: vec3<u32>) {
    let res = params.resolution;
    if (id.x >= res || id.y >= res) {
        return;
    }
    let face = id.z + params.first_face;
    let direction = texel_direction(id.xy, face, res);
    let point = direction + warp_offset(params.warp, direction, params.shape.z);
    let fields = vec4(
        (ridged(params.ridge, params.shape.y, point) + 1.0) * 0.5,
        unit_fbm(params.mask, point),
        unit_fbm(params.hue, point),
        unit_fbm(params.drift, point),
    );
    textureStore(fields_out, vec2<i32>(id.xy), i32(face), fields);
}

@compute @workgroup_size(8, 8, 1)
fn bake_carry(@builtin(global_invocation_id) id: vec3<u32>) {
    let res = params.carry_resolution;
    if (id.x >= res || id.y >= res) {
        return;
    }
    textureStore(carry_out, vec2<i32>(id.xy), i32(id.z), vec4(carry(texel_direction(id.xy, id.z, res)), 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn bake_density(@builtin(global_invocation_id) id: vec3<u32>) {
    let res = params.density_resolution;
    if (id.x >= res || id.y >= res) {
        return;
    }
    textureStore(density_out, vec2<i32>(id.xy), i32(id.z), vec4(style_density(texel_direction(id.xy, id.z, res))));
}
