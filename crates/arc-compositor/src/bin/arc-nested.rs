//! Nested Smithay compositor spike (Milestone 0001 deliverable).
//!
//! Boots a real Smithay display server nested inside the current desktop
//! session (ADR-0001), rendering the Arc obsidian void via the Smithay
//! GLES renderer. Keyboard events feed the Global Intent Buffer (ADR-0003);
//! the full kinetic typography port rides on the surface-element integration
//! in Milestone 0002.

use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::{Color32F, Frame, Renderer, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    reexports::winit::platform::pump_events::PumpStatus,
    utils::Rectangle,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arc_compositor=info".into()),
        )
        .init();

    tracing::info!("Arc nested Smithay spike starting (Milestone 0001)...");

    let (mut backend, mut winit_loop) = winit::init::<GlesRenderer>()?;
    tracing::info!("Nested compositor online. Rendering obsidian void.");

    loop {
        let status = winit_loop.dispatch_new_events(|event| match event {
            WinitEvent::Input(InputEvent::Keyboard { event }) => {
                // Raw scancode path — xkbcommon seat translation and the
                // Global Intent Buffer wiring land with the seat integration.
                let _: smithay::backend::input::Keycode = event.key_code();
                let _: smithay::backend::input::KeyState = event.state();
            }
            _ => {}
        });

        match status {
            PumpStatus::Continue => (),
            PumpStatus::Exit(_) => {
                tracing::info!("Nested compositor stopping.");
                return Ok(());
            }
        }

        let size = backend.window_size();
        let damage = Rectangle::from_size(size);

        {
            let (renderer, mut framebuffer) = backend.bind()?;
            let mut frame =
                renderer.render(&mut framebuffer, size, smithay::utils::Transform::Flipped180)?;
            // Pure OLED void — procedural watermark shader port follows with
            // the GlesTexRenderElement integration in Milestone 0002.
            frame.clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[damage])?;
            frame.finish()?;
        }

        backend.submit(Some(&[damage]))?;
    }
}
