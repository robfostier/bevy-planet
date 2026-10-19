// Nebula bake: the six faces of the nebula's fields cubemap, at startup and for
// every new seed. Reproduces the `noise` crate's Perlin / Fbm / RidgedMulti
// exactly, using the same permutation tables (uploaded from the CPU by
// `sky/nebula/gas`). The sky composes the gas from the fields every frame
// (`look.wgsl`).
// - fields: r the ridge (0..1, before its gamma), g the voids mask's noise, b
//   the hue field's noise, a the drift's noise (all 0..1).

#import "shaders/sky/nebula/gas/noise.wgsl"::{NoiseField, fbm, ridged}
#import "shaders/common/cube.wgsl"::texel_direction

struct NebulaParams {
    warp: NoiseField,
    warp_mask: NoiseField,
    ridge: NoiseField,
    mask: NoiseField,
    hue: NoiseField,
    drift: NoiseField,
    // x warp strength, y ridge attenuation
    shape: vec4<f32>,
    // Texels on a side of the fields' faces.
    resolution: u32,
    pad_a: u32,
    pad_b: u32,
    pad_c: u32,
}

@group(0) @binding(0) var fields_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(1) var<uniform> params: NebulaParams;

// FBM remapped to [0, 1].
fn unit_fbm(f: NoiseField, p: vec3<f32>) -> f32 {
    return (fbm(f, p) + 1.0) * 0.5;
}

// The warp's displacement at `point`.
fn warp_offset(point: vec3<f32>) -> vec3<f32> {
    let w = params.warp;
    let k = params.shape.x;
    let q = vec3(
        fbm(w, point + vec3(0.0, 0.0, 0.0)),
        fbm(w, point + vec3(5.2, 1.3, 7.1)),
        fbm(w, point + vec3(8.5, 2.8, 3.4)),
    );
    let r = vec3(
        fbm(w, point + vec3(0.0, 0.0, 0.0) + k * q),
        fbm(w, point + vec3(1.3, 7.1, 2.8) + k * q),
        fbm(w, point + vec3(9.2, 3.4, 5.2) + k * q),
    );
    return k * r * unit_fbm(params.warp_mask, point);
}

@compute @workgroup_size(8, 8, 1)
fn bake(@builtin(global_invocation_id) id: vec3<u32>) {
    let res = params.resolution;
    if (id.x >= res || id.y >= res) {
        return;
    }
    let direction = texel_direction(id.xy, id.z, res);
    let point = direction + warp_offset(direction);
    let fields = vec4(
        (ridged(params.ridge, params.shape.y, point) + 1.0) * 0.5,
        unit_fbm(params.mask, point),
        unit_fbm(params.hue, point),
        unit_fbm(params.drift, point),
    );
    textureStore(fields_out, vec2<i32>(id.xy), i32(id.z), fields);
}
