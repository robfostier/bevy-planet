//! From a star's properties to its look: the base `StarStyle` (the Sun) is
//! re-centred on the star's blackbody color, brightened or dimmed with its
//! temperature (its size is applied as the entity's scale).

use bevy::{
    color::{LinearRgba, Oklaba},
    prelude::*,
};

use super::{
    physics::{SUN_TEMPERATURE, StarProperties},
    style::{StarStyle, Tone},
};

/// Blackbody color (Tanner Helland's fit), linear, normalised to unit luminance.
fn blackbody(kelvin: f32) -> Vec3 {
    let t = kelvin / 100.0;
    let (r, g) = if t <= 66.0 {
        (255.0, 99.470_8 * t.ln() - 161.119_57)
    } else {
        (
            329.698_73 * (t - 60.0).powf(-0.133_204_76),
            288.122_17 * (t - 60.0).powf(-0.075_514_85),
        )
    };
    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };
    let srgb = Srgba::rgb(r / 255.0, g / 255.0, b / 255.0);
    let srgb = Srgba::rgb(
        srgb.red.clamp(0.0, 1.0),
        srgb.green.clamp(0.0, 1.0),
        srgb.blue.clamp(0.0, 1.0),
    );
    let linear = LinearRgba::from(srgb);
    let c = Vec3::new(linear.red, linear.green, linear.blue);
    c / c.dot(Vec3::new(0.2126, 0.7152, 0.0722)).max(1e-4)
}

fn oklab(c: Vec3) -> Oklaba {
    Oklaba::from(LinearRgba::rgb(c.x, c.y, c.z))
}

/// Re-centres a color on this star's blackbody color: its offset from the
/// Sun's blackbody color, in Oklab's a/b plane (the base style's warm limb,
/// cooler cracks...), is kept but scaled by `contrast`, and lightness is kept,
/// so the base style's gradients survive on any star.
fn tint(color: Vec3, from: Oklaba, to: Oklaba, contrast: f32) -> Vec3 {
    let mut lab = oklab(color);
    lab.a = to.a + (lab.a - from.a) * contrast;
    lab.b = to.b + (lab.b - from.b) * contrast;
    let linear = LinearRgba::from(lab);
    Vec3::new(linear.red, linear.green, linear.blue).max(Vec3::ZERO)
}

/// The look of a star with these properties, derived from the base style.
pub(super) fn appearance(base: &StarStyle, star: &StarProperties) -> StarStyle {
    let from = oklab(blackbody(SUN_TEMPERATURE));
    let to = oklab(blackbody(star.temperature));
    let ratio = star.temperature / SUN_TEMPERATURE;
    // Hot stars show weaker color contrast across the disc (their limb is
    // barely reddened); cool ones keep the base style's warm contrast.
    let contrast = ratio.powf(-0.6).min(1.0);
    // Hotter stars burn brighter (surface brightness grows as T^4; strongly
    // compressed, or they would only read as white).
    let brightness = ratio.powf(0.35);
    let tone = |t: Tone| Tone {
        color: tint(t.color, from, to, contrast),
        intensity: t.intensity * brightness,
    };
    StarStyle {
        core: tone(base.core),
        mid: tone(base.mid),
        limb: tone(base.limb),
        spot: tone(base.spot),
        crack: tone(base.crack),
        chromosphere: tone(base.chromosphere),
        light: tint(base.light, from, to, contrast),
        ..base.clone()
    }
}
