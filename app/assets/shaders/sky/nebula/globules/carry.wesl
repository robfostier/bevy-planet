// The globules ride the gas: they live in the style's sky (the gas at
// mid-life), and the carry (`gas/bake.wesl`: xyz where the style's gas now
// shows, minus the direction) moves each one, as one piece, with the gas at
// its centre (cubemaps are left-handed).

// How much farther than its reach a globule's centre may sit from a pixel's
// point in the style's sky: the gas carries the globule by its centre, the
// pixel by its own point, and the two differ a little.
const CARRY_MARGIN: f32 = 1.1;

fn carry_at(carry: texture_cube<f32>, carry_sampler: sampler, dir: vec3<f32>) -> vec4<f32> {
    return textureSampleLevel(carry, carry_sampler, normalize(dir) * vec3(1.0, 1.0, -1.0), 0.0);
}

// The style's point shown in `dir` (the carry is smooth: its value at `dir`
// stands for the one at the answer).
fn style_point(carry: texture_cube<f32>, carry_sampler: sampler, dir: vec3<f32>) -> vec3<f32> {
    return normalize(dir - carry_at(carry, carry_sampler, dir).xyz);
}

// Where the style's point `p` now shows.
fn carried(carry: texture_cube<f32>, carry_sampler: sampler, p: vec3<f32>) -> vec3<f32> {
    return normalize(p + carry_at(carry, carry_sampler, p).xyz);
}
