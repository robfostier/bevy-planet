// The `noise` crate's Perlin / Fbm / RidgedMulti, reproduced exactly: same
// permutation tables (uploaded by `sky/nebula/gas`), same gradients, fades and
// octave weights.

// One noise field's settings (mirrors `GpuNoiseField`).
struct NoiseField {
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    scale_factor: f32,
    octaves: u32,
    // Offset (in u32 entries) of this field's first permutation table.
    table_base: u32,
    pad_a: u32,
    pad_b: u32,
}

@group(0) @binding(2) var<storage, read> tables: array<u32>;

// `noise`'s floor_to_isize: truncation, minus one for values <= 0 (note: an
// exact 0 maps to -1, not 0 -- reproduced on purpose).
fn floor_to_int(v: f32) -> i32 {
    let t = i32(v);
    return select(t, t - 1, v <= 0.0);
}

fn perm(base: u32, i: i32) -> i32 {
    return i32(tables[base + u32(i & 255)]);
}

// PermutationTable::hash for 3 coordinates: P[P[P[x] ^ y] ^ z] (each masked to 8 bits).
fn hash_corner(base: u32, c: vec3<i32>) -> i32 {
    let a = perm(base, c.x) ^ (c.y & 255);
    let b = perm(base, a) ^ (c.z & 255);
    return perm(base, b);
}

fn gradient(base: u32, corner: vec3<i32>, offset: vec3<i32>, distance: vec3<f32>) -> f32 {
    let p = distance - vec3<f32>(offset);
    switch (hash_corner(base, corner + offset) & 15) {
        case 0, 12: { return p.x + p.y; }
        case 1, 13: { return -p.x + p.y; }
        case 2: { return p.x - p.y; }
        case 3: { return -p.x - p.y; }
        case 4: { return p.x + p.z; }
        case 5: { return -p.x + p.z; }
        case 6: { return p.x - p.z; }
        case 7: { return -p.x - p.z; }
        case 8: { return p.y + p.z; }
        case 9, 14: { return -p.y + p.z; }
        case 10: { return p.y - p.z; }
        default: { return -p.y - p.z; } // 11, 15
    }
}

fn quintic(v: vec3<f32>) -> vec3<f32> {
    let x = clamp(v, vec3(0.0), vec3(1.0));
    return x * x * x * (x * (x * 6.0 - 15.0) + 10.0);
}

// noise's `linear(a, b, t)` = b * t + a * (1 - t)
fn lin(a: f32, b: f32, t: f32) -> f32 {
    return b * t + a * (1.0 - t);
}

fn perlin(base: u32, point: vec3<f32>) -> f32 {
    let corner = vec3<i32>(floor_to_int(point.x), floor_to_int(point.y), floor_to_int(point.z));
    let d = point - vec3<f32>(corner);

    let g000 = gradient(base, corner, vec3(0, 0, 0), d);
    let g100 = gradient(base, corner, vec3(1, 0, 0), d);
    let g010 = gradient(base, corner, vec3(0, 1, 0), d);
    let g110 = gradient(base, corner, vec3(1, 1, 0), d);
    let g001 = gradient(base, corner, vec3(0, 0, 1), d);
    let g101 = gradient(base, corner, vec3(1, 0, 1), d);
    let g011 = gradient(base, corner, vec3(0, 1, 1), d);
    let g111 = gradient(base, corner, vec3(1, 1, 1), d);

    let c = quintic(d);
    let r = lin(
        lin(lin(g000, g001, c.z), lin(g010, g011, c.z), c.y),
        lin(lin(g100, g101, c.z), lin(g110, g111, c.z), c.y),
        c.x,
    ) * 1.1547005383792515;
    return clamp(r, -1.0, 1.0);
}

fn fbm(f: NoiseField, p_in: vec3<f32>) -> f32 {
    var p = p_in * f.frequency;
    var result = 0.0;
    var attenuation = f.persistence;
    for (var o = 0u; o < f.octaves; o++) {
        result += perlin(f.table_base + o * 256u, p) * attenuation;
        attenuation *= f.persistence;
        p *= f.lacunarity;
    }
    return result * f.scale_factor;
}

fn ridged(f: NoiseField, ridge_attenuation: f32, p_in: vec3<f32>) -> f32 {
    var p = p_in * f.frequency;
    var result = 0.0;
    var weight = 1.0;
    var attenuation = 1.0;
    for (var o = 0u; o < f.octaves; o++) {
        var signal = perlin(f.table_base + o * 256u, p);
        signal = 1.0 - abs(signal);
        signal *= signal;
        signal *= weight;
        weight = clamp(signal / ridge_attenuation, 0.0, 1.0);
        signal *= attenuation;
        attenuation *= f.persistence;
        result += signal;
        p *= f.lacunarity;
    }
    return result * f.scale_factor - 1.0;
}
