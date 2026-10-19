// Per-pixel procedural stars. Stars live in a 3D grid of cells laid over the
// direction sphere (one candidate star per cell, several layers at different
// cell sizes). Each pixel only checks the 8 cells nearest to it; a star is kept
// only if its projection on the sphere stays inside its own cell, which
// guarantees those 8 cells cover its footprint. Each star is a Gaussian whose
// width never drops below the pixel footprint, so sub-pixel stars stay stable
// instead of popping or aliasing.

import package::shaders::common::blackbody::blackbody;

// One star population (mirrors `GpuStarLayer`).
struct StarLayer {
    // Cells per unit length: higher = smaller cells = more, closer stars.
    scale: f32,
    // Probability that a cell holds a star.
    density: f32,
    // Peak brightness of the brightest star of the layer.
    brightness: f32,
    // Exponent of the brightness distribution: higher = fewer bright stars.
    falloff: f32,
    // Extra star radius, in radians, on top of `star_size`.
    size: f32,
    enabled: u32,
    // How much the nebula hides this layer (multiplies the global occlusion).
    occlusion: f32,
    pad_a: u32,
}

// Settings shared by all layers (mirrors `GpuStarLook`).
struct StarLook {
    // Gaussian sigma of a point star, in radians (a fixed angle).
    star_size: f32,
    // 0 = white stars, 1 = full blackbody colors.
    saturation: f32,
    // Minimum and maximum temperature, Kelvin.
    temp_min: f32,
    temp_max: f32,
    // Exponent biasing temperatures (>1 = more cool/red stars).
    temp_bias: f32,
    seed: u32,
    // Narrowest footprint, in screen pixels, a star may have before aliasing.
    min_sigma_px: f32,
    // Share of each layer's stars shown, relative to the style (1, a system
    // at mid-life; young systems more, old ones fewer).
    thinning: f32,
    // Four-point sparkles on the brightest stars: x intensity (0 = off),
    // y brightness threshold, z arm length (radians at the threshold).
    sparkle: vec4<f32>,
}

// A star's light below which it is not drawn (in star units, before the sky's
// intensity and exposure: about 4e-5 on screen, a tenth of the darkest sRGB
// step).
const GLOW_CUT: f32 = 1e-4;
// A star crossing its threshold, as the starfield ages, fades over this share
// of its own margin (its draw's distance to the threshold).
const FADE_DEPTH: f32 = 0.5;

// Jarzynski & Olano, "Hash Functions for GPU Rendering" (JCGT 2020).
fn pcg3d(v_in: vec3<u32>) -> vec3<u32> {
    var v = v_in * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v ^= v >> vec3(16u);
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return v;
}

fn to_unit(v: vec3<u32>) -> vec3<f32> {
    return vec3<f32>(v >> vec3(8u)) * (1.0 / 16777216.0);
}

