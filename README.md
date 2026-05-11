# rust_game_engine

A learning project: build a small 3D game engine from scratch in Rust by following [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) end-to-end, then using the resulting renderer to build a 3D Pong game.

The engine is the *excuse* — the actual goal is to understand how engines work by writing one.

## Anchor resource

[Learn Wgpu](https://sotrh.github.io/learn-wgpu/) by sotrh. Free, actively maintained, ~13 chapters across Beginner and Intermediate sections. Teaches modern GPU programming via the `wgpu` crate (Rust's WebGPU implementation).

## Phases

### Phase 1 — Learn Wgpu (issues [#1](../../issues/1)–[#13](../../issues/13))

Work through every Beginner and Intermediate chapter of Learn Wgpu. One issue per chapter. For each chapter:

1. Keep the current/reference chapter implementation in `src/`.
2. Implement a small **variation** that proves the concept, not just recognition of it.

By the end you have a hand-rolled 3D renderer with model loading, lighting, normal mapping, a proper camera, and HDR tone mapping.

The Compute Pipelines and Showcase sections are intentionally **out of scope** for v1, except for the Showcase Pong (issue #14, see Phase 2a below).

### Phase 2a — 2D Pong (issue [#14](../../issues/14))

Type-along of [Learn Wgpu's Showcase Pong](https://sotrh.github.io/learn-wgpu/showcase/pong/). First end-to-end game built on the renderer. The point is to see how rendering, input, game state, and a game loop integrate into a working game *with the tutorial's blessing* — that becomes the architectural template the next phase intentionally departs from.

### Phase 2b — 3D Pong (issues [#15](../../issues/15)–[#19](../../issues/19))

Build an original 3D Pong game on top of the renderer. Five issues:

- **#15** Court + paddles + camera (HITL — proportions need eyeballing)
- **#16** Ball + 3D collision (4 walls + 2 paddles)
- **#17** Player input + brain-dead AI + scoring + win condition
- **#18** Depth-perception cue (HITL — pick what reads best)
- **#19** Polish + retrospective in `IDEAS.md`

### Game design

- Rectangular tunnel court — ball bounces off 4 walls + 2 paddles
- 1 player vs. brain-dead AI (tracks ball xy with lag)
- Solid colors only — no model loading, no fancy textures
- No spin / curve mechanics in v1

## Working agreement

- **Pace:** type-along + small variation per chapter. Variations are where real learning happens.
- **Repo layout:** one evolving codebase in `src/`. The latest chapter is the reference implementation; older chapter snapshots are not kept in separate runnable targets. Extract modules/shared helpers when they make the single codebase clearer.
- **Side artifact:** [`IDEAS.md`](IDEAS.md) — running list of "this could be simpler" / "wonder how multiple objects would work" notes during Phase 1. Becomes the input for Phase 2.

## Time estimates

These assume basic Rust knowledge, no prior graphics experience, and the type-along + small-variation working pattern. They are calendar-time estimates of focused work, not wall clock.

| Issue | Title | Hours (low–high) |
|------|------|------|
| #1 | ch01: Dependencies and the window | 2–4 |
| #2 | ch02: The Surface | 4–8 |
| #3 | ch03: The Pipeline | 6–10 |
| #4 | ch04: Buffers and Indices | 5–8 |
| #5 | ch05: Textures and bind groups | 6–10 |
| #6 | ch06: Uniform buffers and a 3D camera | 8–12 |
| #7 | ch07: Instancing | 5–8 |
| #8 | ch08: The Depth Buffer | 4–7 |
| #9 | ch09: Model Loading | 6–10 |
| #10 | ch10: Working with Lights | 6–10 |
| #11 | ch11: Normal Mapping | 5–8 |
| #12 | ch12: A Better Camera | 5–8 |
| #13 | ch13: HDR Rendering | 6–10 |
| **Phase 1 total** | | **68–113** |
| #14 | Phase 2a: 2D Pong (Showcase tutorial) | 10–18 |
| **Phase 2a total** | | **10–18** |
| #15 | 3D Pong: court + paddles + camera | 6–10 |
| #16 | 3D Pong: ball + 3D collision | 8–14 |
| #17 | 3D Pong: input + AI + scoring + win | 6–10 |
| #18 | 3D Pong: depth-perception cue | 4–8 |
| #19 | 3D Pong: polish + retrospective | 6–10 |
| **Phase 2b total** | | **30–52** |
| **Project total** | | **108–183** |

At a sustained ~10 hours/week, that's roughly **3–4.5 months** to v1. At ~5 hours/week, **5–9 months**. Real time will skew higher than the estimates — every estimate ever written has.

## Stack

- Rust (edition 2024)
- `wgpu` for graphics
- `winit` for windowing
- Other crates introduced as the tutorial calls for them

## Running

```sh
cargo run
```

The app in `src/` is the current reference implementation and evolves chapter by chapter.
