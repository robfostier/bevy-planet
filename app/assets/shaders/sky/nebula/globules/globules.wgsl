// Bok globules: clumps of dust in front of the nebula, backlit by its gas,
// computed per screen pixel as a small node graph:
//   SCATTER    large and small globules (`scatter.wgsl`).
//   PLACEMENT  per pixel, the nebula's density: none below `gas_faint`, full
//              from `gas_full`; the gas around a globule sets its light's side.
//   SHAPE      a tile of the atlas (`atlas.wgsl`), in that frame:
//              the shape's exact signed distance, so edges and rims are a
//              fixed number of pixels whatever a globule's size.
//   DENSITY    optical depth: thin edge, opaque core (`opacity`: 0..1).
//   RIM        the edge on the light's side (`rim.wgsl`), gilded with
//              strokes of gold (`gilding.wgsl`); front-lit globules whole.
//   composite  gas * exp(-tau) + DUST * (1 - exp(-tau)) + light * rim

#import "shaders/common/gradient_noise.wgsl"::gnoise
#import "shaders/common/color.wgsl"::{LUMA, from_oklab, to_oklab}
#import "shaders/sky/nebula/globules/shape.wgsl"::EXTENT
#import "shaders/sky/nebula/gas/look.wgsl"::{NebulaLook, cube_texel, gas_color}
#import "shaders/sky/nebula/globules/carry.wgsl"::{CARRY_MARGIN, carried}
#import "shaders/sky/nebula/globules/gilding.wgsl"::gild
#import "shaders/sky/nebula/globules/look.wgsl"::GlobuleLook
#import "shaders/sky/nebula/globules/light.wgsl"::light_side
#import "shaders/sky/nebula/globules/rim.wgsl"::lit_edge
#import "shaders/sky/nebula/globules/tile.wgsl"::{TILES, distance_at}
#import "shaders/sky/nebula/globules/scatter.wgsl"::{Scatter, TIERS, cell_hash, crowded, globule_at, near_cell, globule_presence, tier}

// The dust's own dim light (scattered from the nebula all around): a dark,
// neutral brown, as a share of the gas's luminance.
const DUST: vec3<f32> = vec3(0.16, 0.12, 0.1);
// The dust's optical depth at its core (opaque); its edge is thinner.
const DENSITY: f32 = 7.0;
// Extinction per channel: neutral dust, blue stopped a little more.
const EXTINCTION: vec3<f32> = vec3(0.9, 1.0, 1.1);

// The globule a pixel lies in.
struct Hit {
    // The pixel in its frame (head radii; y away from the light).
    q: vec2<f32>,
    // Where its light comes from.
    side: vec3<f32>,
    // Its atlas tile (xy; z: 1 or -1 to mirror it), and that tile's index.
    tile: vec3<f32>,
    shape: f32,
    // Its head radius (radians).
    radius: f32,
    // Its draw for being front-lit (0..1).
    front: f32,
    // How much of it shows (0..1), with the density.
    presence: f32,
    // The pixel's point in the style's sky, carried with the globule.
    anchor: vec3<f32>,
}

struct Globules {
    // The sky seen through the dust (display units).
    color: vec3<f32>,
    // How much of the light behind (stars included) gets through.
    transmittance: f32,
}

