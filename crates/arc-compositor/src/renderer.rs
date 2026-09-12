use std::sync::Arc;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::intent::IntentManager;
use crate::kinetics::{build_instances, GlyphAtlas, GlyphInstance, KineticLine};
use crate::reflex::ReflexEngine;
use crate::wordmark::{rasterize_wordmark, WatermarkUniforms, WordmarkRaster};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoidUniforms {
    pub time: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub watermark_opacity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    pub screen_size: [f32; 2],
    pub _pad: [f32; 2],
}

pub struct CanvasRenderer {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,

    // Void + wordmark pipeline (typeface-rasterized ARC, watermark.wgsl)
    void_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    wordmark: WordmarkRaster,
    wm_origin: [f32; 2],
    wm_size: [f32; 2],

    // Glyphon text engine
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    viewport: Viewport,

    // Kinetic per-glyph spring layer
    kinetic_line: KineticLine,
    glyph_atlas: GlyphAtlas,
    kinetics_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    kinetics_bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instance_capacity: u64,
    last_live_len: usize,
    /// Letter-by-letter visual stream of the reflex response (REQ-UX-002).
    response_stream: String,

    // Text buffers
    prompt_buffer: Buffer,
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

        // --- Text engine first: the wordmark rasterizes from the typeface ---
        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(&device, &cache);

        // ARC wordmark: URW Gothic Book (Futura-lineage geometric sans),
        // rasterized once at boot. Sized to ~55% of screen width.
        let wm_px = (height as f32) * 0.42;
        let wordmark = rasterize_wordmark(
            &device,
            &queue,
            &mut font_system,
            &mut swash_cache,
            "ARC",
            wm_px,
        );
        let wm_scale = (width as f32 * 0.55) / wordmark.width.max(1) as f32;
        let wm_w = wordmark.width as f32 * wm_scale;
        let wm_h = wordmark.height as f32 * wm_scale;
        let wm_origin = [(width as f32 - wm_w) * 0.5, (height as f32 - wm_h) * 0.5 - height as f32 * 0.06];

