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
