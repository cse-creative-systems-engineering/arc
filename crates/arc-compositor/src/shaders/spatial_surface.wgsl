// 3D Spatial Surface Vertex & Fragment Shader (Milestone 0002, ADR-0010, REQ-SURF-004)
//
// Renders fluid planar surfaces in a continuous 3D perspective frustum with
// depth-of-field rack focus and glassmorphic card borders.

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _pad0: f32,
    focal_distance: f32,
    dof_range: f32,
    _pad1: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

struct SurfaceUniforms {
    model: mat4x4<f32>,
    color_tint: vec4<f32>,
    opacity: f32,
    focus_weight: f32,
    _pad: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> surface: SurfaceUniforms;

@group(1) @binding(1)
var t_content: texture_2d<f32>;
@group(1) @binding(2)
var s_content: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) depth: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos4 = surface.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos4.xyz;
    out.clip_pos = camera.view_proj * world_pos4;
    out.uv = in.uv;
    out.depth = length(out.world_pos - camera.camera_pos);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_sample = textureSample(t_content, s_content, in.uv);

    // Subtle glassmorphic card edge highlight (1px inset border)
    let edge_x = smoothstep(0.0, 0.02, in.uv.x) * smoothstep(1.0, 0.98, in.uv.x);
    let edge_y = smoothstep(0.0, 0.02, in.uv.y) * smoothstep(1.0, 0.98, in.uv.y);
    let border_mask = 1.0 - (edge_x * edge_y);

    let border_color = vec3<f32>(0.85, 0.88, 0.95);
    let surface_base = mix(tex_sample.rgb * surface.color_tint.rgb, border_color, border_mask * 0.4);

    // Depth-of-Field (DoF) rack focus attenuation calculation
    let dist_from_focus = abs(in.depth - camera.focal_distance);
    let dof_blur_factor = clamp(dist_from_focus / camera.dof_range, 0.0, 1.0);

    // Apply perceptual atmospheric softening to out-of-focus background surfaces
    let atmospheric_fog = vec3<f32>(0.005, 0.006, 0.008);
    let final_rgb = mix(surface_base, atmospheric_fog, dof_blur_factor * 0.35);

    let alpha = tex_sample.a * surface.opacity * surface.color_tint.a;
    return vec4<f32>(final_rgb, alpha);
}
