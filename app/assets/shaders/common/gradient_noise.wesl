// 3D gradient noise (Inigo Quilez) and its fBm, shared by the star and the
// globules.
// `fbm_lod` adds detail with proximity: past its base octaves, it keeps adding
// finer ones while they stay larger than a pixel (`footprint`: the size of one
// pixel in noise units, from `fwidth`), fading the last one in. Far away it is
// exactly the base fBm; up close, finer and finer structure appears.

fn hash_gradient(p: vec3<f32>) -> vec3<f32> {
    let q = vec3(
        dot(p, vec3(127.1, 311.7, 74.7)),
        dot(p, vec3(269.5, 183.3, 246.1)),
        dot(p, vec3(113.5, 271.9, 124.6)),
    );
    return -1.0 + 2.0 * fract(sin(q) * 43758.5453123);
}

// Gradient noise in about [-1, 1], with a quintic fade.
fn gnoise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    return mix(
        mix(
            mix(dot(hash_gradient(i + vec3(0., 0., 0.)), f - vec3(0., 0., 0.)),
                dot(hash_gradient(i + vec3(1., 0., 0.)), f - vec3(1., 0., 0.)), u.x),
            mix(dot(hash_gradient(i + vec3(0., 1., 0.)), f - vec3(0., 1., 0.)),
                dot(hash_gradient(i + vec3(1., 1., 0.)), f - vec3(1., 1., 0.)), u.x), u.y),
        mix(
            mix(dot(hash_gradient(i + vec3(0., 0., 1.)), f - vec3(0., 0., 1.)),
                dot(hash_gradient(i + vec3(1., 0., 1.)), f - vec3(1., 0., 1.)), u.x),
            mix(dot(hash_gradient(i + vec3(0., 1., 1.)), f - vec3(0., 1., 1.)),
                dot(hash_gradient(i + vec3(1., 1., 1.)), f - vec3(1., 1., 1.)), u.x), u.y),
        u.z);
}

// Four octaves of gradient noise.
fn fbm(p_in: vec3<f32>) -> f32 {
    var p = p_in;
    var a = 0.5;
    var s = 0.0;
    for (var i = 0; i < 4; i++) {
        s += a * gnoise(p);
        p = p * 2.03 + vec3(1.7, 9.2, 3.1);
        a *= 0.5;
    }
    return s;
}

// Octaves added at most past the base ones, when seen up close.
const DETAIL_OCTAVES: i32 = 6;

// fBm with `base` octaves always, plus finer octaves while larger than a pixel.
fn fbm_lod(p_in: vec3<f32>, footprint: f32, base: i32) -> f32 {
    var p = p_in;
    var a = 0.5;
    var frequency = 1.0;
    var s = 0.0;
    for (var i = 0; i < base + DETAIL_OCTAVES; i++) {
        var weight = 1.0;
        if (i >= base) {
            // An octave fades in as its wavelength grows from ~10 to ~5 pixels,
            // so it never adds sub-pixel noise, and far away nothing changes.
            weight = 1.0 - smoothstep(0.1, 0.2, footprint * frequency);
            if (weight <= 0.0) {
                break;
            }
            // Finer octaves keep a bit more energy, so the detail reads.
            a *= 1.2;
        }
        s += a * weight * gnoise(p);
        p = p * 2.03 + vec3(1.7, 9.2, 3.1);
        frequency *= 2.03;
        a *= 0.5;
    }
    return s;
}
