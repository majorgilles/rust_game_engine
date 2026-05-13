//! Mesh data and vertex layout for the current renderable shape.
//!
//! The CPU-side `Vertex` type and `Vertex::desc()` must match the WGSL shader's
//! `@location` inputs. The renderer uploads `VERTICES` and `INDICES` into GPU
//! buffers and draws them with one indexed draw call.

// Chapter 5 variation: UVs go past 1.0 so sampler address modes can tile/mirror the texture.
pub const VERTICES: &[Vertex] = &[
    Vertex {
        // bottom-left
        position: [-0.5, -0.5, 0.0],
        texture_coordinates: [0.0, 1.0],
    },
    Vertex {
        // bottom-right
        position: [0.5, -0.5, 0.0],
        texture_coordinates: [3.0, 1.0],
    },
    Vertex {
        // top-right
        position: [0.5, 0.5, 0.0],
        texture_coordinates: [3.0, 0.0],
    },
    Vertex {
        // top-left
        position: [-0.5, 0.5, 0.0],
        texture_coordinates: [0.0, 0.0],
    },
];

pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_coordinates: [f32; 2], // UV / tex_coords, address we use to read a texel
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
