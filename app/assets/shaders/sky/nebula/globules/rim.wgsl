// A globule's lit edge, read from its atlas tile (`tile.wgsl`): how far a
// pixel lies from the edge facing the light.

#import "shaders/sky/nebula/globules/tile.wgsl"::distance_at

// How far past the lit edge (head radii) the gas must be open for the rim
// (not a hole's inner wall).
const OPEN: f32 = 0.25;
// Steps to find a pixel's distance to the edge, towards the light.
const RIM_STEPS: i32 = 6;

// From `q` inside a globule, towards the light (-y; `px`: a pixel, `width`:
// how far to look, both in head radii): x the distance to the edge that way
// (sphere-traced through the distance), y 1 where that edge exists within
// `width` and the gas past it is open (not a hole's inner wall), else 0.
fn lit_edge(atlas: texture_2d<f32>, atlas_sampler: sampler, q: vec2<f32>, tile: vec3<f32>, px: f32, width: f32) -> vec2<f32> {
    var toward = 0.0;
    var through = 0.0;
    for (var k = 0; k < RIM_STEPS; k++) {
        let d = distance_at(atlas, atlas_sampler, q - vec2(0.0, toward), tile);
        if (d >= 0.0) {
            through = 1.0;
            break;
        }
        toward += max(-d, 0.5 * px);
    }
    // Grazing an edge, the steps may not get out: then the edge is between.
    through = max(through, step(0.0, distance_at(atlas, atlas_sampler, q - vec2(0.0, width), tile)));
    let open = smoothstep(0.0, px, distance_at(atlas, atlas_sampler, q - vec2(0.0, toward + OPEN), tile));
    return vec2(toward, through * open);
}
