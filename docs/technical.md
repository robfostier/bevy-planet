# Technical documentation

> Living document: updated every session by the tutor. Describes HOW the project is built and WHY these choices were made.

## Stack

- Language: Rust (edition 2024)
- Engine: Bevy 0.19 (ECS, PBR rendering, winit)
- Dev build: `cargo run --features bevy/dynamic_linking` (dynamic linking for fast recompiles; never in release).
- The `Update` schedule has `ambiguity_detection: LogLevel::Warn` enabled permanently (`main.rs`), and the `bevy` dependency carries the `debug` feature so the resulting warnings name the conflicting systems directly instead of needing the feature toggled on ad hoc.

## Repository layout

The code lives in `app/` (package `bevy-planet`): a library crate (`bevy_planet`, root `lib.rs`) holding the game logic, plus a thin binary (`main.rs`) that only builds the `App` and adds plugins. Domain logic is organised as Bevy `Plugin`s exposed by the library; internal components and systems stay private to their module. Documentation and tooling stay at the repo root; there is no Cargo manifest at the root (the `Makefile` drives cargo inside `app/`). The library boundary keeps the logic testable behind a clean public API.

## Architecture (target)

> Not frozen. The learner designs the decomposition; this section records decisions as they are made. Standard patterns aimed for:
>
> - Bevy `Plugin` to modularise each major domain (planet, cameras, states, ...).
> - Bevy `States` to manage switching between the two view modes.

### Envisioned modules (to be confirmed in practice)

| Module / domain     | Role                                                                   |
|---------------------|------------------------------------------------------------------------|
| `star_system` (mvp) | Central star (light) + orbiting/spinning bodies, as `StarSystemPlugin` |
| `camera` (mvp)      | Orbit camera, as `CameraPlugin`; submodules below                      |
| `camera::input`     | Continuous per-frame input: drag to rotate, scroll to zoom             |
| `camera::picking`   | Click-to-select: double-click switches target, starts a transition     |
| `camera::sync`      | Applies `OrbitCamera` state to the `Transform` each frame              |
| `utils`             | Small standalone helpers (currently: easing functions)                 |
| (to define)         | Procedural generation (source of truth)                                |
| (to define)         | Orbital view (whole planet, low-res)                                   |
| (to define)         | First-person view (surface, high-res)                                  |
| (to define)         | Mapping between coordinate systems                                     |

`camera` only holds the shared state (`OrbitCamera`, `CameraTransition`) and the `Plugin`; each submodule covers a distinct kind of concern (continuous input with no state machine, versus an event-driven `Observer` with a debounce state machine, versus per-frame `Transform` sync) rather than being split by raw line count. The orbit camera and a future ground/first-person camera are deliberately not pre-organised around a shared abstraction yet -- the ground camera's actual shape is still unknown, so that decomposition is deferred to when that work actually starts.

## The two scales and the coordinate mapping

Core of the project. Two reference frames to connect:

- "Planet" coordinates: independent of rendering (for example spherical lat/long + altitude, or cube-sphere face coordinates).
- "World" coordinates: the Cartesian space of the Bevy scene (`Transform`).

The generation function takes a position on the sphere and returns the relief. It must be:

- deterministic (same seed -> same world),
- resolution-independent (sampleable at any density),
- unique (both orbital and first-person call it).

This is pure computation: to be covered by unit tests.

## State-of-the-art leads (to explore)

- Base sphere: cube-sphere vs icosphere.
- Noise: `noise` crate (Perlin/Simplex), fBm, ridged multifractal, domain warping.
- LOD: chunked LOD, quadtree on cube-sphere faces, geometry clipmaps.
- Precision at planetary scale: floating origin / origin rebasing.

## Technical decisions (lightweight ADR)

| Date       | Decision                                                                          | Reason                                                                |
|------------|-----------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| 2026-06-16 | Bevy 0.18, Rust edition 2024                                                      | Latest stable versions                                                |
| 2026-06-16 | `dynamic_linking` in dev only                                                     | Fast recompiles                                                       |
| 2026-06-16 | Deps at `opt-level=3`, our code at `opt-level=1`                                  | Fast dev build + smooth runtime                                       |
| 2026-06-18 | Target switches animate (position and look-at eased together, one shared `Timer`) | Avoid an instant, jarring snap when the orbit camera's target changes |
| 2026-06-18 | `camera.rs` split into `camera/{input,picking,sync}.rs`                           | Single file had grown past 200 lines, mixing distinct concerns        |
| 2026-06-20 | Bevy 0.19 upgrade                                                                 | Stay on latest stable, per policy                                     |
| 2026-06-20 | Removed `ClusterConfig::Single` (no clustering)                                   | Far-Z bound lagged a frame in 0.19, could leave the star unlit        |
| 2026-06-20 | `StarSystemSet` orders movement before camera sync                                | Fixed a `Transform` race causing camera jitter                        |
| 2026-06-20 | `ambiguity_detection: Warn` enabled permanently                                   | Catch future ordering conflicts early                                 |

## Known issues / environment

None currently known. The dev machine moved from WSL2 to a native Linux dual-boot setup (2026-06-17), which closed the WSL-specific GPU and input issues recorded in the logbook; hardware acceleration on the new machine still needs to be confirmed once heavier meshes / LOD make it relevant.