// `gas`: the gas shown in `dir` (display units; `scale` turns the baked
// fields' units into them); `point`: the style's point shown in `dir` (the
// globules live in the style's sky and ride the gas, see `carry.wgsl`);
// `pixel_angle`: the angle one pixel spans.
fn globules(
    nebula: texture_cube<f32>,
    nebula_sampler: sampler,
    atlas: texture_2d<f32>,
    atlas_sampler: sampler,
    carry: texture_cube<f32>,
    density: texture_cube<f32>,
    mask: texture_cube<f32>,
    scale: f32,
    dir: vec3<f32>,
    gas: vec3<f32>,
    point: vec3<f32>,
    pixel_angle: f32,
    look: NebulaLook,
    g: GlobuleLook,
) -> Globules {
    // The tiers whose globules may show here (`mask.wgsl`); none on most of
    // the sky, left to the gas without searching.
    let tiers = cube_texel(mask, nebula_sampler, point);
    if (all(tiers.rgb <= vec3(0.0))) {
        return Globules(gas, 1.0);
    }
    // SCATTER (`scatter.wgsl`): tiers of sizes, few large, many small.
    let scatter = Scatter(g.count_large, g.count_small, g.size, g.size_variation, g.draw_start, g.density);
    // The nearest globule under this pixel wins (the smallest signed
    // distance); the rest of the sky is left to the gas.
    var best = 1e3;
    var hit: Hit;
    for (var t = 0; t < TIERS; t++) {
        if (tiers[t] <= 0.0) {
            continue;
        }
        let level = tier(t, scatter);
        for (var k = 0; k < 8; k++) {
            let cell = near_cell(point, level, k);
            let globule = globule_at(cell, level, scatter, max(scatter.density, 1.0));
            let center = globule.xyz;
            let radius = globule.w;
            // Near enough in the style's sky (with a margin: the gas carries
            // it), then where the gas at its centre carries it, as one piece.
            if (radius <= 0.0 || length(point - center) > CARRY_MARGIN * EXTENT * radius) {
                continue;
            }
            let shown = carried(carry, nebula_sampler, center);
            let offset = dir - shown;
            if (length(offset) > EXTENT * radius) {
                continue;
            }
            // SHAPE, in its frame (y away from the light, x across): its tile.
            let lit = light_side(density, nebula_sampler, center, radius);
            let to_light = normalize(lit - dot(lit, shown) * shown);
            let q = vec2(dot(offset, cross(shown, -to_light)), dot(offset, -to_light)) / radius;
            let h = cell_hash(cell, t, scatter);
            let shape = floor(fract(h.z * 91.3 + h.w * 17.1) * TILES * TILES);
            let tile = vec3(shape % TILES, floor(shape / TILES), select(1.0, -1.0, fract(h.y * 7.7) > 0.5));
            let d = distance_at(atlas, atlas_sampler, q, tile);
            // Not under this pixel (beyond its anti-aliased edge), or behind a
            // nearer one: skipped before the costly overlap check.
            if (d >= min(best, 0.5 * pixel_angle / radius) || crowded(globule, t, scatter)) {
                continue;
            }
            best = d;
            hit = Hit(q, to_light, tile, shape, radius, fract(h.w * 53.7 + h.x * 3.1), globule_presence(cell, level, scatter), center + offset);
        }
    }
    if (best >= 1e3) {
        return Globules(gas, 1.0);
    }

    // Everything of a globule scales with its opacity (at 0 nothing is left)
    // and, pixel by pixel, with the nebula's density behind it (whatever its
    // colour): none below `gas_faint`, full from `gas_full`.
    let placed = cube_texel(density, nebula_sampler, hit.anchor).r;
    let strength = g.opacity * smoothstep(g.gas_faint, g.gas_full, placed) * hit.presence;
    // The light: the hue of the gas behind at full saturation, as strong as
    // the brighter of that gas and the gas on the light's side.
    let beside = gas_color(cube_texel(nebula, nebula_sampler, dir + hit.side * g.size * 0.3), look) * scale;
    let lab = to_oklab(max(gas, vec3(1e-9)));
    let hue = lab.yz / max(length(lab.yz), 1e-6);
    let light = max(from_oklab(vec3(0.75, hue * 0.24)), vec3(0.0)) / 0.75
        * dot(gas, LUMA) * clamp(dot(beside, LUMA) / max(dot(gas, LUMA), 1e-9), 1.0, 2.0);
    // The silhouette, anti-aliased over a pixel from its exact distance.
    let px = pixel_angle / hit.radius;
    let edge = clamp(0.5 - best / px, 0.0, 1.0);

    // FRONT LIT: a share (`front_lit`) of the globules, drawn at random, lie
    // deep in the nebula, lit from the front: a flat of the light (exposed by
    // `front_intensity`) over the gas, no rim, no density.
    if (hit.front < g.front_lit) {
        let cover = edge * strength;
        return Globules(mix(gas, light * exp2(g.front_intensity), cover), 1.0 - cover);
    }
    // DENSITY: thin at the edge, opaque at the core, with a grain.
    let grain = gnoise(vec3(hit.q * 2.5, hit.shape)) + 0.5 * gnoise(vec3(hit.q * 5.3, hit.shape + 7.0));
    let profile = mix(0.12, 1.0, smoothstep(0.0, 0.4, -best)) * (0.6 + 0.4 * smoothstep(-0.5, 0.4, grain));
    let tau = DENSITY * edge * profile;
    // RIM: within `rim` pixels of the edge facing the light, full at the
    // edge and fading out over the rim's inner `rim_falloff` share (0: a hard
    // band; 1: edge to end). GILDING: gold strokes on a share (`gilding`) of
    // it, paler over cold gas (hue towards green-blue: Oklab a, b < 0).
    let lit_side = lit_edge(atlas, atlas_sampler, hit.q, hit.tile, px, g.rim * px);
    let toward = lit_side.x / px;
    let rim = edge * lit_side.y * clamp((1.0 - toward / g.rim) / max(g.rim_falloff, 0.01), 0.0, 1.0);
    let cold = smoothstep(0.2, -0.6, dot(hue, vec2(0.7071)));
    let gold = gild(hit.q, hit.shape, toward, g.rim, g.gilding, cold);
    let rim_light = mix(light, gold.rgb * dot(light, LUMA), gold.a);

    // Composite (see the header): the dust absorbs the gas behind
    // (`opacity`: how much the opaque core hides) and glows dimly; the rim
    // adds on top.
    let absorbed = mix(vec3(1.0), exp(-tau * EXTINCTION), strength);
    let dust = DUST * dot(gas, LUMA) * (1.0 - absorbed);
    let lit = rim_light * exp2(g.rim_intensity) * rim * strength;
    return Globules(gas * absorbed + dust + lit, absorbed.g);
}
