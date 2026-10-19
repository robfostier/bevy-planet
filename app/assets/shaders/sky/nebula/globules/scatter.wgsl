// The globules' scatter: TIERS tiers of sizes, from `count_large` large
// globules (tier 0, `size`) to `count_small` small ones (the last tier,
// `size * (1 - size_variation)`); the tiers between are in between, in size
// and count (geometric steps). Each tier is a grid laid over the direction
// sphere, one candidate per cell; a globule stays inside its own cell (so two
// of a tier never overlap), and a smaller one is dropped where a larger one
// already stands.

#import "shaders/sky/nebula/globules/shape.wgsl"::EXTENT

const TIERS: i32 = 3;
// A cell's width, in a tier's globule reaches (above 2: a globule never
// leaves its cell, so the 8 cells nearest to a point cover it).
const CELL_SPAN: f32 = 2.4;
// How much of their reach two globules may not share (EXTENT is generous:
// their silhouettes rarely fill it).
const KEEP_CLEAR: f32 = 0.7;
// A globule crossing its threshold, as the density changes, fades over this
// share of its own margin (its draw's distance to the threshold).
const FADE_DEPTH: f32 = 0.5;

// The scatter's settings.
struct Scatter {
    count_large: f32,
    count_small: f32,
    // Head radius of the largest globules (radians).
    size: f32,
    variation: f32,
    // Where the draws start (0..1000), from the seed: another start draws
    // another sky of globules (placement, shapes, light).
    draw_start: f32,
    // The gas left, relative to the base (1): scales how many globules show.
    density: f32,
}

fn hash4(cell: vec3<f32>) -> vec4<f32> {
    let q = vec4(
        dot(cell, vec3(127.1, 311.7, 74.7)),
        dot(cell, vec3(269.5, 183.3, 246.1)),
        dot(cell, vec3(113.5, 271.9, 124.6)),
        dot(cell, vec3(301.3, 97.1, 199.9)),
    );
    return fract(sin(q) * 43758.5453);
}

// Where a tier's draws start.
fn draw_offset(t: i32, s: Scatter) -> f32 {
    return f32(t) * 1000.123 + s.draw_start;
}

// The random draw of tier `t`'s `cell` (four numbers in 0..1).
fn cell_hash(cell: vec3<f32>, t: i32, s: Scatter) -> vec4<f32> {
    return hash4(cell + draw_offset(t, s));
}

// Its last number alone (`cell_hash(..).w`, a quarter of the cost): whether
// the cell holds a globule, checked first since most cells are empty.
fn cell_draw(cell: vec3<f32>, t: i32, s: Scatter) -> f32 {
    return fract(sin(dot(cell + draw_offset(t, s), vec3(301.3, 97.1, 199.9))) * 43758.5453);
}

// One tier's constants (the same for all its cells: computed once per pixel).
struct Tier {
    index: i32,
    // Head radius of its globules.
    size: f32,
    // Cells per unit of the direction sphere.
    cells: f32,
    // Probability that one of its cells holds a globule.
    chance: f32,
}

// Tier `t`: its place between the largest (0) and the smallest (1) sets its
// size and its count (geometric steps).
fn tier(t: i32, s: Scatter) -> Tier {
    let place = f32(t) / f32(TIERS - 1);
    let size = s.size * pow(max(1.0 - s.variation, 0.05), place);
    let cells = 1.0 / (CELL_SPAN * EXTENT * size);
    let count = exp(mix(log(max(s.count_large, 1e-3)), log(max(s.count_small, 1e-3)), place));
    // The cells the sky passes through number about 4 pi cells^2.
    return Tier(t, size, cells, count / (12.566 * cells * cells));
}

// The 8 cells of a tier nearest to `dir`: the `k`-th of them.
fn near_cell(dir: vec3<f32>, level: Tier, k: i32) -> vec3<f32> {
    return floor(dir * level.cells - 0.5) + vec3(f32(k & 1), f32((k >> 1) & 1), f32(k >> 2));
}

// The globule of a tier in `cell`, if any: its centre (xyz, on the sphere)
// and head radius (w; 0 when the cell is empty). `reach`: how far past the
// base share of occupied cells to look (the density, above 1).
fn globule_at(cell: vec3<f32>, level: Tier, s: Scatter, reach: f32) -> vec4<f32> {
    if (cell_draw(cell, level.index, s) > level.chance * reach) {
        return vec4(0.0);
    }
    let h = cell_hash(cell, level.index, s);
    let radius = level.size * mix(0.8, 1.0, fract(h.x * 31.7 + h.y * 5.3));
    let jitter = 1.0 - 2.0 * (radius / level.size) / CELL_SPAN;
    let drawn = (cell + 0.5 + (fract(h.xyz * 7.13 + 0.31) - 0.5) * jitter) / level.cells;
    if (abs(length(drawn) - 1.0) > 0.5 / level.cells) {
        return vec4(0.0);
    }
    return vec4(normalize(drawn), radius);
}

// How much of the globule in `cell` shows (0..1): all of it for the base sky;
// as the density changes, the globules whose draw nears its threshold fade in
// or out over a range as deep as their own margin, so none ever pops.
fn globule_presence(cell: vec3<f32>, level: Tier, s: Scatter) -> f32 {
    let draw = cell_draw(cell, level.index, s);
    let fade = FADE_DEPTH * level.chance * abs(1.0 - s.density);
    let fading_out = select(0.0, 1.0, s.density < 1.0);
    return clamp((level.chance * s.density - draw) / max(fade, 1e-9) + fading_out, 0.0, 1.0);
}

// Whether a larger tier already has a globule where tier `t`'s globule
// `candidate` (centre, radius) would stand.
fn crowded(candidate: vec4<f32>, t: i32, s: Scatter) -> bool {
    for (var u = 0; u < t; u++) {
        let level = tier(u, s);
        for (var k = 0; k < 8; k++) {
            // The base sky's globules: the crowding never changes with the
            // density, so no globule pops out when another fades in.
            let other = globule_at(near_cell(candidate.xyz, level, k), level, s, 1.0);
            if (other.w > 0.0 && length(candidate.xyz - other.xyz) < KEEP_CLEAR * EXTENT * (candidate.w + other.w)) {
                return true;
            }
        }
    }
    return false;
}
