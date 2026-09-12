mod app;
mod intent;
mod renderer;

use app::ArcApp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arc_compositor=info".into()),
        )
        .init();

    tracing::info!("Starting Arc Compositor Substrate (Milestone 0001)...");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ArcApp::new();
    event_loop.run_app(&mut app)?;

    tracing::info!("Arc Compositor Substrate cleanly stopped.");
    Ok(())
}
