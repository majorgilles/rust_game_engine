use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
// a trait we'll implement
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

struct State {
    // Arc: wgpu's Surface (added in ch02) needs to share window ownership.
    window: Arc<Window>,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        Self { window }
    }

    fn resize(&mut self, _width: u32, _height: u32) {
        // ch02 will reconfig the wgpu surface here
    }

    fn render(&mut self) {
        // ch02 will record + submit a render pass here
        self.window.request_redraw();
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Rust Game Engine - ch01")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

        self.state = {
            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");
            let window_ref = Arc::new(window);
            Some(State::new(window_ref))
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
                println!("Quitting!");
                event_loop.exit();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init(); // turns on logging output. wgpu will use it later
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();

    event_loop.run_app(&mut app).expect("Event loop crashed.")
}
