// Reading a globule's tile of the atlas (`atlas.wgsl`): its shape's exact
// signed distance, in the globule's frame. The atlas' sizes come from the
// Rust side (`atlas/mod.rs`), as shader defs.

#import "shaders/sky/nebula/globules/shape.wgsl"::EXTENT

// The atlas: tiles of TILE texels, TILES on a side.
const TILE: f32 = f32(#{GLOBULE_TILE});
const TILES: f32 = f32(#{GLOBULE_TILES});

// A globule's signed distance at `q` (its frame, head radii; < 0 inside),
// from its atlas tile (xy: tile, z: 1 or -1 to mirror it).
fn distance_at(atlas: texture_2d<f32>, atlas_sampler: sampler, q: vec2<f32>, tile: vec3<f32>) -> f32 {
    let local = vec2(q.x * tile.z, q.y) / EXTENT * 0.5 + 0.5;
    let beyond = length(max(abs(local - 0.5) - 0.5, vec2(0.0))) * 2.0 * EXTENT;
    let uv = (tile.xy + 0.5 / TILE + clamp(local, vec2(0.0), vec2(1.0)) * (TILE - 1.0) / TILE) / TILES;
    return textureSampleLevel(atlas, atlas_sampler, uv, 0.0).r + beyond;
}
