//! Binary entry point.
//!
//! Keep this file intentionally small: initialize logging, create the winit
//! event loop, and hand control to the app module.

mod app;
mod mesh;
mod renderer;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init(); // turns on logging output. wgpu will use it later
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();

    event_loop.run_app(&mut app).expect("Event loop crashed.")
}
