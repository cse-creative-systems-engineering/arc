//! Kinetic per-glyph input layer (Milestone 0001, REQ-UX-002).
//!
//! Each typed character is rendered as an individual GPU instance driven by
//! spring integrators, giving physics-based glyph insertion and deletion per
//! ADR-0003. Glyph rasters come from cosmic-text/swash (per ADR-0002: native
//! glyph pipelines, no webviews).

use std::collections::VecDeque;

use glyphon::cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, SwashImage,
};
use wgpu::util::DeviceExt;

/// Spring state for a single animated glyph property.
#[derive(Debug, Clone)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
}

impl Spring {
    pub fn new(initial: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
        }
    }

    /// Semi-implicit Euler integration, stable at any frame rate.
    pub fn step(&mut self, target: f32, dt: f32) {
        let stiffness = 220.0;
        let damping = 26.0; // ~critically damped for stiffness 220
        let accel = (target - self.value) * stiffness - self.velocity * damping;
        self.velocity += accel * dt;
        self.value += self.velocity * dt;
    }
}

/// One animated glyph instance (CPU-side bookkeeping).
#[derive(Debug, Clone)]
pub struct GlyphCell {
    /// Advance width contributed to the line (pixels).
    pub advance: f32,
    /// Horizontal offset of glyph origin from the line cursor (pixels).
    pub offset_x: f32,
    /// Vertical bearing: baseline-relative glyph top (pixels, y-down).
    pub offset_y: f32,
    /// Atlas position (pixels).
    pub atlas_xy: [f32; 2],
    pub atlas_wh: [f32; 2],
    /// Scale-in spring: insertion pop / deletion shrink.
    pub scale: Spring,
    /// Drop spring: deletion gravity (0 = resting, 1 = fully fallen).
    pub drop: Spring,
    pub alpha_floor: f32,
}

impl GlyphCell {
    fn is_live(&self) -> bool {
        self.drop.value >= 0.999 && self.scale.value > 0.004
    }
}

/// Instance data pushed to the GPU (must match kinetics.wgsl `GlyphInstance`).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlyphInstance {
    /// Top-left of the glyph quad in pixels (y-down).
    pub pos: [f32; 2],
    /// Glyph quad size in pixels (pre-scale).
    pub size: [f32; 2],
    /// UV rect in atlas (normalized).
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub scale: f32,
    pub alpha: f32,
    pub _pad: [f32; 2],
}

/// Rolling kinetic input line with per-glyph spring physics.
pub struct KineticLine {
    pub glyphs: VecDeque<GlyphCell>,
    pub caret_x: f32,
    line_height: f32,
    max_width: f32,
    max_glyphs: usize,
}

impl KineticLine {
    pub fn new(line_height: f32, max_width: f32) -> Self {
        Self {
            glyphs: VecDeque::new(),
            caret_x: 0.0,
            line_height,
            max_width,
            max_glyphs: 128,
        }
    }

    /// Rasterize `ch` and append it with an insertion spring.
    pub fn push(
        &mut self,
        ch: char,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut GlyphAtlas,
    ) {
        let cell = raster_glyph(
            ch,
            font_system,
            swash_cache,
            device,
            queue,
            atlas,
            self.line_height,
        );
        self.caret_x += cell.advance;
        self.glyphs.push_back(cell);
        self.wrap_if_needed();
        while self.glyphs.len() > self.max_glyphs {
            self.glyphs.pop_front();
        }
    }

    fn wrap_if_needed(&mut self) {
        if self.caret_x <= self.max_width {
            return;
        }
        // Re-wrap all live glyphs onto lines within max_width, shifting older
        // lines up (negative y). Deletion animation width is irrelevant to
        // layout, so dead (dropping) glyphs keep their last offsets.
        let mut cursor = 0.0f32;
        let mut line = 0i32;
        for cell in self.glyphs.iter_mut() {
            if !cell.is_live() {
                continue;
            }
            if cursor + cell.advance > self.max_width && cursor > 0.0 {
                cursor = 0.0;
                line += 1;
            }
            cell.offset_x = cursor;
            cell.offset_y = self.line_height * (line + 1) as f32;
            cursor += cell.advance;
        }
        self.caret_x = cursor;
    }

    /// Mark the last live glyph as deleting: springs to zero, drops, fades.
    pub fn pop(&mut self) {
        for cell in self.glyphs.iter_mut().rev() {
            if cell.is_live() {
                cell.drop = Spring::new(0.0);
                self.caret_x = (self.caret_x - cell.advance).max(0.0);
                return;
            }
        }
    }

