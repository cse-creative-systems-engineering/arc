// Kinetic per-glyph instanced renderer (Milestone 0001, REQ-UX-002).

struct Camera {
    screen_size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

struct GlyphInstance {
    pos: vec2<f32>,
    size: vec2<f32>,
    uv0: vec2<f32>,
    uv1: vec2<f32>,
    scale: f32,
    alpha: f32,
    pad: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @location(0) instance_pos: vec2<f32>,
    @location(1) instance_size: vec2<f32>,
    @location(2) instance_uv0: vec2<f32>,
    @location(3) instance_uv1: vec2<f32>,
    @location(4) instance_alpha: f32,
) -> VertexOutput {
    // Two-triangle quad from instance rect.
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let c = corners[vertex];
    let px = instance_pos + c * instance_size;

    var out: VertexOutput;
    let ndc = vec2<f32>(
        px.x / camera.screen_size.x * 2.0 - 1.0,
        1.0 - px.y / camera.screen_size.y * 2.0,
    );
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = mix(instance_uv0, instance_uv1, c);
    out.alpha = instance_alpha;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Warm-white ink matching Arc's kinetic typography palette.
    let coverage = textureSample(t_atlas, s_atlas, in.uv).r;
    let ink = vec3<f32>(0.93, 0.91, 0.88);
    return vec4<f32>(ink, coverage * in.alpha);
}
