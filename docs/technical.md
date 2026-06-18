# Technical documentation

> Living document: updated every session by the tutor. Describes HOW the project is built and WHY these choices were made.

## Stack

- Language: Rust (edition 2024)
- Engine: Bevy 0.18 (ECS, PBR rendering, winit)
- Dev build: `cargo run --features bevy/dynamic_linking` (dynamic linking for fast recompiles; never in release).

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
| (to define)         | Procedural generation (source of truth)                                |
| (to define)         | Orbital view (whole planet, low-res)                                   |
| (to define)         | First-person view (surface, high-res)                                  |
| (to define)         | Mapping between coordinate systems                                     |

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

| Date       | Decision                                         | Reason                          |
|------------|--------------------------------------------------|---------------------------------|
| 2026-06-16 | Bevy 0.18, Rust edition 2024                     | Latest stable versions          |
| 2026-06-16 | `dynamic_linking` in dev only                    | Fast recompiles                 |
| 2026-06-16 | Deps at `opt-level=3`, our code at `opt-level=1` | Fast dev build + smooth runtime |

## Known issues / environment

None currently known. The dev machine moved from WSL2 to a native Linux dual-boot setup (2026-06-17), which closed the WSL-specific GPU and input issues recorded in the logbook; hardware acceleration on the new machine still needs to be confirmed once heavier meshes / LOD make it relevant.
