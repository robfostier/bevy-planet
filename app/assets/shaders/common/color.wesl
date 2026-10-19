// Colour helpers: luminance, and Oklab (perceptual) conversions and hue
// rotation, with bevy_color's LinearRgba <-> Oklab matrices (they match
// `Oklcha` on the CPU).

// Relative luminance of a linear RGB colour (Rec. 709).
const LUMA: vec3<f32> = vec3(0.2126, 0.7152, 0.0722);

fn cbrt(x: f32) -> f32 {
    return sign(x) * pow(abs(x), 1.0 / 3.0);
}

// Rotates an Oklab colour's hue by `shift` degrees (its lightness and chroma
// kept).
fn rotate_hue(lab: vec3<f32>, shift: f32) -> vec3<f32> {
    let c = cos(radians(shift));
    let s = sin(radians(shift));
    return vec3(lab.x, lab.y * c - lab.z * s, lab.y * s + lab.z * c);
}

fn to_oklab(c: vec3<f32>) -> vec3<f32> {
    let l = cbrt(0.41222146 * c.r + 0.53633255 * c.g + 0.051445995 * c.b);
    let m = cbrt(0.2119035 * c.r + 0.6806995 * c.g + 0.10739696 * c.b);
    let s = cbrt(0.08830246 * c.r + 0.28171885 * c.g + 0.6299787 * c.b);
    return vec3(
        0.21045426 * l + 0.7936178 * m - 0.004072047 * s,
        1.9779985 * l - 2.4285922 * m + 0.4505937 * s,
        0.025904037 * l + 0.78277177 * m - 0.80867577 * s,
    );
}

fn from_oklab(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let m_ = lab.x - 0.105561346 * lab.y - 0.06385417 * lab.z;
    let s_ = lab.x - 0.08948418 * lab.y - 1.2914855 * lab.z;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;
    return vec3(
        4.0767417 * l - 3.3077116 * m + 0.23096994 * s,
        -1.268438 * l + 2.6097574 * m - 0.34131938 * s,
        -0.0041960863 * l - 0.7034186 * m + 1.7076147 * s,
    );
}
