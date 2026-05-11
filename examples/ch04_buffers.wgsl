// Chapter 4 shader: vertex inputs from a vertex buffer.
//
// Differs from ch03 in two ways:
//   - We declare a VertexInput struct with `@location(0)` and `@location(1)`.
//     These numbers must match the `shader_location` fields in
//     `Vertex::desc()` on the Rust side. Mismatch = wrong field reads garbage.
//   - We pass color from vertex stage to fragment stage via VertexOutput.
//     The fragment shader interpolates `color` across the triangle for free.
//
// Reading: WGSL @location reference
// <https://www.w3.org/TR/WGSL/#input-output-locations>

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// Vertex shader: runs once per vertex (3 times for our triangle).
// Returns the corner's position in clip space (the GPU's coordinate system).
// vertex_index will take values 0, 1, 2
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 1.0);
    out.color = input.color;
    return out;
}

// Fragment shader: runs once per pixel inside the triangle.
// `@builtin(position)` here is the pixel's screen-space position in pixels.
// "Fragment" is the GPU's word for "candidate pixel"; you can read them as the same thing here
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
       return vec4<f32>(input.color, 1.0);
}
