// The globules' look: their tunables (mirrors `GpuGlobules`; see
// `GlobuleStyle` for each).
struct GlobuleLook {
    count_large: f32,
    count_small: f32,
    // Radians.
    size: f32,
    size_variation: f32,
    // Where the draws start (0..1000), from the seed.
    draw_start: f32,
    opacity: f32,
    gas_faint: f32,
    gas_full: f32,
    // Pixels.
    rim: f32,
    rim_falloff: f32,
    // Stops.
    rim_intensity: f32,
    front_lit: f32,
    // Stops.
    front_intensity: f32,
    gilding: f32,
    // The gas left, relative to the style (1).
    density: f32,
}
