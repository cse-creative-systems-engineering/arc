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

// Distance to circular arc. Angles may span the ±π wrap: the given angle is
// normalized into [a0, a0+2π) before the containment test.
fn sd_arc(p: vec2<f32>, center: vec2<f32>, radius: f32, a0: f32, a1: f32) -> f32 {
    let d = p - center;
    var angle = atan2(d.y, d.x);
    if (angle < a0) {
        angle = angle + 6.28318530718;
    }
    let r_diff = abs(length(d) - radius);
    if (angle >= a0 && angle <= a1) {
        return r_diff;
    }
    let p0 = center + vec2<f32>(cos(a0), sin(a0)) * radius;
    let p1 = center + vec2<f32>(cos(a1), sin(a1)) * radius;
    return min(length(p - p0), length(p - p1));
}

// Procedural Signed Distance Field for the ARC wordmark.
// Editorial proportions: cap height y ∈ [-0.14, 0.14], hairline stroke,
// true circular bowls (no stick-figure segments).
fn sd_arc_logo(p: vec2<f32>) -> f32 {
    // ── Letter 'A' (apex at x = -0.40) ──
    // Legs splay elegantly; apex has a tiny flat top for optical crispness.
    let a_left  = sd_segment(p, vec2<f32>(-0.52, -0.14), vec2<f32>(-0.405, 0.135));
    let a_right = sd_segment(p, vec2<f32>(-0.405, 0.135), vec2<f32>(-0.29, -0.14));
    // Crossbar sits slightly low (optically centered), spanning leg to leg.
    let a_bar   = sd_segment(p, vec2<f32>(-0.474, -0.03), vec2<f32>(-0.326, -0.03));
    let d_a = min(min(a_left, a_right), a_bar);

    // ── Letter 'R' (spine at x = -0.10, bowl right edge x = 0.16) ──
    // Spine carries full cap height; bowl is a true semicircle; leg kicks
    // out from the bowl's lower quadrant.
    let r_spine = sd_segment(p, vec2<f32>(-0.10, -0.14), vec2<f32>(-0.10, 0.14));
    let r_top   = sd_segment(p, vec2<f32>(-0.10, 0.14), vec2<f32>(0.02, 0.14));
    let r_bowl  = sd_arc(p, vec2<f32>(0.02, 0.0), 0.14, -1.5707963, 1.5707963);
    let r_mid   = sd_segment(p, vec2<f32>(-0.10, 0.0), vec2<f32>(0.09, 0.0));
    let r_leg   = sd_segment(p, vec2<f32>(0.055, -0.02), vec2<f32>(0.155, -0.14));
    let d_r = min(min(min(min(r_spine, r_top), r_bowl), r_mid), r_leg);

    // ── Letter 'C' (center (0.36, 0), radius 0.14, opening facing right) ──
    // A genuine circular arc, tips cut at ±55° with small horizontal
    // terminals for a modernist finish.
    let c_body  = sd_arc(p, vec2<f32>(0.36, 0.0), 0.14, 0.9599, 5.3233);
    let c_tip_t = sd_segment(p, vec2<f32>(0.36 + 0.14 * 0.5736, 0.14 * 0.8192), vec2<f32>(0.36 + 0.14 * 0.5736 - 0.055, 0.14 * 0.8192));
    let c_tip_b = sd_segment(p, vec2<f32>(0.36 + 0.14 * 0.5736, -0.14 * 0.8192), vec2<f32>(0.36 + 0.14 * 0.5736 - 0.055, -0.14 * 0.8192));
    let d_c = min(min(c_body, c_tip_t), c_tip_b);

    return min(min(d_a, d_r), d_c);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pure OLED Black Void base
    let base_black = vec3<f32>(0.005, 0.006, 0.008);

    // Normalized aspect-ratio corrected coordinates
    let aspect = uniforms.screen_width / uniforms.screen_height;
    var p = (in.uv - vec2<f32>(0.5, 0.44)) * vec2<f32>(aspect, 1.0);
    // UV y grows downward; the SDF letterforms are authored y-up.
    p = vec2<f32>(p.x, -p.y);

    // Scale procedural typography to dominant central proportions
    p = p * 1.55;

    // Distance to ARC geometry
    let dist = sd_arc_logo(p);

    // Hairline stroke — thin, precise, editorial. Narrow AA band keeps the
    // line crisp at 1px-equivalent weight.
    let stroke_radius = 0.004;
    let core = smoothstep(stroke_radius + 0.0015, stroke_radius - 0.0015, dist);

    // Ethereal outer glow / bloom (subtle — the hairline carries the form)
    let glow = exp(-dist * 60.0) * 0.28;

    // Ambient breathing calculation
    // Cinematic entry: 3s of PURE darkness (all luminance gated to zero),
    // then a very slow ~12s bloom from nothing, a held peak, then a glacial
    // decay to the breathing watermark.
    let t = uniforms.time;
    let darkness = smoothstep(3.0, 3.6, t); // hard 0 until t=3s
    let bloom_in = smoothstep(3.0, 15.0, t);
    let bloom_decay = smoothstep(27.0, 15.0, t);
    let boot_peak = bloom_in * bloom_decay * 0.82;
    let ambient_pulse = (0.045 + 0.015 * sin(t * 0.35)) * uniforms.watermark_opacity;
    let effective_alpha = max(boot_peak, ambient_pulse) * darkness;

    // Warm titanium white luminance
    let titanium = vec3<f32>(0.92, 0.90, 0.87);
    let final_color = base_black + titanium * (core * 0.9 + glow * 0.5) * effective_alpha;

    // Subtle edge vignette
    let vignette = 1.0 - length(in.uv - vec2<f32>(0.5, 0.5)) * 0.15;

    return vec4<f32>(final_color * vignette, 1.0);
}
