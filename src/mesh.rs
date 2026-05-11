//! Mesh data and vertex layout for the current renderable shape.
//!
//! The CPU-side `Vertex` type and `Vertex::desc()` must match the WGSL shader's
//! `@location` inputs. The renderer uploads `VERTICES` and `INDICES` into GPU
//! buffers and draws them with one indexed draw call.

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
];

pub const INDICES: &[u16] = &[
    0, 1, 2, // square: lower-right triangle
    0, 2, 3, // square: upper-left triangle
    2, 4, 3, // roof: counter-clockwise, so it is not culled
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
