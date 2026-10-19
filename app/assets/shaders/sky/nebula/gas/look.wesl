// The gas, composed every frame from the baked fields (`bake.wesl`) with its
// look (`NebulaLook`, mirrors `GpuNebulaLook`): the style's shaping and
// palette, evolved by the nebula's properties. So the system's evolution costs
// no bake: only these few sums per pixel.

import package::shaders::common::color::{from_oklab, rotate_hue};
import package::shaders::sky::nebula::gas::curves::{contrast, levels, powf, smooth_unit};

struct NebulaLook {
    // The palette's two colours, in Oklab (w unused).
    primary: vec4<f32>,
    secondary: vec4<f32>,
    gamma: f32,
    evaporation: f32,
    mask_floor: f32,
    mask_ceil: f32,
    mask_contrast: f32,
    hue_floor: f32,
    hue_ceil: f32,
    hue_contrast: f32,
    drift_degrees: f32,
    drift_center: f32,
    saturation: f32,
    intensity: f32,
}

// The fields' texel in a direction (cubemaps are left-handed).
fn cube_texel(cube: texture_cube<f32>, cube_sampler: sampler, dir: vec3<f32>) -> vec4<f32> {
    return textureSampleLevel(cube, cube_sampler, normalize(dir) * vec3(1.0, 1.0, -1.0), 0.0);
}

// The voids mask from its noise (levels from `floor`, smoothed, contrast).
fn voids(noise: f32, floor: f32, look: NebulaLook) -> f32 {
    return contrast(smooth_unit(levels(noise, floor, look.mask_ceil)), look.mask_contrast);
}

// The gas's density: the filaments (their thin edges evaporated: the ridge's
// low end cut, then stretched back to full) carved by the voids.
fn gas_density(fields: vec4<f32>, look: NebulaLook) -> f32 {
    let ridge = max(fields.r - look.evaporation, 0.0) / (1.0 - look.evaporation);
    return powf(ridge, look.gamma) * voids(fields.g, look.mask_floor, look);
}

// The gas's colour (baked units): the palette's two colours, their hue
// rotated by the drift, mixed by the hue field in Oklab (an RGB mix passes
// through a muddy grey), its chroma scaled by the saturation, times the
// density.
fn gas_color(fields: vec4<f32>, look: NebulaLook) -> vec3<f32> {
    let hue = contrast(levels(fields.b, look.hue_floor, look.hue_ceil), look.hue_contrast);
    let shift = (fields.a - (0.5 - look.drift_center * 0.5)) * look.drift_degrees;
    // Both colours rotate alike, so the mix rotates once.
    let lab = rotate_hue(mix(look.secondary.xyz, look.primary.xyz, hue), shift);
    let color = max(from_oklab(vec3(lab.x, lab.yz * look.saturation)), vec3(0.0));
    return color * gas_density(fields, look) * look.intensity;
}
