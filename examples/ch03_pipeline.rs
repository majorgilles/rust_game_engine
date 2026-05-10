//! Chapter 3: drawing a single triangle via a render pipeline.
//!
//! The wgpu pipeline at a glance:
//!
//!   Window  ─►  Surface  ─►  Adapter (a GPU)  ─►  Device + Queue
//!
//! Each frame we acquire a texture from the Surface, record a render pass
//! that clears it to a color, submit the commands, then present the frame.
//!
//! # How does an image actually end up on the screen?
//!
//! The high-level story: the CPU records a list of commands and hands them
//! to the GPU. The GPU runs them in parallel, writing colors into a texture.
//! The OS then "presents" that texture to the monitor in sync with the display.
//!
//! # Why building a pipeline feels verbose
//!
//! A render pipeline is the GPU's promise that *everything matches*:
//! vertex shader output → fragment shader input, fragment output format →
//! render target format, vertex buffer layout → vertex shader inputs, and
//! so on. If any of those don't line up, wgpu rejects the pipeline at
//! creation time with a clear message.
//!
//! That up-front strictness is why a pipeline descriptor has so many fields:
//! it's catching mistakes that would otherwise show up later as a black
//! screen, garbled colors, or a mysterious crash with no explanation.
//! Verbose now, debuggable forever.
//!
//! ## Recommended reading (start here)
//!
//! - **Learn Wgpu — Tutorial 3: The Pipeline**
//!   <https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/>
//!   The chapter this example follows. Walks through writing the shader,
//!   building a render pipeline, and issuing the draw call for one triangle.
//!
//! - **WebGPU Fundamentals — "Inter-stage variables"**
//!   <https://webgpufundamentals.org/webgpu/lessons/webgpu-inter-stage-variables.html>
//!   Beginner intro to *what a vertex shader and fragment shader actually do*,
//!   and how data flows from one to the other. Same concepts as wgpu, easier prose.
//!
//! - **WGSL Tour (interactive)**
//!   <https://google.github.io/tour-of-wgsl/>
//!   Bite-sized lessons on WGSL, the shader language we're about to write.
//!   Skim "Hello WGSL" and "Functions" — that's enough for this chapter.
//!
//! - **wgpu examples — `hello_triangle`**
//!   <https://github.com/gfx-rs/wgpu/tree/trunk/examples/features/src/hello_triangle>
//!   The official wgpu "draw one triangle" example. Useful as a second
//!   reference when our code feels unclear — small enough to read end-to-end.
//!
//! For deeper dives (graphics pipeline internals, real-engine renderers,
//! shader programming), see `FURTHER_READING.md` at the project root.
//!
//! ## Glossary mapping (this file → industry terms)
//!
//! | This file                    | What it's called elsewhere                   |
//! |------------------------------|----------------------------------------------|
//! | `ShaderModule`               | compiled shader / shader blob                |
//! | `RenderPipeline`             | pipeline state object (PSO in D3D)           |
//! | vertex shader (`vs_main`)    | vertex stage / vertex program                |
//! | fragment shader (`fs_main`)  | fragment stage / pixel shader (D3D)          |
//! | `@builtin(position)`         | clip-space position / `gl_Position` in GLSL  |
//! | `draw(0..3, 0..1)`           | non-indexed draw call                        |

