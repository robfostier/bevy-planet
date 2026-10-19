//! Small scalar helper curves reused across the codebase.

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
