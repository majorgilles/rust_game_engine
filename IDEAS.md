# Ideas & learnings

Running notebook of concept notes, "this could be simpler" thoughts, and surprises
encountered while working through Learn Wgpu. Re-read before starting Phase 2.

## ch01

### `Arc<Window>` — shared ownership for the surface

Wrapping the `winit::Window` in `Arc<Window>` looks like overkill in ch01 (only one
consumer). It's there because ch02's `wgpu::Surface` will hold its own reference to
the same window, and `Window` isn't `Clone`. `Arc` ("Atomic Reference Count") gives
shared ownership: multiple holders, the window is freed when the last `Arc` drops.
Roughly the default in Java; an opt-in in Rust.

Tradeoff: a tiny atomic increment/decrement per clone. Negligible here — we clone
maybe twice for the lifetime of the program.

## ch02

### "WebGPU" is not web-only — the name is historical

WebGPU is a *graphics API specification*, not a browser feature. It was designed by
browser vendors at the W3C as the successor to WebGL, hence the name — but the API
itself has no browser dependency. Implementations translate WebGPU calls into the
native graphics API of whatever platform you're on:

| Platform     | Native API                |
|--------------|---------------------------|
| Windows      | Direct3D 12 (or Vulkan)   |
| Linux        | Vulkan                    |
| macOS / iOS  | Metal                     |
| Android      | Vulkan                    |
| Browser      | The browser's WebGPU impl |

`wgpu` (the Rust crate) is one such implementation. When we call
`wgpu::Instance::new`, we're loading the actual native GPU driver — no browser, no
WebAssembly, no JavaScript involved. `cargo run` produces a normal native binary.

The bonus: the same code can also be compiled to WebAssembly and run in a browser
tab using the browser's built-in WebGPU. One codebase, two deployment targets.

**Why it caught on outside the web:** Vulkan / D3D12 / Metal are powerful but
verbose (hundreds of lines for a triangle). WebGPU is a deliberately simplified
version of those concepts while still being modern (compute shaders, explicit
pipeline objects). It hits a sweet spot between "OpenGL was too magical" and
"Vulkan is too much" — so Rust game devs and scientific computing folks adopted it
as their default cross-platform GPU API even with no web involvement.

Mental model: wgpu = native graphics with a portable API; "Web" is historical.


## ch04

### Winding order + culling was the big gotcha

With `front_face: Ccw` and `cull_mode: Back`, the order of triangle indices decides
whether a triangle is visible. The roof triangle `2, 3, 4` disappeared because it was
wound the wrong way and got back-face culled. Reordering it to `2, 4, 3` made it
counter-clockwise from the camera's point of view, so it rendered.

Mental model: if geometry vanishes after changing indices, check triangle winding
before assuming the buffers or shader are broken.

## ch05

### Texture mapping is not "putting an image on a quad"

The GPU does not really know about "quads" or "one texture per quad." It knows about
triangles, vertex attributes, bound resources, and shaders.

The chain is:

1. Rust binds a texture + sampler in a bind group.
2. Each vertex contains both a screen-space `position` and a `texture_coordinates`
   UV value.
3. The index buffer groups vertices into triangles.
4. The vertex shader outputs clip-space position and passes UVs along.
5. The rasterizer fills the triangle and interpolates UVs for each fragment.
6. The fragment shader calls `textureSample(...)` with the interpolated UV.
7. The sampler decides how to handle out-of-range UVs and filtering.

Mental model: the texture is not copied onto geometry. The fragment shader asks,
"for this pixel, what color should I read from the currently bound texture?"

### Interpolation: vertex data becomes fragment data

UVs are only written explicitly on vertices, but the fragment shader runs for many
fragments/pixels inside the triangle. The GPU fills in the missing values between
the corners automatically.

For a triangle with corner UVs:

```text
A: [0.0, 3.0]
B: [3.0, 3.0]
C: [3.0, 0.0]
```

a fragment inside the triangle gets a weighted blend of those values. Roughly:

```text
fragment UV = 20% of A + 50% of B + 30% of C
```

That is interpolation: calculating the in-between attribute value based on where the
fragment lies inside the triangle.

In this shader, `texture_coordinates` leaves the vertex shader as a corner value:

```wgsl
out.texture_coordinates = input.texture_coordinates;
```

but arrives in the fragment shader as a per-fragment interpolated value:

```wgsl
textureSample(t_diffuse, s_diffuse, input.texture_coordinates)
```

Mental model: the vertex shader provides UVs at triangle corners; the rasterizer
creates a fresh in-between UV for every fragment.

### Sampling: a UV coordinate becomes a color

Sampling means reading a color from a texture using a UV coordinate and sampler
rules.

A texture is a grid of texels:

```text
texel = texture pixel
```

When the fragment shader calls:

```wgsl
textureSample(t_diffuse, s_diffuse, input.texture_coordinates)
```

it is asking:

```text
Given this UV, what color should this fragment be?
```

The texture provides the image data. The sampler provides the reading rules:

- **Address mode** decides what happens outside `0.0..1.0`.
  - `Repeat`: `1.25` behaves like `0.25`.
  - `MirrorRepeat`: every other repeated span is flipped.
  - `ClampToEdge`: values outside the range stick to the nearest edge.
- **Filter mode** decides how to read between texels.
  - `Nearest`: pick the closest texel.
  - `Linear`: blend nearby texels for a smoother result.

Mental model: interpolation chooses the UV for this fragment; sampling turns that UV
into a color.

### UV ranges control how many texture spans are crossed

`0.0..1.0` in UV space means one full texture span.

So:

| UV range | Visible result with repeat-style address modes |
|----------|------------------------------------------------|
| `0.0..1.0` | 1 copy total |
| `0.0..2.0` | 2 copies total |
| `0.0..3.0` | 3 copies total |

The word "repeat" was confusing: `0.0..3.0` is not "repeat once"; it means the
interpolated UV walks through three full texture-width intervals. The sampler wraps
or mirrors those coordinates back into the actual `0.0..1.0` image range.

### Independent quads can reasonably duplicate vertices

For the texture lab, keeping each quad as its own four vertices is acceptable and
arguably clearer than sharing vertices.

A `Vertex` is not just a position. It is the full bundle:

```rust
position + texture_coordinates
```

Two corners at the same screen position cannot be merged if they need different UVs,
materials, normals, or other attributes. Shared vertices make sense for one
continuous surface; duplicated vertices make sense for independent tiles or UV seams.

For this experiment:

```text
16 independent quads * 4 vertices = 64 vertices
```

is tiny and conceptually honest.

### CPU setup vs GPU execution

The Rust/wgpu code runs on the CPU and configures work:

```rust
create_render_pipeline(...)
set_vertex_buffer(...)
set_index_buffer(...)
draw_indexed(...)
```

The WGSL shader functions run on the GPU:

```wgsl
@vertex
fn vs_main(...)

@fragment
fn fs_main(...)
```

Between them, the GPU has fixed-function stages for primitive assembly,
rasterization, interpolation, texture sampling support, depth testing, and output.

Mental model: CPU submits a recipe; GPU hardware executes the graphics pipeline.
