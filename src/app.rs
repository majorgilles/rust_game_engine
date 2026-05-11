//! Top-level application lifecycle and event routing.
//!
//! This module owns the `winit` `ApplicationHandler`: it creates the window,
//! forwards resize/redraw events to the renderer, and handles global app input
//! like Escape-to-quit. GPU details stay in `renderer`.

use crate::renderer::Renderer;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

/// The winit application handler. Holds rendering state in an Option because
/// the window (and therefore the wgpu state) only exists between `resumed` and
/// `suspended` — on platforms like Android the OS can drop them out from under us.
#[derive(Default)]
pub struct App {
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Rust Game Engine")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

        self.renderer = {
            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");
            let window_ref = Arc::new(window);
            Some(Renderer::new(window_ref))
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(physical_size.width, physical_size.height)
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.render()
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
            } => event_loop.exit(),
            _ => {}
        }
    }
}
