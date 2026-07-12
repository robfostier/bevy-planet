//! Small scalar helper curves: easing and tone-shaping functions (levels, gamma,
//! contrast, smoothstep) reused across the codebase.

use std::f32::consts::PI;

/// Smooth in-out sine easing: slow at both ends, fastest around the middle.
pub(crate) fn ease_in_out_sine(t: f32) -> f32 {
    debug_assert!(
        (0.0..=1.0).contains(&t),
        "t={}, should be between 0 and 1",
        t
    );

    -(f32::cos(PI * t) - 1.0) / 2.0
}

/// Smoothstep: the classic `3t^2 - 2t^3` ease on `[0, 1]`, flat at both ends.
/// Assumes `t` is already within `[0, 1]` (does not clamp).
pub(crate) fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Maps a value in `[floor, ceil]` to `[0, 1]`, clamping values outside the range to 0 or 1.
pub(crate) fn levels(value: f32, floor: f32, ceil: f32) -> f32 {
    ((value - floor) / (ceil - floor)).clamp(0.0, 1.0)
}

/// Gamma curve: `value.powf(gamma)`. Below 1 lifts midtones (brighter), above 1
/// crushes them (darker).
pub(crate) fn gamma(value: f32, gamma: f32) -> f32 {
    value.powf(gamma)
}

/// Ratio-based S-curve contrast around 0.5: raises `value` and its complement to
/// `strength` and renormalises, so higher `strength` pushes tones toward 0 or 1.
pub(crate) fn contrast(value: f32, strength: f32) -> f32 {
    let value_weight = value.powf(strength);
    let complement_weight = (1.0 - value).powf(strength);
    value_weight / (value_weight + complement_weight)
}
