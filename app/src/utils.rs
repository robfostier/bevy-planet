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

/// Cubic polynomial easing: slow at both ends, fastest around the middle.
pub(crate) fn smoothstep(t: f32) -> f32 {
    debug_assert!(
        (0.0..=1.0).contains(&t),
        "t={}, should be between 0 and 1",
        t
    );

    t * t * (3.0 - 2.0 * t)
}

/// Leverages `smoothstep` to map a value in `[floor, ceil]` to `[0, 1]`, clamping
/// values outside the range to 0 or 1.
pub(crate) fn levels(value: f32, floor: f32, ceil: f32) -> f32 {
    let t = ((value - floor) / (ceil - floor)).clamp(0.0, 1.0);
    smoothstep(t)
}
