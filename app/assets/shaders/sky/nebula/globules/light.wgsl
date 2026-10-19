// The side a globule's light comes from: towards the densest of the style's
// gas around it (its density, not its colour or its evolution, so the globule
// keeps its orientation whatever the palette or the system's age).

#import "shaders/sky/nebula/gas/look.wgsl"::cube_texel

// `density`: the style's density (`gas/bake.wgsl`); `center`, `radius`: the
// globule, in the style's sky.
fn light_side(density: texture_cube<f32>, density_sampler: sampler, center: vec3<f32>, radius: f32) -> vec3<f32> {
    let up = cross(vec3(0.0, 1.0, 0.0), center);
    let east = normalize(select(up, vec3(1.0, 0.0, 0.0), dot(up, up) < 1e-6));
    let north = cross(center, east);
    var slope = vec2(0.0);
    for (var k = 0; k < 4; k++) {
        let o = vec2(cos(f32(k) * 1.5708), sin(f32(k) * 1.5708));
        slope += o * cube_texel(density, density_sampler, center + (east * o.x + north * o.y) * radius * 1.5).r;
    }
    let towards = select(vec2(0.0, 1.0), slope / max(length(slope), 1e-9), length(slope) > 1e-9);
    return east * towards.x + north * towards.y;
}
