# Technical documentation

> Living document: updated every session by the tutor. Describes HOW the project is built and WHY these choices were made.

## Stack

- Language: Rust (edition 2024)
- Engine: Bevy 0.18 (ECS, PBR rendering, winit)
- Dev build: `cargo run --features bevy/dynamic_linking` (dynamic linking for fast recompiles; never in release).

## Repository layout

The Rust crate lives in `app/` (package `bevy-planet`, a single binary for now); documentation and tooling stay at the repo root. There is no Cargo manifest at the root: the `Makefile` drives cargo inside `app/`. If a pure, testable generation library later needs to be separated from the Bevy app, `app/` can be promoted to a Cargo workspace at that point (do not pre-build it).

## Architecture (target)

> Not frozen. The learner designs the decomposition; this section records decisions as they are made. Standard patterns aimed for:
>
> - Bevy `Plugin` to modularise each major domain (planet, cameras, states, ...).
> - Bevy `States` to manage switching between the two view modes.

### Envisioned modules (to be confirmed in practice)

| Domain      | Role                                    |
|-------------|-----------------------------------------|
| (to define) | Procedural generation (source of truth) |
| (to define) | Orbital view (whole planet, low-res)    |
| (to define) | First-person view (surface, high-res)   |
| (to define) | Mapping between coordinate systems      |

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

### GPU acceleration under WSL2 (software rendering fallback)

On the current dev machine (WSL2, Ubuntu 24.04, NVIDIA GeForce RTX 2060 SUPER) Bevy renders with `llvmpipe` (CPU software rasterizer), logged as warning `b0006`. This is acceptable for the early, lightweight milestones; it must be revisited before heavy meshes / LOD.

Diagnosis (2026-06-16):

- The GPU is reachable: `GALLIUM_DRIVER=d3d12 glxinfo`/`eglinfo` both report `D3D12 (NVIDIA GeForce RTX 2060 SUPER)`, so Mesa's `d3d12` Gallium driver gives hardware OpenGL.
- Mesa defaults to `llvmpipe` because WSL exposes no DRM node; the `d3d12` driver is only used when forced (`GALLIUM_DRIVER=d3d12` / `MESA_LOADER_DRIVER_OVERRIDE=d3d12`).
- Hardware Vulkan is unavailable: only `lavapipe` (software) is present; there is no Dozen (`dzn`) ICD or library on disk (`find /usr -iname '*dzn*'` is empty). Hardware Vulkan in WSL goes through `dzn` (Vulkan-on-D3D12), which Ubuntu's `mesa-vulkan-drivers` does not ship here.
- wgpu's GL backend cannot grab the working `d3d12` adapter under WSLg: `WGPU_BACKEND=gl GALLIUM_DRIVER=d3d12` still panics with "Unable to find a GPU" (the windowed EGL path fails with `DRI3 error: Could not get DRI3 device`, a known wgpu-on-WSL limitation).

Future options (when performance matters):

- Install a Mesa build that ships the `dzn` Vulkan driver (PPA or source) to get hardware Vulkan, Bevy's preferred backend.
- Build/run on native Windows or native Linux, where the GPU drivers expose hardware Vulkan directly.
