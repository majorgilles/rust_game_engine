# Further reading

Deeper-dive resources, kept out of the per-chapter file headers so the in-code
reading lists stay short and beginner-friendly. Come here when you want to
understand what's happening *underneath* the wgpu API.

## How GPUs actually render images

- **Ryg's "A trip through the Graphics Pipeline 2011" series**
  <https://fgiesen.wordpress.com/2011/07/09/a-trip-through-the-graphics-pipeline-2011-index/>
  The classic deep dive. Explains what GPUs *actually do* between submit and
  present — command processors, rasterization, framebuffers, vsync. Long, but
  pieces fall into place.

- **Fabien Sanglard — Doom 3 renderer breakdown**
  <https://fabiensanglard.net/doom3/renderer.php>
  Concrete tour of how a real (older but still excellent) engine turns scene
  data into pixels. Useful once you've seen the basic clear-and-present loop.

- **"A Trip Down the Graphics Pipeline" — Jim Blinn (PDF, classic paper)**
  <https://www.cs.cornell.edu/courses/cs4620/2017sp/cs4620/lectures/01-intro.pdf>
  Older but timeless overview of the conceptual stages: vertices → primitives
  → fragments → pixels.

- **Real-Time Rendering (book, Akenine-Möller et al.)**
  <https://www.realtimerendering.com/>
  The reference textbook. Free chapters online; the rest is worth buying once
  you're past basics.

## WebGPU and wgpu specifications

- **WebGPU Specification (W3C)**
  <https://www.w3.org/TR/webgpu/>
  The actual standards document. Dense, but the source of truth for what every
  type and method means.

- **WGSL Specification (the shading language)**
  <https://www.w3.org/TR/WGSL/>
  Companion spec for the shader language. Skim it before ch3 when shaders
  arrive.

- **wgpu changelog & milestone notes**
  <https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md>
  Useful when an API has shifted between wgpu versions (the crate evolves
  faster than tutorials online).

- **Surfman / "Why is graphics hard?" (Mozilla)**
  <https://hacks.mozilla.org/2020/04/the-baseline-interpreter-a-faster-js-interpreter-in-firefox-75/>
  Background on why bridging windows and GPUs across OSes is genuinely
  difficult — context for what wgpu's `Surface` is hiding from you.

## Shader programming

- **The Book of Shaders**
  <https://thebookofshaders.com/>
  Best gentle intro to thinking pixel-by-pixel. Uses GLSL, but the concepts
  transfer directly to WGSL (wgpu's shader language).

- **Inigo Quilez — articles & shadertoy**
  <https://iquilezles.org/articles/>
  Once shaders click, this is where the rabbit hole gets deep. Procedural
  graphics, signed distance fields, raymarching.

## Native graphics APIs (what wgpu sits on top of)

- **Vulkan Tutorial**
  <https://vulkan-tutorial.com/>
  Shows you what wgpu is *saving you from*. Reading the first few chapters
  builds appreciation for WebGPU's simplifications.

- **Apple — Metal Programming Guide**
  <https://developer.apple.com/library/archive/documentation/Miscellaneous/Conceptual/MetalProgrammingGuide/>
  The macOS/iOS native API wgpu translates to on Apple platforms.

- **Microsoft — Direct3D 12 programming guide**
  <https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide>
  The Windows native API wgpu uses by default on Windows.
