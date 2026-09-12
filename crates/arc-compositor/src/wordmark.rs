//! Watermark renderer: rasterizes the ARC wordmark from a real typeface
//! (URW Gothic Book — geometric modernist, per ADR-0002's native glyph
//! pipeline mandate) into a texture at boot, then composites it with the
//! cinematic fade curve. Replaces the hand-drawn SDF letterforms.

use bytemuck::Pod;
use bytemuck::Zeroable;
use glyphon::cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, SwashImage,
};

/// Rasterize `text` into a tightly-cropped R8 alpha texture.
/// Returns the texture, its dimensions, and the baseline-to-top offset used.
pub struct WordmarkRaster {
    pub texture: wgpu::Texture,
    pub width: u32,
    pub height: u32,
}

pub fn rasterize_wordmark(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    text: &str,
    px: f32,
) -> WordmarkRaster {
    let mut buffer = Buffer::new(font_system, Metrics::new(px, px * 1.2));
    buffer.set_size(Some(f32::MAX), Some(f32::MAX));
    buffer.set_text(
        text,
        &Attrs::new()
            .family(Family::Name("URW Gothic"))
            .weight(glyphon::cosmic_text::Weight::LIGHT),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);

    // Measure first: collect glyph placements relative to line origin.
    let mut placed: Vec<(SwashImage, f32, f32)> = Vec::new();
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for run in buffer.layout_runs() {
        let line_y = run.line_y;
        for glyph in run.glyphs.iter() {
            let physical = glyph.physical((0.0, 0.0), 1.0);
            if let Some(image) = swash_cache
                .get_image(font_system, physical.cache_key)
                .clone()
            {
                let gx = physical.x as f32 + image.placement.left as f32;
                let gy = line_y + physical.y as f32 + image.placement.top as f32;
                let gw = image.placement.width as f32;
                let gh = image.placement.height as f32;
                if gw == 0.0 || gh == 0.0 {
                    continue;
                }
                min_x = min_x.min(gx);
                min_y = min_y.min(gy);
                max_x = max_x.max(gx + gw);
                max_y = max_y.max(gy + gh);
                placed.push((image, gx, gy));
            }
        }
    }

    // Fallback: nothing rasterized (font missing) — caller gets a 1x1 texture.
    if placed.is_empty() {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Wordmark Empty"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(1),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        return WordmarkRaster {
            texture,
            width: 1,
            height: 1,
        };
    }

    let width = (max_x - min_x).ceil() as u32;
    let height = (max_y - min_y).ceil() as u32;

    // Pack coverage into the copy-aligned buffer, compositing glyph images.
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let stride = (width as usize + align - 1) / align * align;
    let mut pixels = vec![0u8; stride * height as usize];

    for (image, gx, gy) in &placed {
        let ox = (gx - min_x) as usize;
        let oy = (gy - min_y) as usize;
        let gw = image.placement.width as usize;
        let gh = image.placement.height as usize;
        for row in 0..gh {
            let dst_row = oy + row;
            if dst_row >= height as usize {
                continue;
            }
            for col in 0..gw {
                let dst_col = ox + col;
                if dst_col >= width as usize {
                    continue;
                }
                let src = &image.data[row * gw + col];
                // Max-composite coverage (glyphs never overlap meaningfully).
                let dst = &mut pixels[dst_row * stride + dst_col];
                *dst = (*dst).max(*src);
            }
        }
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ARC Wordmark"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(stride as u32),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    // Debug aid: ARC_DUMP_WORDMARK=1 writes the CPU-side coverage as PGM.
    if std::env::var("ARC_DUMP_WORDMARK").is_ok() {
        let mut pgm = format!("P5\n{} {}\n255\n", width, height).into_bytes();
        for row in 0..height as usize {
            pgm.extend_from_slice(&pixels[row * stride..row * stride + width as usize]);
        }
        let _ = std::fs::write("/tmp/arc_wordmark.pgm", &pgm);
    }

    WordmarkRaster {
        texture,
        width,
        height,
    }
}

/// Uniforms for the wordmark fade (matches watermark.wgsl).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct WatermarkUniforms {
    /// (x, y) of wordmark top-left, pixels.
    pub origin: [f32; 2],
    /// (width, height) at render scale, pixels.
    pub size: [f32; 2],
    /// (screen_w, screen_h).
    pub screen: [f32; 2],
    pub alpha: f32,
    pub _pad: [f32; 13], // total 16 floats = 64 bytes, matching WGSL incl. alignment
}
