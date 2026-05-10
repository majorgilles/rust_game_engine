//! Chapter 1: opening an OS window and running an event loop.
//!
//! No GPU here yet — this chapter is just about getting a window on screen and
//! reacting to events (resize, redraw, keyboard, close). Chapter 2 plugs wgpu
//! into the same skeleton.
//!
//! # The event-loop model
//!
//! Modern desktop apps are *event-driven*: instead of running top-to-bottom,
//! they hand control to the OS and wake up only when something happens — the
//! user moves the mouse, resizes the window, the OS asks for a redraw. winit
//! is the cross-platform library that gives us that loop.
//!
//! The flow:
//!
//!   `EventLoop::new()`  ─►  `run_app(&mut App)`  ─►  callbacks fire forever
//!     - `resumed`        : create the window (and later, GPU state)
//!     - `window_event`   : react to resize / redraw / input / close
//!     - (`suspended`)    : tear things down on mobile when backgrounded
//!
//! # Why a separate `State` and `App`?
//!
//! - `App` is the *handler* winit talks to; it must exist before the event loop
//!   starts, but the OS window doesn't exist yet at that point.
//! - `State` holds everything that depends on having a live window. We build it
//!   inside `resumed`, once winit gives us a real window to attach to.
//!
//! This split is a winit convention. It looks like overkill for one window, but
//! it scales naturally to platforms (Android, iOS) where the OS can drop the
//! window out from under you and recreate it later.
//!
//! ## Recommended reading (start here)
//!
//! - **winit — `ApplicationHandler` docs**
//!   <https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html>
//!   The trait we implement. Lists every callback winit can fire.
//!
//! - **winit — Migrating to 0.30 (event-loop redesign rationale)**
//!   <https://docs.rs/winit/0.30.0/winit/changelog/v0_30/index.html>
//!   Why the API looks the way it does — `resumed`/`suspended` exist because
//!   mobile platforms forced a rethink of "the window is always alive."
//!
//! - **The Rust Book — `Arc<T>`**
//!   <https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-arct>
//!   We wrap the window in `Arc` to allow shared ownership; ch02's Surface will
//!   keep its own handle to it.
//!
//! For deeper dives, see `FURTHER_READING.md` at the project root.

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

/// Everything that depends on having a live OS window.
///
/// Built inside `App::resumed` once winit has actually given us a window.
/// In ch02 this struct grows to hold the wgpu Surface, Device, Queue, etc.
struct State {
    /// The OS window. Wrapped in `Arc` (atomic reference count) for shared
    /// ownership: ch02's `wgpu::Surface` will hold its own handle to the same
    /// window, and `Window` itself is not `Clone`. The window stays alive
    /// until the *last* `Arc` drops.
    window: Arc<Window>,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        Self { window }
    }

    /// Called whenever the OS resizes the window.
    /// Ch01 has nothing to do here; ch02 reconfigures the wgpu surface.
    fn resize(&mut self, _width: u32, _height: u32) {
        // ch02 will reconfig the wgpu surface here
    }

    /// Called when the OS asks us to redraw. Ch01 just re-arms the next redraw
    /// so we keep getting events; ch02 records and submits a render pass.
    fn render(&mut self) {
        // ch02 will record + submit a render pass here
        self.window.request_redraw();
    }
}

/// The winit application handler — the object whose methods winit calls.
///
/// `state` is `Option` because the window (and any state that depends on it)
/// only exists between `resumed` and `suspended`. On desktop we resume once
/// at startup and never suspend; on mobile the OS can suspend/resume freely.
#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    /// Called when the application becomes active. On desktop this fires once
    /// at startup; on mobile it can fire again after a `suspended`.
    ///
    /// This is where we create the window — it must happen *here*, not in
    /// `main`, because the event loop has to be running before we can ask the
    /// OS for a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Guard against re-entry: if we already built our state in a previous
        // resume, don't rebuild it.
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Rust Game Engine - ch01")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

        self.state = {
            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");
            // Wrap in Arc so ch02's Surface can co-own the window.
            let window_ref = Arc::new(window);
            Some(State::new(window_ref))
        }
    }

    /// Called for every event targeted at our window — resize, redraw,
    /// keyboard, mouse, close, focus changes, and many more. We `match` on the
    /// variants we care about and ignore the rest with `_ => {}`.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // User clicked the close button or pressed Alt-F4 / Cmd-Q.
            WindowEvent::CloseRequested => {
                println!("Quitting!");
                event_loop.exit()
            }
            // Window changed size. Ch02 will use this to reconfigure the surface.
            WindowEvent::Resized(physical_size) => {
                println!("Resized called!");
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size.width, physical_size.height)
                }
            }
            // OS asked us to redraw. Triggered by initial show, expose, or our
            // own `window.request_redraw()` call inside `render`.
            WindowEvent::RedrawRequested => {
                println!("Redraw requested!");
                if let Some(state) = self.state.as_mut() {
                    state.render()
                }
            }
            // Destructuring pattern: only matches *Pressed* events for the
            // *Escape* key specifically. The `..` ignores the other fields of
            // KeyEvent and WindowEvent::KeyboardInput we don't care about.
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                println!("Escape pressed - quitting!");
                event_loop.exit()
            }
            _ => {}
        }
    }
}

fn main() {
    // Route `log` crate output to stderr. Ch01 doesn't log anything itself,
    // but winit and (in ch02) wgpu both emit useful warnings via `log`.
    // Run with `RUST_LOG=info` to see them.
    env_logger::init();

    // Build the loop, then hand control to it. `run_app` blocks until the loop
    // exits — every app event is delivered via the `App` callbacks above.
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Event loop crashed.")
}
