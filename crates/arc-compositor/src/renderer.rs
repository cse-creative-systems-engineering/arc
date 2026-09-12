use std::sync::Arc;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::intent::IntentManager;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoidUniforms {
    pub time: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub watermark_opacity: f32,
}

pub struct CanvasRenderer {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,

    // Void pipeline
    void_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    // Glyphon text engine
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    viewport: Viewport,

    // Text buffers
    prompt_buffer: Buffer,
    input_buffer: Buffer,
    status_buffer: Buffer,
}

impl CanvasRenderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create WGPU surface from window");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Arc Compositor Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create WGPU device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // --- Void Shader Pipeline ---
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Obsidian Void & Watermark Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/void.wgsl").into()),
        });

        let initial_uniforms = VoidUniforms {
            time: 0.0,
            screen_width: width as f32,
            screen_height: height as f32,
            watermark_opacity: 1.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Void Uniform Buffer"),
            contents: bytemuck::cast_slice(&[initial_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Void Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Void Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Void Pipeline Layout"),
            bind_group_layouts: &[Some(&uniform_bind_group_layout)],
            immediate_size: 0,
        });

        let void_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Void Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // --- Glyphon Text Engine ---
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(&device, &cache);

        // Text buffers
        let prompt_buffer = Buffer::new(&mut font_system, Metrics::new(32.0, 44.0));
        let input_buffer = Buffer::new(&mut font_system, Metrics::new(26.0, 36.0));
        let status_buffer = Buffer::new(&mut font_system, Metrics::new(18.0, 26.0));

        Self {
            window,
            device,
            queue,
            surface,
            config,
            void_pipeline,
            uniform_buffer,
            uniform_bind_group,
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            viewport,
            prompt_buffer,
            input_buffer,
            status_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, intent: &IntentManager) -> Result<(), ()> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex) => tex,
            wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                return Ok(());
            }
            _ => return Ok(()),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let elapsed_secs = intent.elapsed().as_secs_f32();
        let width = self.config.width as f32;
        let height = self.config.height as f32;

        // Update void uniforms
        let uniforms = VoidUniforms {
            time: elapsed_secs,
            screen_width: width,
            screen_height: height,
            watermark_opacity: 1.0,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Update text viewport
        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );

        // 1. Prepare Welcome Prompt
        let visible_prompt = intent.visible_prompt();
        self.prompt_buffer.set_size(
            Some(width * 0.75),
            Some(height * 0.3),
        );
        self.prompt_buffer.set_text(
            visible_prompt,
            &Attrs::new()
                .family(Family::Serif)
                .weight(Weight::MEDIUM),
            Shaping::Advanced,
            None,
        );

        // 2. Prepare User Zero-Input Text
        let mut user_display = intent.user_input.clone();
        if intent.is_prompt_complete() && !intent.is_committed {
            // Blinking caret
            if (elapsed_secs * 2.5) as u32 % 2 == 0 {
                user_display.push('│');
            }
        }
        self.input_buffer.set_size(
            Some(width * 0.75),
            Some(height * 0.2),
        );
        self.input_buffer.set_text(
            &user_display,
            &Attrs::new()
                .family(Family::Monospace)
                .weight(Weight::NORMAL),
            Shaping::Advanced,
            None,
        );

        // 3. Prepare Status Reflex Narrative
        let status_display = intent.status_message.as_deref().unwrap_or("");
        self.status_buffer.set_size(
            Some(width * 0.75),
            Some(height * 0.15),
        );
        self.status_buffer.set_text(
            status_display,
            &Attrs::new()
                .family(Family::Monospace)
                .weight(Weight::LIGHT),
            Shaping::Advanced,
            None,
        );

        // Center calculation
        let center_x = width * 0.14;
        let prompt_y = height * 0.44;
        let input_y = prompt_y + 60.0;
        let status_y = input_y + 50.0;

        let text_areas = [
            TextArea {
                buffer: &self.prompt_buffer,
                left: center_x,
                top: prompt_y,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: Color::rgb(238, 235, 230),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &self.input_buffer,
                left: center_x,
                top: input_y,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: Color::rgb(142, 202, 230),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &self.status_buffer,
                left: center_x,
                top: status_y,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: Color::rgb(163, 190, 140),
                custom_glyphs: &[],
            },
        ];

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .expect("Failed to prepare text rendering");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Canvas Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Canvas Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // 1. Draw Obsidian Void & Procedural ARC Watermark
            render_pass.set_pipeline(&self.void_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            // 2. Draw Kinetic Typography Overlays
            self.text_renderer
                .render(&self.text_atlas, &self.viewport, &mut render_pass)
                .expect("Failed to render text pass");
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);

        self.text_atlas.trim();

        Ok(())
    }
}
