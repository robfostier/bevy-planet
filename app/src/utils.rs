use std::f32::consts::PI;

/// Smooth in-out sine easing: slow at both ends, fastest around the middle.
pub(crate) fn ease_in_out_sine(x: f32) -> f32 {
    debug_assert!(
        (0.0..=1.0).contains(&x),
        "x={}, should be between 0 and 1",
        x
    );

    -(f32::cos(PI * x) - 1.0) / 2.0
}
