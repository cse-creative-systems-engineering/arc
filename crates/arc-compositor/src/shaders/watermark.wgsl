// ARC wordmark composite pass — samples the rasterized wordmark texture
// and applies the cinematic fade. Replaces the hand-drawn SDF.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

struct WatermarkUniforms {
    origin: vec2<f32>,
    size: vec2<f32>,
    screen: vec2<f32>,
    alpha: f32,
    pad: vec3<f32>,
    pad2: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> wm: WatermarkUniforms;
@group(0) @binding(1)
var t_wordmark: texture_2d<f32>;
@group(0) @binding(2)
var s_wordmark: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pure OLED black void.
    let base_black = vec3<f32>(0.005, 0.006, 0.008);

    // Wordmark quad in UV space (y-down, matching framebuffer UVs).
    let quad_min = wm.origin / wm.screen;
    let quad_max = (wm.origin + wm.size) / wm.screen;

    // Outside the quad: void only (plus subtle vignette).
    var coverage = 0.0;
    var local_uv = vec2<f32>(0.0);
    if (in.uv.x >= quad_min.x && in.uv.x <= quad_max.x
        && in.uv.y >= quad_min.y && in.uv.y <= quad_max.y) {
        local_uv = (in.uv - quad_min) / (quad_max - quad_min);
        coverage = textureSample(t_wordmark, s_wordmark, local_uv).r;
    }

    // Warm titanium luminance, softly bloomed around the letterforms.
    let glow = exp(-coverage * 0.0) * 0.0; // coverage-weighted below
    let bloom = smoothstep(0.05, 1.0, coverage);

    let titanium = vec3<f32>(0.92, 0.90, 0.87);
    let lit = base_black + titanium * (coverage * 0.85 + bloom * 0.35);

    // Subtle edge vignette over the whole canvas.
    let vignette = 1.0 - length(in.uv - vec2<f32>(0.5, 0.5)) * 0.15;

    return vec4<f32>(lit * vignette * wm.alpha, 1.0);
}
