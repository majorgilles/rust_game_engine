//! Mesh data and vertex layout for the current renderable shape.
//!
//! The CPU-side `Vertex` type and `Vertex::desc()` must match the WGSL shader's
//! `@location` inputs. The renderer uploads `VERTICES` and `INDICES` into GPU
//! buffers and draws them with one indexed draw call.

const VERTICES_COUNT_PER_QUAD: usize = 4;

// Chapter 5 variation: UVs go past 1.0 so sampler address modes can tile/mirror the texture.

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Debug)]
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

fn compute_stride(number_of_quads: i32) -> f32 {
    1.0 / ((number_of_quads as f32).sqrt() / 2.0)
}

fn map_clip_x_to_uv_x(clip_x: f32) -> f32 {
    (clip_x + 1.0) * 0.5
}

fn map_clip_y_to_uv_y(clip_y: f32) -> f32 {
    (1.0 - clip_y) * 0.5
}

fn push_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    vertex_bounds: &QuadBounds,
    uv_bounds: &QuadBounds,
) {
    let base = vertices.len() as u16;
    vertices.extend_from_slice(&[
        Vertex {
            // bottom-left
            position: [vertex_bounds.left, vertex_bounds.bottom, 0.0],
            texture_coordinates: [uv_bounds.left, uv_bounds.bottom],
        },
        Vertex {
            // bottom-right
            position: [vertex_bounds.right, vertex_bounds.bottom, 0.0],
            texture_coordinates: [uv_bounds.right, uv_bounds.bottom],
        },
        Vertex {
            // top-right
            position: [vertex_bounds.right, vertex_bounds.top, 0.0],
            texture_coordinates: [uv_bounds.right, uv_bounds.top],
        },
        Vertex {
            // top-left
            position: [vertex_bounds.left, vertex_bounds.top, 0.0],
            texture_coordinates: [uv_bounds.left, uv_bounds.top],
        },
    ]);

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

struct QuadBounds {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

pub fn create_vertices_for_quads(number_of_quads: i32) -> (Vec<Vertex>, Vec<u16>) {
    let stride = compute_stride(number_of_quads);
    let mut vertex_bounds: Vec<QuadBounds> = Vec::new();
    let start = -1.0;
    let quads_per_side = (number_of_quads as f32).sqrt() as i32;

    for y in 0..quads_per_side {
        for x in 0..quads_per_side {
            let left = start + x as f32 * stride;
            let right = left + stride;
            let bottom = start + y as f32 * stride;
            let top = bottom + stride;
            let bounds = QuadBounds {
                left,
                right,
                bottom,
                top,
            };
            vertex_bounds.push(bounds);
        }
    }

    let uv_bounds: Vec<QuadBounds> = vertex_bounds
        .iter()
        .map(|bounds| QuadBounds {
            left: map_clip_x_to_uv_x(bounds.left),
            right: map_clip_x_to_uv_x(bounds.right),
            bottom: map_clip_y_to_uv_y(bounds.bottom),
            top: map_clip_y_to_uv_y(bounds.top),
        })
        .collect();

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    vertex_bounds
        .iter()
        .zip(uv_bounds.iter())
        .for_each(|(bounds, uv_bounds)| {
            push_quad(&mut vertices, &mut indices, &bounds, &uv_bounds)
        });
    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stride() {
        assert_eq!(compute_stride(1), 2.0);
        assert_eq!(compute_stride(4), 1.0);
        assert_eq!(compute_stride(16), 0.5);
    }

    #[test]
    fn test_map_clip_x_to_uv_x() {
        assert_eq!(map_clip_x_to_uv_x(-1.0), 0.0);
        assert_eq!(map_clip_x_to_uv_x(1.0), 1.0);
    }

    #[test]
    fn test_map_clip_y_to_uv_y() {
        assert_eq!(map_clip_y_to_uv_y(-1.0), 1.0);
        assert_eq!(map_clip_y_to_uv_y(1.0), 0.0);
    }

    #[test]
    fn test_create_vertices_for_quads() {
        // given
        let expected_vertices: &[Vertex] = &[
            Vertex {
                // bottom-left
                position: [-1.0, -1.0, 0.0],
                texture_coordinates: [0.0, 1.0],
            },
            Vertex {
                // bottom-right
                position: [1.0, -1.0, 0.0],
                texture_coordinates: [1.0, 1.0],
            },
            Vertex {
                // top-right
                position: [1.0, 1.0, 0.0],
                texture_coordinates: [1.0, 0.0],
            },
            Vertex {
                // top-left
                position: [-1.0, 1.0, 0.0],
                texture_coordinates: [0.0, 0.0],
            },
        ];
        let expected_indices: &[u16] = &[0, 1, 2, 0, 2, 3];

        // when
        let (vertices, indices) = create_vertices_for_quads(1);

        // then
        assert_eq!(vertices, expected_vertices);
        assert_eq!(indices, expected_indices);
    }
}
