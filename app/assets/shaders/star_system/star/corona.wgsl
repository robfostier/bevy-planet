// Star corona: a quad centred on the star and facing the camera, drawn
// additively around the silhouette. A soft glow in the chromosphere's color,
// bright at the limb, that melts into the sky.

#import bevy_pbr::{
    mesh_view_bindings::view,
    mesh_functions,
}

struct Corona {
    // The chromosphere's color (rgb) and intensity (a): the glow continues it.
    color: vec4<f32>,
    // x star radius (world), y quad half-extent in star radii, z glow falloff (radii), w glow intensity
    shape: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> corona: Corona;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // Position in the quad plane, in units of the silhouette radius.
    @location(0) plane: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let center = mesh_functions::get_world_from_local(vertex.instance_index)[3].xyz;
    // Face the camera position (not the screen plane), so the silhouette is a
    // circle centred on the quad wherever the star sits on screen.
    let to_star = normalize(center - view.world_position);
    var right = cross(to_star, normalize(view.world_from_view[1].xyz));
    if (dot(right, right) < 1e-6) {
        right = cross(to_star, normalize(view.world_from_view[0].xyz));
    }
    right = normalize(right);
    let up = cross(right, to_star);
    // Seen from distance d, a sphere's silhouette in the plane through its
    // centre has radius R d / sqrt(d^2 - R^2).
    let radius = corona.shape.x;
    let d = max(distance(view.world_position, center), radius * 1.001);
    let silhouette = radius * d / sqrt(d * d - radius * radius);
    let offset = vertex.position.xy * corona.shape.y; // quad is [-1, 1]^2
    let world = center + (right * offset.x + up * offset.y) * silhouette;

    var out: VertexOutput;
    out.clip_position = view.clip_from_world * vec4(world, 1.0);
    out.plane = offset;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = length(in.plane);
    let h = r - 1.0; // height above the limb, in silhouette radii
    // Starts slightly inside the exact circle, to fill the gaps between it and
    // the sphere's polygonal silhouette (the sphere hides the rest).
    if (h < -0.05) {
        return vec4(0.0);
    }
    // Bright at the limb, then a slow fall-off with a long tail.
    let x = max(h, 0.0) / corona.shape.z;
    let glow = corona.shape.w / (1.0 + x + 2.0 * x * x);
    // Fade out before the quad's edge.
    let fade = 1.0 - smoothstep(corona.shape.y * 0.75, corona.shape.y, r);
    // Premultiplied blend with alpha 0 = additive.
    return vec4(corona.color.rgb * glow * fade, 0.0);
}
