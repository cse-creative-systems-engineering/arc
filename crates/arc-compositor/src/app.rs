use std::sync::Arc;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::intent::IntentManager;
use crate::renderer::CanvasRenderer;

pub struct ArcApp {
    window: Option<Arc<Window>>,
    renderer: Option<CanvasRenderer>,
    intent: IntentManager,
}

impl ArcApp {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            intent: IntentManager::new(),
        }
    }
}

impl ApplicationHandler for ArcApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!("Initializing Arc Compositor Window & WGPU Substrate...");

            let window_attrs = WindowAttributes::default()
                .with_title("Arc — Autonomous Operating Substrate")
                .with_inner_size(LogicalSize::new(1440.0, 900.0))
                .with_min_inner_size(LogicalSize::new(800.0, 600.0))
                .with_active(true);

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create native compositor test window"),
            );

            let renderer = pollster::block_on(CanvasRenderer::new(window.clone()));

            self.window = Some(window);
            self.renderer = Some(renderer);
            info!("Arc Display Substrate online. Entering 120Hz/V-Sync render loop.");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Arc Compositor shutdown requested by user.");
                event_loop.exit();
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(physical_size.width, physical_size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    let _ = renderer.render(&self.intent);
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        text,
                        ..
                    },
                ..
            } => {
                match logical_key {
                    Key::Named(NamedKey::Escape) => {
                        self.intent.clear();
                    }
                    Key::Named(NamedKey::Backspace) => {
                        self.intent.pop_char();
                    }
                    Key::Named(NamedKey::Enter) => {
                        self.intent.commit();
                    }
                    Key::Named(NamedKey::Space) => {
                        self.intent.push_char(' ');
                    }
                    Key::Character(s) => {
                        for c in s.chars() {
                            self.intent.push_char(c);
                        }
                    }
                    _ => {
                        if let Some(txt) = text {
                            for c in txt.chars() {
                                self.intent.push_char(c);
                            }
                        }
                    }
                }

                if let Some(renderer) = &self.renderer {
                    renderer.window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            // Keep the kinetic typography and procedural bloom breathing continuously
            renderer.window.request_redraw();
        }
    }
}
