//! Nested Smithay compositor (Milestone 0001 deliverable, ADR-0001).
//!
//! Boots a real Smithay Wayland display server nested inside the current
//! desktop session, with a fully wired keyboard seat: raw winit keycodes
//! flow through the Smithay keyboard state machine (xkbcommon keymap,
//! modifiers, repeat) and land as Unicode characters in the Global Intent
//! Buffer (ADR-0003 — the display surface is the input buffer).
//!
//! Renders the obsidian void via the Smithay GLES renderer. The kinetic
//! canvas ports onto renderer elements in Milestone 0002.

use smithay::{
    backend::{
        input::{Event, InputEvent, KeyState, KeyboardKeyEvent},
        renderer::{gles::GlesRenderer, Color32F, Frame, Renderer},
        winit::{self, WinitEvent},
    },
    input::{keyboard::FilterResult, Seat, SeatHandler, SeatState},
    reexports::winit::platform::pump_events::PumpStatus,
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::wl_surface::WlSurface,
        Client, Display,
    },
    utils::Rectangle,
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
    delegate_compositor,
    delegate_seat,
};

use arc_compositor::intent::IntentManager;

struct App {
    compositor_state: CompositorState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    /// Characters resolved through the seat keymap this frame.
    typed: Vec<char>,
    pending_enter: bool,
    pending_backspace: bool,
    pending_escape: bool,
}

impl SeatHandler for App {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
}

impl CompositorHandler for App {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, _surface: &WlSurface) {}
}

delegate_compositor!(App);
delegate_seat!(App);

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arc_compositor=info".into()),
        )
        .init();

    tracing::info!("Arc nested Smithay compositor starting (Milestone 0001)...");

    let display: Display<App> = Display::new()?;
    let dh = display.handle();
    let compositor_state = CompositorState::new::<App>(&dh);
    let mut seat_state = SeatState::new();
    let seat = seat_state.new_wl_seat(&dh, "arc");

    let mut state = App {
        compositor_state,
        seat_state,
        seat,
        typed: Vec::new(),
        pending_enter: false,
        pending_backspace: false,
        pending_escape: false,
    };

    let keyboard = state.seat.add_keyboard(Default::default(), 200, 200)?;
    let _listener_socket = None::<()>; // no Wayland clients yet — M0002 wires the socket

    let (mut backend, mut winit_loop) = winit::init::<GlesRenderer>()?;
    tracing::info!("Nested compositor online. Zero-input canvas listening.");

    let mut intent = IntentManager::new();

    loop {
        let status = winit_loop.dispatch_new_events(|event| match event {
            WinitEvent::Input(InputEvent::Keyboard { event }) => {
                // Only act on key-down; key-ups flow through the seat for
                // client state consistency.
                if event.state() == KeyState::Pressed {
                    let keycode = event.key_code();
                    // Route through the seat keymap: xkbcommon resolves the
                    // layout-aware keysym so we get real characters, not
                    // raw scancodes (REQ-UX-001, ADR-0003).
                    keyboard.input(
                        &mut state,
                        keycode,
                        KeyState::Pressed,
                        0.into(),
                        event.time() as u32,
                        |_state, _mods, keysym| {
                            use xkbcommon::xkb::keysym_to_utf8;
                            let sym = keysym.modified_sym();
                            let utf8 = keysym_to_utf8(sym);
                            match sym.raw() {
                                0xff1b => {
                                    // Escape
                                    _state.pending_escape = true;
                                }
                                0xff08 => {
                                    // Backspace
                                    _state.pending_backspace = true;
                                }
                                0xff0d | 0xff8d => {
                                    // Return / KP_Enter
                                    _state.pending_enter = true;
                                }
                                _ => {
                                    for ch in utf8.chars() {
                                        if !ch.is_control() {
                                            _state.typed.push(ch);
                                        }
                                    }
                                }
                            }
                            // No Wayland clients yet — intercept everything
                            // so keystrokes feed the Global Intent Buffer.
                            FilterResult::Intercept(())
                        },
                    );
                }
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

        // Drain resolved keystrokes into the Global Intent Buffer.
        for ch in state.typed.drain(..) {
            intent.push_char(ch);
        }
        if state.pending_backspace {
            state.pending_backspace = false;
            intent.pop_char();
        }
        if state.pending_enter {
            state.pending_enter = false;
            intent.commit();
            if let Some(q) = intent.committed_query() {
                tracing::info!("Intent committed: {q:?}");
            }
        }
        if state.pending_escape {
            state.pending_escape = false;
            intent.clear();
        }

        let size = backend.window_size();
        let damage = Rectangle::from_size(size);

        {
            let (renderer, mut framebuffer) = backend.bind()?;
            let mut frame = renderer.render(
                &mut framebuffer,
                size,
                smithay::utils::Transform::Flipped180,
            )?;
            // Obsidian void. The kinetic canvas (kinetics.rs + watermark)
            // renders as GLES elements in the Milestone 0002 port.
            frame.clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[damage])?;
            let _ = frame.finish()?;
        }

        backend.submit(Some(&[damage]))?;
    }
}
