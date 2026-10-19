// The globule generator: one unique silhouette per seed, in the globule's own
// frame (units of its head radius; x across, y away from its light, so tails
// trail to +y). It returns a field: > 0 inside the dust, 0 on its edge.
//
// - Family (drawn per seed): compact (a body ringed with lobes), comet (a head
//   trailing a tapering, bending tail), cluster (loose knots) or filament (a
//   bent chain); then, for all, sub-lobes on its outline and torn-off knots
//   around it. Its shape is the ENVELOPE, a soft silhouette.
// - Its edge band is torn by an fBm noise (own frequency and strength): lobes,
//   notches and torn-off fragments, while the core stays solid.
// - Round bumps at three scales plump up the lobes' tips (`buoyancy`).

#import "shaders/common/gradient_noise.wgsl"::gnoise

// How far a globule reaches from its centre, in head radii (its tail included).
const EXTENT: f32 = 3.5;

fn hash(seed: f32, k: f32) -> f32 {
    return fract(sin(seed * 12.9898 + k * 78.233) * 43758.5453);
}

// Smooth union of two signed distances (negative inside).
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn capsule(q: vec2<f32>, a: vec2<f32>, b: vec2<f32>, ra: f32, rb: f32) -> f32 {
    let ab = b - a;
    let t = clamp(dot(q - a, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
    return length(q - a - ab * t) - mix(ra, rb, t);
}

fn blob(q: vec2<f32>, c: vec2<f32>, r: f32, stretch: f32) -> f32 {
    return length((q - c) / vec2(stretch, 1.0)) - r;
}

// The family's signed distance (negative inside).
fn envelope(q: vec2<f32>, seed: f32) -> f32 {
    let pick = hash(seed, 1.0);
    let k = 0.15 + 0.2 * hash(seed, 2.0);
    var d = 9.0;
    if (pick < 0.35) {
        // Compact: a body ringed with lobes.
        d = blob(q, vec2(0.0), 0.6 + 0.3 * hash(seed, 3.0), 0.7 + 0.7 * hash(seed, 4.0));
        for (var i = 0; i < 7; i++) {
            if (f32(i) > 3.0 + 4.0 * hash(seed, 5.0)) { break; }
            let a = 6.283 * hash(seed, 10.0 + f32(i));
            let reach = 0.5 + 0.4 * hash(seed, 20.0 + f32(i));
            d = smin(d, blob(q, vec2(cos(a), sin(a)) * reach, 0.2 + 0.3 * hash(seed, 30.0 + f32(i)), 1.0), k);
        }
    } else if (pick < 0.65) {
        // Comet: a head trailing a tapering, bending tail, knots on its head.
        let head = 0.5 + 0.3 * hash(seed, 3.0);
        d = blob(q, vec2(0.0), head, 0.8 + 0.4 * hash(seed, 4.0));
        let len = 1.5 + 2.0 * hash(seed, 5.0);
        let bend = (hash(seed, 6.0) - 0.5) * 1.2;
        var prev = vec2(0.0);
        var w = head * (0.6 + 0.3 * hash(seed, 7.0));
        for (var i = 1; i <= 5; i++) {
            let t = f32(i) / 5.0;
            let pt = vec2(bend * t * t * len * 0.5, t * len);
            let w2 = head * (1.0 - t) * 0.7 + 0.05;
            d = smin(d, capsule(q, prev, pt, w, w2), k * 0.6);
            prev = pt;
            w = w2;
        }
        for (var i = 0; i < 3; i++) {
            if (f32(i) > 3.0 * hash(seed, 8.0)) { break; }
            let a = -2.6 + 2.1 * hash(seed, 40.0 + f32(i));
            let reach = head * (0.7 + 0.3 * hash(seed, 50.0 + f32(i)));
            d = smin(d, blob(q, vec2(cos(a), sin(a)) * reach, 0.15 + 0.2 * hash(seed, 60.0 + f32(i)), 1.0), k);
        }
    } else if (pick < 0.85) {
        // Cluster: loose knots.
        for (var i = 0; i < 7; i++) {
            if (f32(i) > 3.0 + 4.0 * hash(seed, 5.0)) { break; }
            let c = (vec2(hash(seed, 70.0 + f32(i)), hash(seed, 80.0 + f32(i))) - 0.5) * 2.0;
            d = smin(d, blob(q, c, 0.15 + 0.3 * hash(seed, 90.0 + f32(i)), 0.7 + 0.7 * hash(seed, 100.0 + f32(i))), k * 0.5);
        }
    } else {
        // Filament: a bent chain.
        let a = 3.1416 * hash(seed, 3.0);
        let u = vec2(cos(a), sin(a));
        let n = vec2(-u.y, u.x);
        let len = 1.8 + 1.2 * hash(seed, 4.0);
        let bend = (hash(seed, 5.0) - 0.5) * 1.6;
        var prev = vec2(0.0);
        var pw = 0.0;
        for (var i = 0; i < 7; i++) {
            let t = f32(i) / 6.0 - 0.5;
            let pt = u * t * len + n * bend * (t * t - 0.25);
            let w = 0.15 + 0.2 * hash(seed, 110.0 + f32(i));
            if (i > 0) {
                d = smin(d, capsule(q, prev, pt, pw, w), k * 0.5);
            }
            prev = pt;
            pw = w;
        }
    }
    // Second level, for every family: sub-lobes melted onto the body, and
    // torn-off knots floating around it.
    for (var i = 0; i < 6; i++) {
        if (f32(i) > 2.0 + 4.0 * hash(seed, 130.0)) { break; }
        let a = 6.283 * hash(seed, 140.0 + f32(i));
        let c = vec2(cos(a), sin(a)) * (0.7 + 0.6 * hash(seed, 150.0 + f32(i)));
        d = smin(d, blob(q, c, 0.12 + 0.18 * hash(seed, 160.0 + f32(i)), 1.0), k * 0.5);
    }
    for (var i = 0; i < 4; i++) {
        if (f32(i) > 4.0 * hash(seed, 170.0)) { break; }
        let a = 6.283 * hash(seed, 180.0 + f32(i));
        let c = vec2(cos(a), sin(a)) * (1.4 + 0.8 * hash(seed, 190.0 + f32(i)));
        d = min(d, blob(q, c, 0.06 + 0.1 * hash(seed, 200.0 + f32(i)), 1.0));
    }
    return d;
}

// Round bumps of one scale: 1 at a bump's centre, a dome falling to 0 at its
// radius (one bump per grid cell).
fn domes(q: vec2<f32>, cell: f32, seed: f32) -> f32 {
    let g = floor(q / cell);
    var best = 0.0;
    for (var i = 0; i < 9; i++) {
        let c = g + vec2(f32(i % 3), f32(i / 3)) - 1.0;
        let h = fract(sin(vec3(dot(c, vec2(127.1, 311.7)), dot(c, vec2(269.5, 183.3)), dot(c, vec2(113.5, 271.9))) + seed) * 43758.5453);
        let r = cell * (0.5 + 0.4 * h.z);
        let x = length(q - (c + h.xy) * cell) / r;
        best = max(best, sqrt(clamp(1.0 - x * x, 0.0, 1.0)));
    }
    return best;
}

// fBm of gradient noise, normalised to about unit spread; octaves fade out as
// they shrink below a pixel (`footprint`: one pixel, in noise units).
fn tear(q: vec2<f32>, seed: f32, footprint: f32) -> f32 {
    var p = vec3(q, seed);
    var a = 0.5;
    var f = 1.0;
    var s = 0.0;
    for (var i = 0; i < 6; i++) {
        s += a * (1.0 - smoothstep(0.3, 0.5, footprint * f)) * gnoise(p);
        p = p * 2.0 + vec3(3.1, 7.7, 1.3);
        f *= 2.0;
        a *= 0.5;
    }
    return s / 0.15;
}

// The silhouette's field (> 0 inside, 0 on its edge). `footprint`: one
// texel of the atlas, in head radii.
fn globule_field(q: vec2<f32>, seed: f32, footprint: f32, buoyancy: f32) -> f32 {
    let soft = 0.25 + 0.15 * hash(seed, 120.0);
    let e = 1.0 - clamp((envelope(q, seed) + soft) / (2.0 * soft), 0.0, 1.0);
    let env = e * e * (3.0 - 2.0 * e);
    // The detail only tears the envelope's soft band; the core and the
    // outside stay as they are.
    let band = 4.0 * env * (1.0 - env);
    if (band < 1e-3) {
        return env - 0.5;
    }
    let f = 1.0 + 0.8 * hash(seed, 121.0);
    let strength = 0.6 + 0.4 * hash(seed, 122.0);
    var bumps = 0.0;
    var cell = 0.5 + 0.3 * hash(seed, 123.0);
    var a = 1.0;
    for (var i = 0; i < 3; i++) {
        bumps += a * domes(q, cell, seed + f32(i) * 17.0);
        cell *= 0.5;
        a *= 0.55;
    }
    let detail = strength * tear(q * f, seed, footprint * f) + buoyancy * (bumps - 0.6);
    return env - 0.5 + detail * band;
}
