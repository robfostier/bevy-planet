// Cubemap faces baked by compute shaders: the direction of a face's texel
// (cubemaps are left-handed, hence the flipped z).

fn face_uv_to_direction(face: u32, u: f32, v: f32) -> vec3<f32> {
    var d: vec3<f32>;
    switch (face) {
        case 0u: { d = vec3(1.0, -v, -u); }
        case 1u: { d = vec3(-1.0, -v, u); }
        case 2u: { d = vec3(u, 1.0, v); }
        case 3u: { d = vec3(u, -1.0, -v); }
        case 4u: { d = vec3(u, -v, 1.0); }
        default: { d = vec3(-u, -v, -1.0); }
    }
    return normalize(d);
}

// The direction of `texel` on `face`, at `res` texels a side.
fn texel_direction(texel: vec2<u32>, face: u32, res: u32) -> vec3<f32> {
    let uv = (vec2<f32>(texel) + 0.5) / f32(res) * 2.0 - 1.0;
    return face_uv_to_direction(face, uv.x, uv.y) * vec3(1.0, 1.0, -1.0);
}