        // --- Void Shader Pipeline ---
        let void_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Obsidian Void & Wordmark Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/watermark.wgsl").into()),
        });

        let initial_uniforms = WatermarkUniforms {
            origin: wm_origin,
            size: [wm_w, wm_h],
            screen: [width as f32, height as f32],
            alpha: 0.0, // cinematic fade written per-frame
            _pad: [0.0; 13],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Void Uniform Buffer"),
            contents: bytemuck::cast_slice(&[initial_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let wordmark_view = wordmark.texture.create_view(&Default::default());
        let wordmark_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Wordmark Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Void Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Void Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&wordmark_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&wordmark_sampler),
                },
            ],
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
                module: &void_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &void_shader,
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

        // Text buffers — oversized literary display face for the prompt
        // (architecture §3.1: center third of the viewport), mono for status.
        // Glyphon TextArea scale is applied per-frame against these metrics;
        // base metrics sized for a 1440p-class canvas and scaled by resolution.
        let ui_scale = (height as f32 / 900.0).clamp(0.8, 2.0);
        let prompt_px = 72.0 * ui_scale;
        let prompt_buffer = Buffer::new(&mut font_system, Metrics::new(prompt_px, prompt_px * 1.25));
        let status_px = 18.0 * ui_scale;
        let status_buffer = Buffer::new(&mut font_system, Metrics::new(status_px, status_px * 1.45));

        // --- Kinetic Per-Glyph Spring Layer ---
        let glyph_atlas = GlyphAtlas::new(&device);
        let kinetic_line = KineticLine::new(64.0, width as f32 * 0.72);

        let kinetics_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kinetic Glyph Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/kinetics.wgsl").into()),
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Kinetic Camera Uniform"),
            contents: bytemuck::cast_slice(&[CameraUniforms {
                screen_size: [width as f32, height as f32],
                _pad: [0.0; 2],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Kinetic Camera Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let kinetics_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Kinetic Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let kinetics_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Kinetic Pipeline Layout"),
                bind_group_layouts: &[Some(&camera_layout), Some(&glyph_atlas.bind_group_layout)],
                immediate_size: 0,
            });

        let instance_capacity = 256u64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kinetic Instance Buffer"),
            size: instance_capacity * std::mem::size_of::<GlyphInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let glyph_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }, // pos
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 8,
                    shader_location: 1,
                }, // size
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 16,
                    shader_location: 2,
                }, // uv0
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 3,
                }, // uv1
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 40,
                    shader_location: 4,
                }, // alpha (scale at 32 unused by shader)
            ],
        };

        let kinetics_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Kinetic Render Pipeline"),
            layout: Some(&kinetics_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &kinetics_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(glyph_vertex_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &kinetics_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let mut this = Self {
            window,
            device,
            queue,
            surface,
            config,
            void_pipeline,
            uniform_buffer,
            uniform_bind_group,
            wordmark,
            wm_origin,
            wm_size: [wm_w, wm_h],
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            viewport,
            kinetic_line,
            glyph_atlas,
            kinetics_pipeline,
            camera_buffer,
            kinetics_bind_group,
            instance_buffer,
            instance_capacity,
            last_live_len: 0,
            response_stream: String::new(),
            prompt_buffer,
            status_buffer,
        };
        this.sync_kinetic_with_intent(&IntentManager::new());
        this
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[CameraUniforms {
                    screen_size: [width as f32, height as f32],
                    _pad: [0.0; 2],
                }]),
            );
        }
    }

    /// Mirror intent buffer diffs into the kinetic line (push/pop with springs).
    fn sync_kinetic_with_intent(&mut self, intent: &IntentManager) {
        // Backspace: live count shrank -> pop the last live glyph.
        let live_count = intent.user_input.chars().count();
        if live_count < self.last_live_len {
            for _ in 0..(self.last_live_len - live_count) {
                self.kinetic_line.pop();
            }
        } else if live_count > self.last_live_len {
            let chars: Vec<char> = intent.user_input.chars().collect();
            for &ch in &chars[self.last_live_len..live_count] {
                if ch == ' ' {
                    // Space advances the cursor without a visible glyph.
                    self.kinetic_line.caret_x += 18.0;
                    self.last_live_len += 1;
                    continue;
                }
                self.kinetic_line.push(
                    ch,
                    &mut self.font_system,
                    &mut self.swash_cache,
                    &self.device,
                    &self.queue,
                    &mut self.glyph_atlas,
                );
                self.last_live_len += 1;
            }
        }
        self.last_live_len = live_count;
    }

    pub fn render(
        &mut self,
        intent: &mut IntentManager,
        reflex: Option<&mut ReflexEngine>,
    ) -> Result<(), ()> {
        // Cinematic clock starts at first presented frame, not process spawn.
        intent.start_clock();
        self.sync_kinetic_with_intent(intent);

        // Reflex visual stream: append newly-arrived characters. The engine
        // paces tokens letter-by-letter; the canvas just mirrors the buffer.
        if let Some(engine) = reflex {
            engine.poll();
            if engine.response_so_far.len() != self.response_stream.len() {
                self.response_stream = engine.response_so_far.clone();
            }
        }

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
        let _ = elapsed_secs;
        let width = self.config.width as f32;
        let height = self.config.height as f32;

        let uniforms = WatermarkUniforms {
            origin: self.wm_origin,
            size: self.wm_size,
            screen: [width as f32, height as f32],
            alpha: 0.0,
            _pad: [0.0; 13],
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

        // 1. Prepare Welcome Prompt (oversized literary display face, center third)
        let visible_prompt = intent.visible_prompt();
        self.prompt_buffer.set_size(Some(width * 0.75), Some(height * 0.3));
        self.prompt_buffer.set_text(
            visible_prompt,
            &Attrs::new().family(Family::Serif).weight(Weight::MEDIUM),
            Shaping::Advanced,
            None,
        );

        // 2. Prepare Status Reflex Narrative — the letter-by-letter Arc
        // response, or the local acknowledgement while the query is in flight.
        let status_display = if !self.response_stream.is_empty() {
            self.response_stream.as_str()
        } else {
            intent.status_message.as_deref().unwrap_or("")
        };
        self.status_buffer.set_size(Some(width * 0.75), Some(height * 0.15));
        self.status_buffer.set_text(
            status_display,
            &Attrs::new().family(Family::Monospace).weight(Weight::LIGHT),
            Shaping::Advanced,
            None,
        );

        // Kinetic typography: user zero-input stream renders via spring layer,
        // not glyphon. Blinking caret appended as a live glyph-cell is
        // Milestone 0002 polish; caret position tracked here for the future.
        let dt = 1.0 / 120.0;
        let kinetic_alive = self.kinetic_line.step(dt);
        let mut instances: Vec<GlyphInstance> = Vec::with_capacity(128);
        build_instances(
            &self.kinetic_line,
            [width * 0.14, height * 0.52],
            1024.0,
            &mut instances,
        );
        let byte_len = (instances.len() * std::mem::size_of::<GlyphInstance>()) as u64;
        if !instances.is_empty() {
            debug_assert!(
                byte_len <= self.instance_capacity * std::mem::size_of::<GlyphInstance>() as u64,
                "kinetic instance overflow"
            );
            self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }

        let text_areas = [
            TextArea {
                buffer: &self.prompt_buffer,
                left: width * 0.5 - width * 0.375, // centered within the 0.75 span
                top: height * 0.44,
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
                buffer: &self.status_buffer,
                left: width * 0.14,
                top: height * 0.66,
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

            // 2. Draw Kinetic Per-Glyph Spring Layer
            if !instances.is_empty() {
                render_pass.set_pipeline(&self.kinetics_pipeline);
                render_pass.set_bind_group(0, &self.kinetics_bind_group, &[]);
                render_pass.set_bind_group(1, &self.glyph_atlas.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..byte_len));
                render_pass.draw(0..6, 0..instances.len() as u32);
            }

            // 3. Draw Glyphon Typography Overlays
            let _ = kinetic_alive; // redraws are continuous via about_to_wait
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


fn smoothstep_01(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
