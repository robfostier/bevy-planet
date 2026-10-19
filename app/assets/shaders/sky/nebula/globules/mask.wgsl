// The globules' mask: a low-resolution cubemap of the style's sky, per tier
// (r, g, b: tiers 0, 1, 2), 1 wherever one of its globules may show (within
// its reach, with the carry's margin and a texel's slack; not dropped for a
// larger one; over gas dense enough to show it), 0 elsewhere. The sky reads
// it once per pixel and searches only the tiers it shows. Baked for each
// system seed, with the most gas a cloud keeps (`globules/mask`).

#import "shaders/common/cube.wgsl"::texel_direction
#import "shaders/sky/nebula/gas/look.wgsl"::cube_texel
#import "shaders/sky/nebula/globules/carry.wgsl"::CARRY_MARGIN
#import "shaders/sky/nebula/globules/scatter.wgsl"::{Scatter, TIERS, crowded, globule_at, tier}
#import "shaders/sky/nebula/globules/shape.wgsl"::EXTENT

// Mirrors `GpuMaskBake`: the scatter (see `Scatter`), the style's density
// below which no globule shows (`GlobuleLook`), and the mask's texels on a side
// of a face.
struct MaskBake {
    count_large: f32,
    count_small: f32,
    size: f32,
    variation: f32,
    draw_start: f32,
    density: f32,
    gas_faint: f32,
    resolution: u32,
}

@group(0) @binding(0) var mask_out: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> bake: MaskBake;
@group(0) @binding(2) var style_density: texture_cube<f32>;
@group(0) @binding(3) var density_sampler: sampler;

// How many texels of slack around a globule's reach: the sky reads the mask
// filtered (up to a texel away from a marked centre).
const SLACK_TEXELS: f32 = 2.0;

// Whether the gas around `point` (within `slack`, radians) may be dense enough
// to show a globule (`globules.wgsl`: none at or below `gas_faint`).
fn dense_enough(point: vec3<f32>, slack: f32) -> bool {
    let up = cross(vec3(0.0, 1.0, 0.0), point);
    let east = normalize(select(up, vec3(1.0, 0.0, 0.0), dot(up, up) < 1e-6));
    let north = cross(point, east);
    var densest = cube_texel(style_density, density_sampler, point).r;
    for (var k = 0; k < 8; k++) {
        let a = f32(k) * 0.7854;
        let o = (east * cos(a) + north * sin(a)) * slack;
        densest = max(densest, cube_texel(style_density, density_sampler, point + o).r);
    }
    return densest > bake.gas_faint;
}

// Whether a globule of tier `t` may show at `point` (`slack`: radians).
fn may_show(point: vec3<f32>, t: i32, s: Scatter, slack: f32) -> bool {
    let level = tier(t, s);
    // The 27 cells around this one: a texel's slack may reach past the 8 the
    // sky searches.
    let base = floor(point * level.cells) - 1.0;
    for (var k = 0; k < 27; k++) {
        let cell = base + vec3(f32(k % 3), f32((k / 3) % 3), f32(k / 9));
        let globule = globule_at(cell, level, s, max(s.density, 1.0));
        if (globule.w > 0.0 && length(point - globule.xyz) < CARRY_MARGIN * EXTENT * globule.w + slack && !crowded(globule, t, s)) {
            return true;
        }
    }
    return false;
}

@compute @workgroup_size(8, 8, 1)
fn bake_mask(@builtin(global_invocation_id) id: vec3<u32>) {
    let res = bake.resolution;
    if (id.x >= res || id.y >= res) {
        return;
    }
    let s = Scatter(bake.count_large, bake.count_small, bake.size, bake.variation, bake.draw_start, bake.density);
    let point = texel_direction(id.xy, id.z, res);
    // A texel spans at most 2 / res radians (at a face's centre).
    let slack = SLACK_TEXELS * 2.0 / f32(res);
    var shows = vec4(0.0);
    // Twice the slack: the density is coarser than the mask.
    if (!dense_enough(point, 2.0 * slack)) {
        textureStore(mask_out, vec2<i32>(id.xy), i32(id.z), shows);
        return;
    }
    for (var t = 0; t < TIERS; t++) {
        shows[t] = select(0.0, 1.0, may_show(point, t, s, slack));
    }
    textureStore(mask_out, vec2<i32>(id.xy), i32(id.z), shows);
}
