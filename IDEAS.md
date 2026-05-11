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
