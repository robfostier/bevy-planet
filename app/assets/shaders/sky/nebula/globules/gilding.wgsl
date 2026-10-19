// The gilding: short strokes of saturated gold laid on a globule's lit edge
// (after the painted accents of Supergiant's Hades).

#import "shaders/common/gradient_noise.wgsl"::gnoise
#import "shaders/common/color.wgsl"::LUMA

// Strokes as thick as the rim on the edge, cut into dashes by a noise of
// DASHES per head radius (fixed on the globule), from a pale to a deep gold
// (paler, champagne, over cold gas: it reads better on blue), GILD_GAIN times
// as bright as the rim.
const DASHES: f32 = 3.0;
const GILD_GAIN: f32 = 2.5;
// The dash noise (gradient noise, about -0.3..0.3) is spread over 0..1 by
// NOISE_SPREAD; a dash's ends soften over DASH_SOFTNESS of that range.
const NOISE_SPREAD: f32 = 1.7;
const DASH_SOFTNESS: f32 = 0.04;
const GOLD_PALE: vec3<f32> = vec3(1.0, 0.8, 0.32);
const GOLD_DEEP: vec3<f32> = vec3(1.0, 0.5, 0.1);
const GOLD_COLD: vec3<f32> = vec3(1.0, 0.93, 0.68);

// The gilding at `q` (its frame; `seed`: the globule's), `toward` pixels from
// the lit edge, on a rim `width` pixels thick, over gas `cold` (0 warm, 1
// cold): rgb its gold (GILD_GAIN luminance), a its coverage: the edge's
// `share` (0..1) is gilded.
fn gild(q: vec2<f32>, seed: f32, toward: f32, width: f32, share: f32, cold: f32) -> vec4<f32> {
    let stroke = 1.0 - smoothstep(width - 0.5, width + 0.5, toward);
    // The dash noise spread over 0..1, gilded above 1 - share (none at 0,
    // all at 1).
    let n = clamp(gnoise(vec3(q * DASHES, seed + 31.0)) * NOISE_SPREAD + 0.5, 0.0, 0.999);
    let cut = mix(1.0 + DASH_SOFTNESS, -DASH_SOFTNESS, share);
    let dash = smoothstep(cut - DASH_SOFTNESS, cut, n);
    let warm = mix(GOLD_PALE, GOLD_DEEP, smoothstep(-0.3, 0.3, gnoise(vec3(q * 3.0, seed + 57.0))));
    let gold = mix(warm, GOLD_COLD, cold);
    return vec4(gold / dot(gold, LUMA) * GILD_GAIN, stroke * dash);
}