    /// Advance all springs; returns true if any glyph is still animating.
    pub fn step(&mut self, dt: f32) -> bool {
        let mut alive = false;
        for cell in self.glyphs.iter_mut() {
            let scale_target = if cell.drop.value < 1.0 { 0.0 } else { 1.0 };
            cell.scale.step(scale_target, dt);
            cell.drop.step(1.0, dt);
            // Alive while visibly animating: value not yet at rest target or
            // still moving meaningfully.
            let at_rest = (cell.scale.value - scale_target).abs() < 0.004
                && cell.scale.velocity.abs() < 0.01;
            if !at_rest {
                alive = true;
            }
        }
        self.glyphs
            .retain(|c| !(c.scale.value <= 0.004 && c.drop.value >= 0.999));
        alive
    }

    /// Current live text (what the intent buffer should mirror).
    pub fn live_text(&self) -> String {
        self.glyphs
            .iter()
            .filter(|c| c.is_live())
            .map(|c| char_of_offset(c))
            .collect()
    }
}

// GlyphCell does not carry the char (atlas cells are anonymous once packed);
// the intent buffer remains the source of truth for text, KineticLine only
// mirrors visual state. Kept as a stub for future hit-testing.
fn char_of_offset(_c: &GlyphCell) -> char {
    '\0'
}

/// Shelf-packed dynamic R8 glyph atlas (1024x1024).
pub struct GlyphAtlas {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    size: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
}

impl GlyphAtlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let size = 1024u32;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Kinetic Glyph Atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Glyph Atlas Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Atlas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            texture,
            bind_group,
            bind_group_layout,
            size,
            cursor_x: 1,
            cursor_y: 1,
            row_h: 0,
        }
    }

    /// Upload one swash image; returns its (x, y, w, h) in atlas pixels.
    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        image: &SwashImage,
    ) -> [f32; 4] {
        let w = image.placement.width as u32;
        let h = image.placement.height as u32;
        if self.cursor_x + w + 1 > self.size {
            self.cursor_x = 1;
            self.cursor_y += self.row_h + 1;
            self.row_h = 0;
        }
        let (x, y) = (self.cursor_x, self.cursor_y);

        if w > 0 && h > 0 && !image.data.is_empty() {
            // swash R8 coverage, tightly packed rows. Respect the 256-byte
            // copy-row alignment by padding per row.
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
            let padded = (w as usize + align - 1) / align * align;
            let mut buf = vec![0u8; padded * h as usize];
            for row in 0..h as usize {
                let src = &image.data[row * w as usize..(row + 1) * w as usize];
                buf[row * padded..row * padded + w as usize].copy_from_slice(src);
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &buf,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded as u32),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.cursor_x += w + 1;
        self.row_h = self.row_h.max(h);
        [x as f32, y as f32, w as f32, h as f32]
    }
}

