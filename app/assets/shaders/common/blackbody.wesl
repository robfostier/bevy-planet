// Blackbody colours, for stars of a temperature.

import package::shaders::common::color::LUMA;

fn unit_luma(c: vec3<f32>) -> vec3<f32> {
    return c / max(dot(c, LUMA), 1e-4);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3(2.4)), c / 12.92, c <= vec3(0.04045));
}

// Blackbody color (Tanner Helland's fit), normalised to unit luminance so the
// temperature only changes the tint, never the brightness.
fn blackbody(kelvin: f32) -> vec3<f32> {
    let t = kelvin / 100.0;
    var c: vec3<f32>;
    if (t <= 66.0) {
        c.r = 255.0;
        c.g = 99.4708025861 * log(t) - 161.1195681661;
    } else {
        c.r = 329.698727446 * pow(t - 60.0, -0.1332047592);
        c.g = 288.1221695283 * pow(t - 60.0, -0.0755148492);
    }
    if (t >= 66.0) {
        c.b = 255.0;
    } else if (t <= 19.0) {
        c.b = 0.0;
    } else {
        c.b = 138.5177312231 * log(t - 10.0) - 305.0447927307;
    }
    return unit_luma(srgb_to_linear(clamp(c / 255.0, vec3(0.0), vec3(1.0))));
}
