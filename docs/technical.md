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
| `skybox` (mvp)      | Procedural nebula cubemap for the `Skybox` (FBM Perlin)                |
| `utils`             | Small standalone helpers (currently: easing functions)                 |
| `dev_tools` (dev)   | Diagnostics + a single egui inspector window (live tuning)             |
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

| Date       | Decision                                                                          | Reason                                                                     |
|------------|-----------------------------------------------------------------------------------|----------------------------------------------------------------------------|
| 2026-06-16 | Bevy 0.18, Rust edition 2024                                                      | Latest stable versions                                                     |
| 2026-06-16 | `dynamic_linking` in dev only                                                     | Fast recompiles                                                            |
| 2026-06-16 | Deps at `opt-level=3`, our code at `opt-level=1`                                  | Fast dev build + smooth runtime                                            |
| 2026-06-18 | Target switches animate (position and look-at eased together, one shared `Timer`) | Avoid an instant, jarring snap when the orbit camera's target changes      |
| 2026-06-18 | `camera.rs` split into `camera/{input,picking,sync}.rs`                           | Single file had grown past 200 lines, mixing distinct concerns             |
| 2026-06-20 | Bevy 0.19 upgrade                                                                 | Stay on latest stable, per policy                                          |
| 2026-06-20 | Removed `ClusterConfig::Single` (no clustering)                                   | Far-Z bound lagged a frame in 0.19, could leave the star unlit             |
| 2026-06-20 | `StarSystemSet` orders movement before camera sync                                | Fixed a `Transform` race causing camera jitter                             |
| 2026-06-20 | `ambiguity_detection: Warn` enabled permanently                                   | Catch future ordering conflicts early                                      |
| 2026-06-22 | `Hdr` + `Bloom` + `Tonemapping::TonyMcMapface` on the orbit camera                | Realistic star bloom/exposure for the PBR pass                             |
| 2026-06-22 | Star/planet scale retuned (star 2.0, planet 0.25, orbit 8.6)                      | Stay readable at the new exposure/bloom settings                           |
| 2026-06-24 | Procedural cubemap `Skybox` (CPU, FBM Perlin noise), not a loaded HDRI            | Wanted a primordial-nebula look, no matching off-the-shelf asset           |
| 2026-06-24 | `face_uv_to_direction` uses the OpenGL/Vulkan cube-face formula directly          | `CUBE_MAP_FACES`'s `looking_at`-derived basis mismatched face winding      |
| 2026-06-24 | Skybox content Z-mirrored before writing                                          | Compensates `skybox.wgsl`'s own Z-negation at sample time                  |
| 2026-07-04 | `bevy-inspector-egui` panel exposing `SkyboxParams` (seed, frequency)             | Tune the nebula look at runtime without recompiling                        |
| 2026-07-04 | Nebula cubemap behind a reserved `SkyboxCubemap` handle, rebuilt on change        | Decouple generation from the camera; a single build path                   |
| 2026-07-04 | egui on a dedicated 2D overlay camera, not the orbit camera                       | Inside the HDR/`Bloom` pipeline the egui pass was drawn over by the skybox |
| 2026-07-04 | Overlay camera carries `Hdr`, `auto_create_primary_context` disabled              | Share the orbit camera's target format; avoid a duplicate egui context     |
| 2026-07-04 | Debug tooling extracted into a `dev_tools` module (`DevToolsPlugin`)              | Keep `main` thin; gate all dev tooling out of release in one place         |
| 2026-07-04 | One custom egui window via `ui_for_resource`, not per-resource quick plugins      | Single "Dev Tools" panel with collapsible sections; scales to many tools   |

## Debug tooling

The egui inspector (`bevy-inspector-egui`) lives in a dedicated `dev_tools` module (`DevToolsPlugin`), grouped so it can be gated out of release builds in one place. It draws a single "Dev Tools" `egui::Window` from one exclusive system (in the `EguiPrimaryContextPass` schedule), with a collapsible section per resource built via `bevy_inspector::ui_for_resource`; this replaces the per-resource `quick::ResourceInspectorPlugin` (one window each) and scales to many panels. Going manual means the plugin owns the pieces the quick plugin bundled: `EguiPlugin`, `DefaultInspectorConfigPlugin` (the egui widgets for standard types like `u32`/`f64`), and a `register_type` for each inspected type. It renders on a dedicated 2D overlay camera rather than on the orbit camera, because egui hooks into its host camera's render graph and inside the orbit camera's HDR + `Bloom` pipeline the egui pass ended up behind the skybox pass. The overlay camera renders after the orbit camera, does not clear, and carries `PrimaryEguiContext` while `EguiGlobalSettings::auto_create_primary_context` is turned off so bevy_egui does not also attach a context to the 3D camera. It must also carry `Hdr`: two cameras sharing a render target only share the intermediate "main texture" when they agree on its format, and `Hdr` is what selects `Rgba16Float`; that format is part of the texture-cache key in `prepare_view_targets`, so without it the two cameras get separate textures and the overlay's MSAA writeback composites egui over an empty (black) frame.

## Known issues / environment

None currently known. The dev machine moved from WSL2 to a native Linux dual-boot setup (2026-06-17), which closed the WSL-specific GPU and input issues recorded in the logbook; hardware acceleration on the new machine still needs to be confirmed once heavier meshes / LOD make it relevant.