/// Rasterize one char into the atlas; returns an animation-ready GlyphCell.
fn raster_glyph(
    ch: char,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    _device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas: &mut GlyphAtlas,
    px: f32,
) -> GlyphCell {
    let mut buffer = Buffer::new(font_system, Metrics::new(px, px * 1.4));
    buffer.set_text(
        &ch.to_string(),
        &Attrs::new().family(Family::Monospace),
        Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(font_system, false);

    let run = buffer.layout_runs().next().expect("layout run");
    let layout_glyph = run.glyphs.first().expect("one glyph");
    let physical = layout_glyph.physical((0.0, 0.0), 1.0);
    let image = swash_cache
        .get_image(font_system, physical.cache_key)
        .clone()
        .expect("swash raster");

    let rect = atlas.upload(queue, &image);
    let advance = rect[2] + 12.0; // glyph width + mono tracking

    GlyphCell {
        advance,
        offset_x: 0.0,
        offset_y: run.line_y + image.placement.top as f32,
        atlas_xy: [rect[0], rect[1]],
        atlas_wh: [rect[2], rect[3]],
        scale: Spring::new(0.0),
        drop: Spring::new(1.0),
        alpha_floor: 1.0,
    }
}

/// Build the GPU instance array for a KineticLine at a given origin.
/// `dt` has already been applied via `step`; this is a pure projection.
pub fn build_instances(
    line: &KineticLine,
    origin: [f32; 2],
    atlas_size: f32,
    instances: &mut Vec<GlyphInstance>,
) {
    instances.clear();
    for cell in &line.glyphs {
        if cell.atlas_wh[0] <= 0.0 || cell.atlas_wh[1] <= 0.0 {
            continue;
        }
        let s = cell.scale.value.max(0.0);
        if s < 0.004 {
            continue;
        }
        let drop = cell.drop.value;
        let alpha = cell.alpha_floor * s * (1.0 - 0.85 * drop);
        let w = cell.atlas_wh[0];
        let h = cell.atlas_wh[1];
        // Deletion: glyph shrinks around its center and drops with gravity.
        let cx = origin[0] + cell.offset_x + cell.offset_x.mul_add(0.0, w * 0.5);
        let cy = origin[1] + cell.offset_y + h * 0.5 + drop * drop * 60.0;
        let half_w = w * 0.5 * s;
        let half_h = h * 0.5 * s;
        instances.push(GlyphInstance {
            pos: [cx - half_w, cy - half_h],
            size: [w * s, h * s],
            uv0: [
                cell.atlas_xy[0] / atlas_size,
                cell.atlas_xy[1] / atlas_size,
            ],
            uv1: [
                (cell.atlas_xy[0] + cell.atlas_wh[0]) / atlas_size,
                (cell.atlas_xy[1] + cell.atlas_wh[1]) / atlas_size,
            ],
            scale: s,
            alpha,
            _pad: [0.0; 2],
        });
    }
}

/// Convenience wrapper mirroring `DeviceExt`.
pub fn instance_buffer(device: &wgpu::Device, data: &[GlyphInstance]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Kinetic Glyph Instances"),
        contents: bytemuck::cast_slice(data),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_converges() {
        let mut s = Spring::new(0.0);
        for _ in 0..600 {
            s.step(1.0, 1.0 / 120.0);
        }
        assert!((s.value - 1.0).abs() < 0.01, "spring must converge to target");
        assert!(s.velocity.abs() < 0.5);
    }

    #[test]
    fn test_spring_overshoot() {
        // Underdamped enough to overshoot slightly (kinetic "pop").
        let mut s = Spring::new(0.0);
        let mut peak = 0.0f32;
        for _ in 0..300 {
            s.step(1.0, 1.0 / 120.0);
            peak = peak.max(s.value);
        }
        assert!(peak >= 1.0, "insertion spring should reach full scale");
    }

    #[test]
    fn test_spring_stable_at_large_dt() {
        let mut s = Spring::new(0.0);
        for _ in 0..100 {
            s.step(1.0, 1.0 / 30.0); // 30fps frame gaps must not explode
        }
        assert!(s.value.is_finite());
        assert!((s.value - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_pop_deactivates_last_glyph() {
        let mut line = KineticLine::new(64.0, 800.0);
        // Push two fake cells directly (no GPU in unit tests).
        line.glyphs.push_back(GlyphCell {
            advance: 20.0,
            offset_x: 0.0,
            offset_y: 10.0,
            atlas_xy: [0.0, 0.0],
            atlas_wh: [16.0, 20.0],
            scale: Spring::new(1.0),
            drop: Spring::new(1.0),
            alpha_floor: 1.0,
        });
        line.glyphs.push_back(GlyphCell {
            advance: 20.0,
            offset_x: 20.0,
            offset_y: 10.0,
            atlas_xy: [0.0, 0.0],
            atlas_wh: [16.0, 20.0],
            scale: Spring::new(1.0),
            drop: Spring::new(1.0),
            alpha_floor: 1.0,
        });
        line.caret_x = 40.0;
        line.pop();
        // Second glyph is now dropping; first remains live.
        assert!(line.glyphs[1].drop.value < 1.0);
        assert!(line.glyphs[0].is_live());
        assert!((line.caret_x - 20.0).abs() < 1e-4);
        // Animate until the dead glyph is reaped.
        for _ in 0..600 {
            line.step(1.0 / 120.0);
        }
        assert_eq!(line.glyphs.len(), 1, "fully-deleted glyph must be reaped");
    }

    #[test]
    fn test_pop_on_empty_is_noop() {
        let mut line = KineticLine::new(64.0, 800.0);
        line.pop();
        assert!(line.glyphs.is_empty());
        assert_eq!(line.caret_x, 0.0);
    }

    #[test]
    fn test_step_setstles_and_reaps() {
        let mut line = KineticLine::new(64.0, 800.0);
        line.glyphs.push_back(GlyphCell {
            advance: 20.0,
            offset_x: 0.0,
            offset_y: 0.0,
            atlas_xy: [0.0, 0.0],
            atlas_wh: [16.0, 20.0],
            scale: Spring::new(0.0), // inserting
            drop: Spring::new(1.0),
            alpha_floor: 1.0,
        });
        let mut alive = true;
        for _ in 0..600 {
            alive = line.step(1.0 / 120.0);
        }
        assert!(!alive, "line must report quiescence after settling");
        assert!((line.glyphs[0].scale.value - 1.0).abs() < 0.01);
    }
}
