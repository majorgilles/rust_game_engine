//! Rendering backend for the current chapter.
//!
//! This module owns the long-lived `wgpu` objects: surface, device, queue,
//! pipeline, and GPU buffers. `App` tells it when the window resizes or when a
//! frame should be rendered; everything below that boundary is rendering work.

use crate::mesh::{Vertex, create_vertices_for_quads};
use crate::texture::Texture;
use pollster::FutureExt;
use std::iter::once;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

const NUMBER_QUADS: i32 = 4;

/// All long-lived rendering state components. Built once in `resumed`, lives until exit.
pub struct Renderer {
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

    /// The compiled shader and pipeline settings the GPU uses to draw our shapes.
    /// Built once in `new`, bound at the start of every render pass.
    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    _diffuse_texture: Texture,
    clamp_linear_bind_group: wgpu::BindGroup,
    _clamp_linear_sampler: wgpu::Sampler,
    repeat_nearest_bind_group: wgpu::BindGroup,
    _repeat_nearest_sampler: wgpu::Sampler,
    mirror_nearest_bind_group: wgpu::BindGroup,
    _mirror_nearest_sampler: wgpu::Sampler,
    repeat_linear_bind_group: wgpu::BindGroup,
    _repeat_linear_sampler: wgpu::Sampler,
}

impl Renderer {
    /// Build all wgpu state. Async work (adapter/device requests) is run synchronously
    /// here via `pollster`'s `block_on`, since this app has no async runtime.
    pub fn new(window: Arc<Window>) -> Self {
        // `default()` lets wgpu pick whichever backend the OS prefers (Vulkan/DX12/Metal).
        let instance_descriptor = wgpu::InstanceDescriptor::default();
        let instance = wgpu::Instance::new(&instance_descriptor);

        // Cloning an Arc just bumps a refcount; surface and Renderer both end up
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
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()), // into() converts &str from include_str! to Cow that Wgsl(...) expects
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png")
            .expect("Failed to load diffuse texture");

        let clamp_linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Clamp Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let clamp_linear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Clamp Linear Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&clamp_linear_sampler),
                },
            ],
        });

        let repeat_nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Repeat Nearest Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let repeat_nearest_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Repeat Nearest Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&repeat_nearest_sampler),
                },
            ],
        });

        let mirror_nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mirror Nearest Sampler"),
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mirror_nearest_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mirror Nearest Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&mirror_nearest_sampler),
                },
            ],
        });

        let repeat_linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Repeat Linear Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let repeat_linear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Repeat Linear Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&repeat_linear_sampler),
                },
            ],
        });

        // A pipeline layout declares bind-group resources like uniforms, textures and samplers.
        // Vertex/index buffers are not bind groups; they are configured separately in
        // `vertex.buffers` and bound in the render pass. We use no bind groups yet, so this is
        // empty.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // The render pipeline ties everything together: which shader runs at the vertex
        // stage, which runs at the fragment stage, what shape the input is, what the
        // output color format is, and how triangles are turned into pixels.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Buffers Pipeline"),
            layout: Some(&pipeline_layout),

            // ---- Vertex stage ----
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            // ---- Fragment stage ----
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format, // must match the surface's pixel format. If they disagree, the pipeline is invalid: the shader writes one format, the screen expects another
                    blend: Some(wgpu::BlendState::REPLACE), // "the fragment color overwrites whatever was there." The alternative is alpha-blending (semi-transparency), which we don't need
                    write_mask: wgpu::ColorWrites::ALL, // write all four channels (RGBA). You could mask out individual channels for special effects
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            // ---- How triangles get rasterized ----
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // "every 3 consecutive vertices form a triangle." Other options: LineList, PointList, TriangleStrip. With 3 vertices and TriangleList, we get exactly one triangle.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // vertices listed in counter-clockwise order = the front of the triangle. (Ours go bottom-right → top → bottom-left, which is CCW when viewed normally.)
                cull_mode: Some(wgpu::Face::Back), // "throw away triangles whose back side is facing the camera." Saves work; harmless when only one triangle.
                polygon_mode: wgpu::PolygonMode::Fill, // fill the inside. Line would draw only edges (wireframe)
                unclipped_depth: false,
                conservative: false,
            },

            // no depth buffer or MSAA yet - keep it minimal
            depth_stencil: None, // no depth testing yet. We'll add it when we draw 3D meshes that overlap
            multisample: wgpu::MultisampleState::default(), // antialiasing off (samples = 1). Default is fine
            multiview: None,
            cache: None,
        });

        let (vertices, indices) = create_vertices_for_quads(NUMBER_QUADS);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            window,
            surface,
            device,
            queue,
            surface_configuration,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            _diffuse_texture: diffuse_texture,
            clamp_linear_bind_group,
            _clamp_linear_sampler: clamp_linear_sampler,
            repeat_nearest_bind_group,
            _repeat_nearest_sampler: repeat_nearest_sampler,
            mirror_nearest_bind_group,
            _mirror_nearest_sampler: mirror_nearest_sampler,
            repeat_linear_bind_group,
            _repeat_linear_sampler: repeat_linear_sampler,
        }
    }

    /// Update the surface to match the new window size.
    /// The surface's image buffers were sized for the old window — without this they'd
    /// either crash wgpu or stretch a stale image across the new window.
    pub fn resize(&mut self, width: u32, height: u32) {
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
    pub fn render(&mut self) {
        // 1. Acquire. The surface hands us the next texture to render into.
        // Lost/Outdated textures typically follow a resize or wake-from-sleep —
        // recover by reconfiguring and dropping this frame.
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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for quad_index in 0..NUMBER_QUADS {
                let start = (quad_index * 6) as u32;
                let end = start + 6;

                match quad_index {
                    0 => render_pass.set_bind_group(0, &self.repeat_nearest_bind_group, &[]),
                    1 => render_pass.set_bind_group(0, &self.mirror_nearest_bind_group, &[]),
                    2 => render_pass.set_bind_group(0, &self.repeat_linear_bind_group, &[]),
                    3 => render_pass.set_bind_group(0, &self.clamp_linear_bind_group, &[]),
                    _ => unreachable!(),
                }

                render_pass.draw_indexed(start..end, 0, 0..1);
            }
        }

        // 3. Submit. The queue runs the recorded commands on the GPU.
        let command_buffer = encoder.finish();
        self.queue.submit(once(command_buffer));
        // 4. Present. Without this, the GPU drew the image, but the OS never shows it.
        frame.present();
        // Ask winit for another RedrawRequested so we keep rendering continuously.
        self.window.request_redraw();
    }
}