use pollster::FutureExt;
use std::iter::once;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
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

    /// The drawable region of the window from wgpu's point of view —
    /// the bridge between the OS window and the GPU. We acquire a texture
    /// from it each frame, draw into it, and present it.
    ///
    /// The `'static` says the surface's borrowed window lives forever, which
    /// is true because `Arc<Window>` keeps it alive as long as anyone holds one.
    surface: wgpu::Surface<'static>,

    /// Open connection to the GPU. Used to *create* resources
    /// (buffers, textures, pipelines, command encoders).
    device: wgpu::Device,

    /// Command submission channel. We hand it recorded command buffers and
    /// the GPU executes them in order. Submitting is how work actually happens.
    queue: wgpu::Queue,

    /// Size, pixel format, and present settings for the Surface.
    /// Re-applied via `surface.configure` whenever the window resizes.
    surface_configuration: wgpu::SurfaceConfiguration,

    /// The compiled shader + pipeline settings the GPU uses to draw our triangle.
    /// Built once in `new`, bound at the start of every render pass.
    render_pipeline: wgpu::RenderPipeline,
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
        let surface_capabilities = surface.get_capabilities(&adapter);

        // Prefer an sRGB format so a value like 0.5 renders as a perceptual mid-gray.
        // Linear formats would look too dark on the monitor without manual gamma correction.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let size = window.inner_size();
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

        // Compile the WGSL into a shader module the GPU can run.
        // `include_str!` reads the .wgsl file at compile time and embeds it as a string.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ch03_pipeline.wgsl").into()), // into() converts &str from include_str! to Cow that Wgsl(...) expects
        });

        // A pipeline layout declares what *resources* (buffers, textures, samplers) the shader will
        // read from. Our shader reads nothing yet, it computes positions from `vertex_index` and
        // outputs a hardcoded color, so the layout is empty.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Triangle Pipeline Layout"),
            bind_group_layouts: &[], // groups of resources
            push_constant_ranges: &[],
        });

        // The render pipeline ties everything together: which shader runs at the vertex
        // stage, which runs at the fragment stage, what shape the input is, what the
        // output color format is, and how triangles are turned into pixels.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&pipeline_layout),

            // ---- Vertex stage ----
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[], // "no vertex buffers." We don't pass any vertex data from the CPU
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            // ---- Fragment stage ----
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format,  // must match the surface's pixel format. If they disagree, the pipeline is invalid: the shader writes one format, the screen expects another
                    blend: Some(wgpu::BlendState::REPLACE), // "the fragment color overwrites whatever was there." The alternative is alpha-blending (semi-transparency), which we don't need
                    write_mask: wgpu::ColorWrites::ALL,  // write all four channels (RGBA). You could mask out individual channels for special effects
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            // ---- How triangles get rasterized ----
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,  // "every 3 consecutive vertices form a triangle." Other options: LineList, PointList, TriangleStrip. With 3 vertices and TriangleList, we get exactly one triangle.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // vertices listed in counter-clockwise order = the front of the triangle. (Ours go bottom-right → top → bottom-left, which is CCW when viewed normally.)
                cull_mode: Some(wgpu::Face::Back), // "throw away triangles whose back side is facing the camera." Saves work; harmless when only one triangle.
                polygon_mode: wgpu::PolygonMode::Fill, // fill the inside. Line would draw only edges (wireframe)
                unclipped_depth: false,
                conservative:false,
            },

            // no depth buffer or MSAA yet - keep it minimal
            depth_stencil: None,  // no depth testing yet. We'll add it when we draw 3D meshes that overlap
            multisample: wgpu::MultisampleState::default(), // anti-aliasing off (samples = 1). Default is fine
            multiview: None,
            cache: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            surface_configuration,
            render_pipeline,
        }
    }

    /// Update the surface to match the new window size.
    /// The surface's image buffers were sized for the old window — without this they'd
    /// either crash wgpu or stretch a stale image across the new window.
    fn resize(&mut self, width: u32, height: u32) {
        // Minimizing fires a resize with (0, 0). Configuring a 0-sized surface is a
        // wgpu validation error, so skip it; the next resize (un-minimize) will reconfigure.
        if width == 0 || height == 0 {
            return;
        }
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration)
    }

    /// Draw one frame. Called on every `RedrawRequested`.
    ///
    /// The four-step rhythm of a wgpu frame:
    ///   1. **Acquire** — get the next surface texture to draw into.
    ///   2. **Encode** — record GPU commands into a CommandEncoder.
    ///   3. **Submit** — hand the recorded commands to the queue.
    ///   4. **Present** — tell the OS to display the finished frame.
    fn render(&mut self) {
        // 1. Acquire. The surface hands us the next texture to render into.
        // Lost/Outdated typically follow a resize or wake-from-sleep — recover by reconfiguring
        // and dropping this frame.
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

        // 2. Encode. A CommandEncoder is a buffer that we record GPU commands into;
        // nothing executes until we submit. Always cheap to make a fresh one per frame.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Inner scope: the render pass borrows `encoder` mutably. We must drop the pass
        // before calling `encoder.finish()` below — ending this block does that.
        {
            // A TextureView is a typed window into a texture. The render pass writes via the view,
            // not the texture directly, so it knows the format/layout it's working with.
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // `load` says what's already in the attachment when the pass starts; `store` says
            // whether to keep what we wrote. Clear-on-load + Store == "wipe to this color, keep result."
            let operations = wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            };
            // A color attachment hooks a TextureView into a render pass slot.
            // `resolve_target` is for MSAA (multisample antialiasing) — unused here.
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: operations,
            };
            // A render pass is one bundle of "draw into these targets with these settings."
            // Even just to clear, we still need a pass — clearing is part of starting one.
            let descriptor = wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut render_pass = encoder.begin_render_pass(&descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1); // we hard code vertices 0, 1, 2, and we draw 1 instance, 1 triangle
        }

        // 3. Submit. The queue runs the recorded commands on the GPU.
        let command_buffer = encoder.finish();
        self.queue.submit(once(command_buffer));
        // 4. Present. Without this, the GPU drew but the OS never shows it.
        frame.present();
        // Ask winit for another RedrawRequested so we keep rendering continuously.
        self.window.request_redraw();
    }
}

/// The winit application handler. Holds rendering state in an Option because
/// the window (and therefore the wgpu state) only exists between `resumed` and
/// `suspended` — on platforms like Android the OS can drop them out from under us.
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
            .with_title("Rust Game Engine - ch03")
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