// The light of one star layer in direction `dir`. `index` decorrelates the
// layers; `right`/`up` orient the sparkles on screen.
fn star_layer(
    layer: StarLayer,
    look: StarLook,
    index: u32,
    dir: vec3<f32>,
    pixel_angle: f32,
    right: vec3<f32>,
    up: vec3<f32>,
) -> vec3<f32> {
    let p = dir * layer.scale;
    let cell = floor(p);
    let f = p - cell;
    // Towards the nearer neighbour on each axis.
    let step = select(vec3(1.0), vec3(-1.0), f < vec3(0.5));

    // Intrinsic star footprint: a fixed angle, independent of the screen.
    let star_sigma2 = look.star_size * look.star_size + layer.size * layer.size;
    // Never narrower than the anti-aliasing floor of the current screen pixels;
    // when the floor widens a star, its peak drops so its energy is conserved
    // (a small window shows faint stars dimmer, like a downscaled image).
    let floor_sigma = look.min_sigma_px * pixel_angle;
    let sigma2 = max(star_sigma2, floor_sigma * floor_sigma);
    let energy = star_sigma2 / sigma2;
    let layer_seed = look.seed * 0x9E3779B9u + index * 0x85EBCA6Bu;
    // How far a star of this layer can show: its glow, until even its
    // brightest star falls below GLOW_CUT; a sparkle (only on the brightest
    // stars) reaches further, its arms fading below 1e-4 past 9.2 lengths.
    let reach2 = 2.0 * sigma2 * max(log(layer.brightness * energy / GLOW_CUT), 0.0);
    let sparkles = look.sparkle.x > 0.0 && layer.brightness > look.sparkle.y;
    let arm = 9.21 * look.sparkle.z * sqrt(layer.brightness / max(look.sparkle.y, 1e-6));
    let reach = select(sqrt(reach2), max(sqrt(reach2), arm), sparkles);
    // The lowest draw (`r.y`) of a star bright enough to sparkle: past the
    // layer's glow, only those can still show.
    let sparkle_draw = pow(look.sparkle.y / layer.brightness, 1.0 / layer.falloff);
    // A neighbour cell can only hold a star in reach if this point lies
    // within reach of it: its distance to the faces crossed into it, summed
    // in squares over the axes it is offset on (in cells).
    let to_face = min(f, 1.0 - f);
    let to_face2 = to_face * to_face;
    let reach_cells2 = reach * reach * layer.scale * layer.scale;

    var color = vec3(0.0);
    for (var i = 0u; i < 8u; i++) {
        let axes = vec3((i & 1u) != 0u, (i & 2u) != 0u, (i & 4u) != 0u);
        if (dot(select(vec3(0.0), to_face2, axes), vec3(1.0)) >= reach_cells2) {
            continue;
        }
        let offset = select(vec3(0.0), step, axes);
        let c = cell + offset;
        let h = pcg3d(bitcast<vec3<u32>>(vec3<i32>(c)) ^ vec3(layer_seed));
        let star_dir = normalize(c + to_unit(h));
        // Reject stars whose projection leaves their cell (keeps the 8-cell
        // lookup exhaustive, no clipped half-stars on cell borders).
        if (any(floor(star_dir * layer.scale) != c)) {
            continue;
        }
        let d = dir - star_dir;
        let d2 = dot(d, d);
        // Beyond the reach of the layer's brightest star: rejected before its
        // draw.
        if (d2 > reach * reach) {
            continue;
        }
        // A cell holds a star when its draw is under the layer's density,
        // scaled with the system's age (`thinning`); the stars crossing that
        // threshold fade over a range as deep as their own margin (none for
        // the style's starfield), never pop.
        let r = to_unit(pcg3d(h));
        if (d2 > reach2 && r.y <= sparkle_draw) {
            continue;
        }
        let fade = FADE_DEPTH * layer.density * abs(1.0 - look.thinning);
        let fading_out = select(0.0, 1.0, look.thinning < 1.0);
        let presence = clamp((layer.density * look.thinning - r.x) / max(fade, 1e-6) + fading_out, 0.0, 1.0);
        if (presence <= 0.0) {
            continue;
        }
        let brightness = layer.brightness * pow(r.y, layer.falloff) * presence;
        // Beyond this star's own glow (until it falls below GLOW_CUT), unless
        // it sparkles: rejected before its light.
        let sparkling = look.sparkle.x > 0.0 && brightness > look.sparkle.y;
        if (!sparkling && d2 > 2.0 * sigma2 * log(brightness * energy / GLOW_CUT)) {
            continue;
        }
        let falloff = exp(-d2 / (2.0 * sigma2));
        // Screen-aligned four-point sparkle (diffraction spikes) on the
        // brightest stars only.
        var sparkle = 0.0;
        if (sparkling) {
            let sx = dot(d, right);
            let sy = dot(d, up);
            let len = look.sparkle.z * sqrt(brightness / look.sparkle.y);
            sparkle = look.sparkle.x
                * (exp(-abs(sx) / len - sy * sy / sigma2) + exp(-abs(sy) / len - sx * sx / sigma2));
        }
        if (brightness * energy * (falloff + sparkle) < GLOW_CUT) {
            continue;
        }
        let kelvin = mix(look.temp_min, look.temp_max, pow(r.z, look.temp_bias));
        let tint = mix(vec3(1.0), blackbody(kelvin), look.saturation);
        color += tint * brightness * energy * (falloff + sparkle);
    }
    return color;
}
