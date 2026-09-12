struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Full screen triangle generating coordinates covering [-1, 1]
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

struct VoidUniforms {
    time: f32,
    screen_width: f32,
    screen_height: f32,
    watermark_opacity: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: VoidUniforms;

// Distance to 2D line segment
fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Distance to circular arc
fn sd_arc(p: vec2<f32>, center: vec2<f32>, radius: f32, a0: f32, a1: f32) -> f32 {
    let d = p - center;
    let angle = atan2(d.y, d.x);
    let r_diff = abs(length(d) - radius);
    if (angle >= a0 && angle <= a1) {
        return r_diff;
    }
    let p0 = center + vec2<f32>(cos(a0), sin(a0)) * radius;
    let p1 = center + vec2<f32>(cos(a1), sin(a1)) * radius;
    return min(length(p - p0), length(p - p1));
}

// Procedural Signed Distance Field for monolithic modernist "ARC"
fn sd_arc_logo(p: vec2<f32>) -> f32 {
    // Letter 'A' (centered at x = -0.32)
    let a_left   = sd_segment(p, vec2<f32>(-0.40, -0.12), vec2<f32>(-0.32, 0.12));
    let a_right  = sd_segment(p, vec2<f32>(-0.32, 0.12), vec2<f32>(-0.24, -0.12));
    let a_bar    = sd_segment(p, vec2<f32>(-0.37, -0.03), vec2<f32>(-0.27, -0.03));
    let d_a = min(min(a_left, a_right), a_bar);

    // Letter 'R' (centered at x = 0.0)
    let r_spine  = sd_segment(p, vec2<f32>(-0.08, -0.12), vec2<f32>(-0.08, 0.12));
    let r_top    = sd_segment(p, vec2<f32>(-0.08, 0.12), vec2<f32>(0.03, 0.12));
    let r_mid    = sd_segment(p, vec2<f32>(-0.08, 0.01), vec2<f32>(0.03, 0.01));
    let r_loop   = sd_segment(p, vec2<f32>(0.03, 0.12), vec2<f32>(0.03, 0.01));
    let r_leg    = sd_segment(p, vec2<f32>(0.0, 0.01), vec2<f32>(0.08, -0.12));
    let d_r = min(min(min(min(r_spine, r_top), r_mid), r_loop), r_leg);

    // Letter 'C' (centered at x = 0.32)
    let c_spine  = sd_segment(p, vec2<f32>(0.24, -0.10), vec2<f32>(0.24, 0.10));
    let c_top    = sd_segment(p, vec2<f32>(0.24, 0.10), vec2<f32>(0.38, 0.10));
    let c_bottom = sd_segment(p, vec2<f32>(0.24, -0.10), vec2<f32>(0.38, -0.10));
    let d_c = min(min(c_spine, c_top), c_bottom);

    return min(min(d_a, d_r), d_c);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pure OLED Black Void base
    let base_black = vec3<f32>(0.005, 0.006, 0.008);

    // Normalized aspect-ratio corrected coordinates
    let aspect = uniforms.screen_width / uniforms.screen_height;
    var p = (in.uv - vec2<f32>(0.5, 0.44)) * vec2<f32>(aspect, 1.0);

    // Scale procedural typography to dominant central proportions
    p = p * 1.55;

    // Distance to ARC geometry
    let dist = sd_arc_logo(p);

    // Stroke definition with smooth anti-aliased edge
    let stroke_radius = 0.024;
    let core = smoothstep(stroke_radius + 0.004, stroke_radius - 0.002, dist);

    // Ethereal outer glow / bloom
    let glow = exp(-dist * 18.0) * 0.45;

    // Ambient breathing calculation
    // Peak bloom during boot (0.0 to 2.0s), then settling into a persistent ~5% pulsing watermark
    let t = uniforms.time;
    let bloom_in = smoothstep(0.0, 1.6, t);
    let bloom_decay = smoothstep(3.8, 1.6, t);
    let boot_peak = bloom_in * bloom_decay * 0.82;
    let ambient_pulse = (0.045 + 0.02 * sin(t * 1.4)) * uniforms.watermark_opacity;
    let effective_alpha = max(boot_peak, ambient_pulse);

    // Warm titanium white luminance
    let titanium = vec3<f32>(0.92, 0.90, 0.87);
    let final_color = base_black + titanium * (core * 0.9 + glow * 0.5) * effective_alpha;

    // Subtle edge vignette
    let vignette = 1.0 - length(in.uv - vec2<f32>(0.5, 0.5)) * 0.15;

    return vec4<f32>(final_color * vignette, 1.0);
}
