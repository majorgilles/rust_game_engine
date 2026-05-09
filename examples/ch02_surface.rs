use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

struct State {
    window: Arc<Window>,
    instance: wgpu::Instance,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        // InstanceDescriptor::default() lets wgpu pick whichever backend the OS prefers
        let instance_descriptor = wgpu::InstanceDescriptor::default();
        let instance = wgpu::Instance::new(&instance_descriptor);
        Self { window, instance }
    }

    fn resize(&mut self, _width: u32, _height: u32) {}

    fn render(&mut self) {
        self.window.request_redraw();
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Rust Game Engine - ch02")
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
                event_loop.exit()
            }
            WindowEvent::Resized(physical_size) => {
                println!("Resized called!");
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size.width, physical_size.height)
                }
            }
            WindowEvent::RedrawRequested => {
                println!("Redraw requested!");
                if let Some(state) = self.state.as_mut() {
                    state.render()
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        // this pattern is a filter that triggers if the Escape key is pressed ONLY
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
    env_logger::init(); // turns on logging output. wgpu will use it later
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();

    event_loop.run_app(&mut app).expect("Event loop crashed.")
}
