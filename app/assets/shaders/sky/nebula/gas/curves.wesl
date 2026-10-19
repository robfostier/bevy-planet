// Scalar shaping curves applied to the nebula's noise fields.

fn smooth_unit(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

fn levels(v: f32, lo: f32, hi: f32) -> f32 {
    return clamp((v - lo) / (hi - lo), 0.0, 1.0);
}

// `powf` of a non-negative base; WGSL pow is undefined for x < 0 and
// pow(0, y) must give 0 for y > 0.
fn powf(x: f32, y: f32) -> f32 {
    if (x <= 0.0) {
        return select(0.0, 1.0, y == 0.0);
    }
    return pow(x, y);
}

fn contrast(v: f32, strength: f32) -> f32 {
    let a = powf(v, strength);
    let b = powf(1.0 - v, strength);
    return a / (a + b);
}
