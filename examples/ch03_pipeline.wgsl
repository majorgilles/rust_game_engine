// Vertex shader: runs once per vertex (3 times for our triangle).
// Returns the corner's position in clip space (the GPU's coordinate system).
// vertex_index will take values 0, 1, 2
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    // x and y go from -1 (left/bottom) to +1 (right/top). The 0.5 shrinks the triangle so it's not glued to the window edges
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    return vec4<f32>(x, y, 0.0, 1.0);
}

// Fragment shader: runs once per pixel inside the triangle.
// `@builtin(position)` here is the pixel's screen-space position in pixels.
// "Fragment" is the GPU's word for "candidate pixel"; you can read them as the same thing here
@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    if frag_pos.x < 1280.0 / 2.0 { // for the moment we hardcode width !!!
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
