//! Chapter 2: drawing a clear color into a window via wgpu.
//!
//! The wgpu pipeline at a glance:
//!
//!   Window  ─►  Surface  ─►  Adapter (a GPU)  ─►  Device + Queue
//!
//! Each frame we acquire a texture from the Surface, record a render pass
//! that clears it to a color, submit the commands, then present the frame.

use pollster::FutureExt;
use std::iter::once;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

/// All long-lived rendering state. Built once in `resumed`, lives until exit.
struct State {
    /// The OS window. `Arc` because the Surface also keeps a handle to it —
    /// shared ownership is how we promise wgpu that the window outlives the surface.
    window: Arc<Window>,

    /// Entry point to wgpu. Used to enumerate GPUs and to create surfaces.
    /// Made once at startup; nothing else holds state here.
    instance: wgpu::Instance,

    /// The drawable region of the window from wgpu's point of view —
    /// the bridge between the OS window and the GPU. We acquire a texture
    /// from it each frame, draw into it, and present it.
    ///
    /// The `'static` says the surface's borrowed window lives forever, which
    /// is true because `Arc<Window>` keeps it alive as long as anyone holds one.
    surface: wgpu::Surface<'static>,

    /// Handle to a specific physical GPU on the machine.
    /// Describes capabilities and is used to open a device; doesn't run commands itself.
    adapter: wgpu::Adapter,

    /// Open connection to the GPU. Used to *create* resources
    /// (buffers, textures, pipelines, command encoders).
    device: wgpu::Device,

    /// Command submission channel. We hand it recorded command buffers and
    /// the GPU executes them in order. Submitting is how work actually happens.
    queue: wgpu::Queue,

    /// Size, pixel format, and present settings for the Surface.
    /// Re-applied via `surface.configure` whenever the window resizes.
    surface_configuration: wgpu::SurfaceConfiguration,
}

impl State {
    /// Build all wgpu state. Async work (adapter/device requests) is run synchronously
    /// here via `pollster`'s `block_on`, since this example has no async runtime.
    fn new(window: Arc<Window>) -> Self {
        // `default()` lets wgpu pick whichever backend the OS prefers (Vulkan/DX12/Metal).
        let instance_descriptor = wgpu::InstanceDescriptor::default();
        let instance = wgpu::Instance::new(&instance_descriptor);

        // Cloning an Arc just bumps a refcount; surface and State both end up
        // holding the same window.
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // Pick a GPU. `compatible_surface` ensures the chosen GPU can actually
        // render to *this* window — important on dual-GPU laptops.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("Failed to find an appropriate adapter");

        // Open a connection to that GPU. We declare up-front what we need;
        // wgpu fails *now* if the GPU can't deliver, instead of mysteriously later.
        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("Main Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::default(),
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor)
            .block_on()
            .expect("Failed to create device.");

        // Configure the surface: tell it how big, what pixel format, and how to time frames.
        let size = window.inner_size();
        let surface_capabilities = surface.get_capabilities(&adapter);

        // Prefer an sRGB format so a value like 0.5 renders as a perceptual mid-gray.
        // Linear formats would look too dark on the monitor without manual gamma correction.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_configuration = wgpu::SurfaceConfiguration {
            // RENDER_ATTACHMENT == "we will draw into this surface's textures."
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            // Floor at 1: a 0-sized surface is invalid, and minimized windows can report 0.
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);

        Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_configuration,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            // Configuring a 0-sized surface is a wgpu validation error — so we skip it and let the
            // next resize (when the user un-minimizes) reconfigure properly.
            return;
        }
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration)
    }

    fn render(&mut self) { // called each frame
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface
                    .configure(&self.device, &self.surface_configuration);
                return;
            }
            Err(e) => {
                eprintln!("Surface error: {e:?}");
                return;
            }
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let operations = wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            };
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: operations,
            };
            let descriptor = wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let _render_pass = encoder.begin_render_pass(&descriptor);
        } // We can't call encoder.finish() while the pass is still alive, so we drop it by ending the scope, hence the { ... } block

        self.queue.submit(once(encoder.finish()));
        frame.present(); // without that, nothing is drawn
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size.width, physical_size.height)
                }
            }
            WindowEvent::RedrawRequested => {
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
            } => event_loop.exit(),
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
