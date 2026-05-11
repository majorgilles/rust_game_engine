// Current main shader.
//
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
}

// Vertex shader: runs for vertices selected by the indexed draw.
// `input.position` and `input.texture_coordinates` come from the bound vertex buffer,
// using the layout described by `Vertex::desc()` in Rust
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 1.0);
    out.texture_coordinates = input.texture_coordinates;
    return out;
}

// Fragment shader: runs once per pixel inside the rasterized triangles.
// `@builtin(position)` here is the pixel's screen-space position in pixels.
// "Fragment" is the GPU's word for "candidate pixel"; you can read them as the same thing here
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
       return vec4<f32>(input.texture_coordinates, 0.0, 1.0);
}
