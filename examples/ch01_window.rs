use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
// a trait we'll implement
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Rust Game Engine - ch01")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

        self.window = Some(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        )
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
