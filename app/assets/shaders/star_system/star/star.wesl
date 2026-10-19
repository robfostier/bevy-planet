// Star surface: a warm core-to-limb gradient, animated domain-warped granulation
// and dark ridged "cracks" (detailed with proximity, receding from afar), plus a
// bright chromosphere rim. Output is in display-linear HDR units (not scaled by
// the camera exposure), so it always blooms the same.
// The vertex shader gently displaces the sphere with slow, low-frequency noise,
// so the silhouette is never a perfect circle.

import bevy_pbr::render::{
    forward_io::{Vertex, VertexOutput},
    mesh_view_bindings::{view, globals},
    mesh_functions,
    view_transformations::position_world_to_clip,
};
import package::shaders::common::gradient_noise::{fbm, fbm_lod};

struct Star {
    // rgb color, a intensity (display-linear multiplier).
    core: vec4<f32>,
    mid: vec4<f32>,
    limb: vec4<f32>,
    spot: vec4<f32>,
    crack: vec4<f32>,
    // x granule scale, y crack scale, z flow speed, w warp amount
    noise: vec4<f32>,
    // x chromosphere width, y limb exponent
    style: vec4<f32>,
    // x spot threshold, y crack width, z crack coverage, w spot strength
    shape: vec4<f32>,
    // Thin bright rim at the very edge (rgb color, a intensity).
    chromosphere: vec4<f32>,
    // Silhouette: x displacement (fraction of the radius), y noise frequency, z flow speed
    outline: vec4<f32>,
    // x granulation strength seen from afar (1 = as up close)
    texture: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> star: Star;

// A point of the unit sphere, pushed in or out by the slow silhouette noise.
fn displaced(dir: vec3<f32>) -> vec3<f32> {
    let flow = globals.time * star.outline.z;
    let bump = fbm(dir * star.outline.y + vec3(0.0, flow, 0.0));
    return dir * (1.0 + star.outline.x * bump);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    // Push the surface in and out along its normal (a few % of the radius),
    // and take the normal of that displaced surface (finite differences), so
    // the limb gradient and the chromosphere follow the irregular silhouette.
    let radius = length(vertex.position);
    let dir = vertex.position / radius;
    let side = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(dir.y) > 0.9);
    let t1 = normalize(cross(dir, side));
    let t2 = cross(dir, t1);
    let eps = 0.01;
    let p0 = displaced(dir) * radius;
    let p1 = displaced(normalize(dir + eps * t1)) * radius;
    let p2 = displaced(normalize(dir + eps * t2)) * radius;
    let normal = normalize(cross(p1 - p0, p2 - p0));
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4(p0, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(normal, vertex.instance_index);
    @if(VERTEX_UVS_A) out.uv = vertex.uv;
    @if(VERTEX_OUTPUT_INSTANCE_INDEX) out.instance_index = vertex.instance_index;
    return out;
}

// A soft threshold: the ramp spans at least a tenth of the noise's range, so
// shapes melt into each other like gas; up close, `fbm_lod`'s finer octaves
// keep the detail inside those ramps.
fn edge(x: f32, threshold: f32) -> f32 {
    let soft = max(fwidth(x) * 2.0, 0.1);
    return smoothstep(threshold - soft, threshold + soft, x);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position - in.world_position.xyz);
    let mu = clamp(dot(n, v), 0.0, 1.0);
    let t = globals.time;

    // Object-space direction of this point, so the pattern turns with the
    // star's spin (from the position: the normal is bent by the displacement).
    let world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    let local_from_world = transpose(mat3x3<f32>(
        normalize(world_from_local[0].xyz),
        normalize(world_from_local[1].xyz),
        normalize(world_from_local[2].xyz),
    ));
    let p = normalize(local_from_world * (in.world_position.xyz - world_from_local[3].xyz));

    // Core -> mid -> limb gradient, by distance to the disc centre.
    // Projected radius on the disc (0 centre, 1 silhouette): a radial gradient,
    // as drawn, rather than the view angle (which stays flat over most of the disc).
    let limb_t = pow(sqrt(1.0 - mu * mu), star.style.y);
    var color = mix(star.core.rgb * star.core.a, star.mid.rgb * star.mid.a, smoothstep(0.0, 0.6, limb_t));
    color = mix(color, star.limb.rgb * star.limb.a, smoothstep(0.55, 1.0, limb_t));

    // Granulation: bright patches drifting over the surface (domain-warped fBm).
    let gp = p * star.noise.x;
    let warp = vec3(
        fbm(gp + vec3(0.0, 0.0, t * star.noise.z)),
        fbm(gp + vec3(5.2, 1.3, t * star.noise.z)),
        fbm(gp + vec3(9.1, 4.7, t * star.noise.z)),
    );
    // Size of one pixel in granulation units: more octaves as it shrinks.
    let footprint = length(fwidth(gp));
    let g = fbm_lod(gp + star.noise.w * warp + vec3(0.0, t * star.noise.z * 0.5, 0.0), footprint, 4);
    // Seen from afar, the granulation recedes so the core-to-limb gradient
    // reads as a smooth, painted blend; it comes back as the camera nears.
    let texture = mix(1.0, star.texture.x, smoothstep(0.015, 0.06, footprint));
    let spot = edge(g, star.shape.x) * star.shape.w * (1.0 - 0.6 * limb_t) * texture;
    color = mix(color, star.spot.rgb * star.spot.a, spot);

    // Cracks: thin dark ridges between the cells.
    let crack_p = p * star.noise.y + star.noise.w * 1.5 * warp;
    let crack_footprint = footprint * star.noise.y / max(star.noise.x, 1e-3);
    let ridge = 1.0 - abs(2.0 * fbm_lod(crack_p, crack_footprint, 1));
    let crack = edge(ridge, 1.0 - star.shape.y) * smoothstep(star.shape.z, 1.0, 1.0 - abs(g));
    color = mix(color, star.crack.rgb * star.crack.a, crack * (0.35 + 0.65 * limb_t) * texture);

    // Chromosphere: past the darkened limb, a thin bright rim at the very
    // edge (as in eclipse or H-alpha photos), from which the corona rises.
    let radius = sqrt(1.0 - mu * mu);
    let chromosphere = smoothstep(1.0 - star.style.x, 1.0 - star.style.x * 0.4, radius);
    color = mix(color, star.chromosphere.rgb * star.chromosphere.a, chromosphere);

    return vec4(color, 1.0);
}
