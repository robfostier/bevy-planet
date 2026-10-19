// The globule atlas bake: TILES x TILES tiles of TILE x TILE texels, one
// globule shape per tile (`shape.wesl`, in its frame: head radii,
// [-EXTENT, EXTENT] on both axes). Each texel ends with the exact signed
// distance to its tile's silhouette, in head radii (r; < 0 inside), and the
// shape's field (g), so the sky can draw rims a fixed number of pixels wide
// whatever a globule's size.
//
// Three stages, each its own entry point:
//   shape    evaluate the field; seed the jump flood (a texel's own position
//            in r if inside, in g if outside; packed, see `pack`)
//   flood    one jump-flood step (run for steps REACH .. 1): keep, per texel,
//            the nearest inside (r) and outside (g) seed of its own tile
//   resolve  signed distance from the two nearest seeds

import package::shaders::sky::nebula::globules::shape::{EXTENT, globule_field};

// Its sizes, from the Rust side (`atlas/mod.rs`), as shader defs.
const TILE: i32 = i32(constants::GLOBULE_TILE);
const TILES: i32 = i32(constants::GLOBULE_TILES);
// How far from an edge the distances are exact, in texels; clamped beyond.
const REACH: f32 = f32(constants::GLOBULE_REACH);
const NONE: vec2<f32> = vec2(-1.0);
const NONE_PACKED: u32 = 0xffffffffu;

struct Bake {
    // Shape setting.
    buoyancy: f32,
    // The jump-flood step, in texels.
    step_: i32,
    // How far the edges' tearing noise has slid with the system's age.
    slide: f32,
    // The atlas row the slice being baked starts at (whole rows of tiles; the
    // scratch textures hold that slice alone).
    row_offset: i32,
}

// shape
@group(0) @binding(0) var field_out: texture_storage_2d<r32float, write>;
@group(0) @binding(1) var seeds_out: texture_storage_2d<rg32uint, write>;
@group(0) @binding(2) var<uniform> shape_bake: Bake;
// flood
@group(0) @binding(3) var flood_in: texture_2d<u32>;
@group(0) @binding(4) var flood_out: texture_storage_2d<rg32uint, write>;
@group(0) @binding(5) var<uniform> flood_bake: Bake;
// resolve
@group(0) @binding(6) var field_in: texture_2d<f32>;
@group(0) @binding(7) var seeds_in: texture_2d<u32>;
@group(0) @binding(8) var atlas_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(9) var<uniform> resolve_bake: Bake;

// A seed's texel position in one u32 (x low, y high: the atlas is under
// 65536 texels a side), half the traffic of two floats; NONE: none yet.
fn pack(seed: vec2<f32>) -> u32 {
    if (seed.x < 0.0) {
        return NONE_PACKED;
    }
    return u32(seed.x) | (u32(seed.y) << 16u);
}

fn unpack(packed: u32) -> vec2<f32> {
    if (packed == NONE_PACKED) {
        return NONE;
    }
    return vec2(f32(packed & 0xffffu), f32(packed >> 16u));
}

// A texel's point in its globule's frame, and its tile's seed.
fn frame(texel: vec2<i32>) -> vec3<f32> {
    let tile = texel / TILE;
    let local = (vec2<f32>(texel - tile * TILE) + 0.5) / f32(TILE);
    return vec3((local * 2.0 - 1.0) * EXTENT, f32(tile.y * TILES + tile.x) * 17.31 + 3.7);
}

@compute @workgroup_size(8, 8, 1)
fn shape(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = vec2<i32>(id.xy);
    let p = frame(texel + vec2(0, shape_bake.row_offset));
    let footprint = 2.0 * EXTENT / f32(TILE);
    let f = globule_field(p.xy, p.z, footprint, shape_bake.buoyancy, shape_bake.slide);
    textureStore(field_out, texel, vec4(f));
    let own = pack(vec2<f32>(texel));
    textureStore(seeds_out, texel, select(vec4(NONE_PACKED, own, 0u, 0u), vec4(own, NONE_PACKED, 0u, 0u), f > 0.0));
}

// The nearer of `best` and `seed` to `texel` (NONE counts as infinitely far).
fn nearer(texel: vec2<f32>, best: vec2<f32>, seed: vec2<f32>) -> vec2<f32> {
    if (seed.x < 0.0) {
        return best;
    }
    if (best.x < 0.0 || distance(texel, seed) < distance(texel, best)) {
        return seed;
    }
    return best;
}

@compute @workgroup_size(8, 8, 1)
fn flood(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = vec2<i32>(id.xy);
    let tile = texel / TILE;
    let here = vec2<f32>(texel);
    var inside = NONE;
    var outside = NONE;
    for (var i = 0; i < 9; i++) {
        let other = texel + (vec2(i % 3, i / 3) - 1) * flood_bake.step_;
        if (any(other / TILE != tile) || any(other < vec2(0))) {
            continue;
        }
        let s = textureLoad(flood_in, other, 0);
        inside = nearer(here, inside, unpack(s.x));
        outside = nearer(here, outside, unpack(s.y));
    }
    textureStore(flood_out, texel, vec4(pack(inside), pack(outside), 0u, 0u));
}

// The signed distance at `texel` from its two nearest seeds, in head radii:
// inside, minus the distance to the nearest outside texel; outside, the
// distance to the nearest inside one (half a texel: to the edge between).
// Clamped to REACH texels: past it, the flood's seeds are no longer the
// nearest, but the clamp is still no farther than the edge.
fn signed_distance(texel: vec2<i32>) -> f32 {
    let here = vec2<f32>(texel);
    let f = textureLoad(field_in, texel, 0).x;
    let packed = textureLoad(seeds_in, texel, 0);
    let inside = unpack(packed.x);
    let outside = unpack(packed.y);
    var texels = REACH;
    if (f > 0.0 && outside.x >= 0.0) {
        texels = distance(here, outside) - 0.5;
    } else if (f <= 0.0 && inside.x >= 0.0) {
        texels = distance(here, inside) - 0.5;
    }
    return select(1.0, -1.0, f > 0.0) * min(texels, REACH) * 2.0 * EXTENT / f32(TILE);
}

@compute @workgroup_size(8, 8, 1)
fn resolve(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = vec2<i32>(id.xy);
    let f = textureLoad(field_in, texel, 0).x;
    textureStore(atlas_out, texel + vec2(0, resolve_bake.row_offset), vec4(signed_distance(texel), f, 0.0, 1.0));
}
