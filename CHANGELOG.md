# Changelog

Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions following [SemVer](https://semver.org) (`0.x` while the project is in unstable development).

## [Unreleased]

### Added

- 3D scene rendering a lit sphere: a unit sphere with a `StandardMaterial`, a directional sun light, and a 3D camera.
- Animated star system (`StarSystemPlugin`): a central emissive star with a point light, plus a planet that orbits it (position derived from elapsed time) and spins on its own axis.
- Orbit camera (`CameraPlugin`): middle-mouse drag rotates around its target body (azimuth/elevation, elevation clamped to avoid flipping over the poles), scroll wheel zooms in and out within bounds derived from the target's radius, cursor is grabbed and hidden while dragging. The camera tracks the target's actual position, so it keeps following a body that moves (for example the orbiting planet). Defaults to the system's star, exposed by `StarSystemPlugin` through a small `SystemBodies` resource so the two plugins stay decoupled from each other's internal types.
- Click-to-select: double-clicking a celestial body (star or planet) switches the orbit camera's target to it, using Bevy's mesh picking backend.
- Animated target switching: the orbit camera now glides its position and look-at point smoothly to the new target (sine ease-in-out, shared timer) instead of snapping instantly, and resets its zoom to a default distance as part of the same motion.
- Library/binary split: game logic lives in a library crate (`bevy_planet`) exposed as Bevy plugins; the binary is a thin launcher.
- HDR rendering pipeline on the orbit camera: `Hdr`, photometric `Exposure`, `TonyMcMapface` tonemapping, and `Bloom`, as the first step of a PBR/visual-realism pass.
- Procedural nebula skybox (`skybox/`, since moved to the GPU, see below): a 6-face cubemap generated on the CPU, in place of a loaded HDRI. The scalar density multiplies a ridged-multifractal "ridge" structure by a smooth FBM density mask, sampled along a direction that is first optionally domain-warped (a two-level warp after Inigo Quilez, whose amplitude can itself be masked per region); optional levels/contrast/gamma passes carve the black voids and shape the falloff. Color is decoupled from density: a separate hue field (a warped FBM stretched by its own levels, so the palette sees a full range rather than the narrow slice FBM naturally spans) drives a two-color `Palette` lerp, scaled by density and intensity -- or grayscale when the palette is off. An optional spatial drift (a second masked FBM) rotates both palette colors' hue in OkLCH across the sky, with a tunable amplitude in degrees and a direction bias. Split into `params`, `field`, `color`, and `cubemap` submodules.
- Live-tuning inspector for the nebula skybox: every generation stage -- the domain warp and its mask, the ridge, the density mask, the palette (colors, hue field, and hue drift), and the global gamma/levels -- is exposed as a `SkyboxParams` resource and edited at runtime through a `bevy-inspector-egui` panel; the cubemap is held behind a shared `SkyboxCubemap` handle (reserved at build time) and rebuilt only when the parameters change. The inspector lives in a `dev_tools` module (`DevToolsPlugin`) that draws a single "Dev Tools" egui window on a dedicated overlay camera, so egui does not draw inside the HDR orbit camera's pipeline.

- GPU nebula bake: the nebula cubemap is baked by a compute shader that reproduces the `noise` crate's Perlin / Fbm / RidgedMulti exactly (permutation tables uploaded from the CPU), at 1024 per face, instantly on every parameter change; its palette is mixed and hue-rotated in Oklab.
- Sky sphere (`SkyPlugin`): an opaque far-plane sphere compositing the nebula gas, the Bok globules and the stars in one shader, replacing Bevy's `Skybox`.
- Per-pixel procedural starfield: three layers of stars on cell grids over the sky, blackbody colours, pixel-footprint anti-aliasing with energy conservation, sparkles on the brightest stars, dimmed by the nebula; calibrated against reference sky photos.
- Physically driven star: mass, age and metallicity give its temperature, luminosity, size and habitable zone (the Sun's at the defaults); a custom surface material (granulation, cracks, chromosphere, displaced silhouette) and an additive corona derive their look from a Sun-like base style.
- Cinematic camera look: exposure, bloom and vignette tuned so the star burns beyond white.
- Bok globules: dust clumps in the nebula, backlit by its gas -- tiers of sizes scattered over the sky, 64 generated shapes baked once into an atlas with exact signed distance fields (jump flood), a rim a fixed number of pixels wide with a falloff, gold gilding strokes on a share of the rims, a share of front-lit globules, placement from the gas density, and a seed. Each parameter is a single inspector slider.
- System inputs (`SystemParams`): a seed, an age and a metallicity (0..1) drive the whole system; every part derives its own seed by its path, and every base style is locked in code. The inspector shows the inputs and the derived star and nebula properties.
- A living sky: as the system ages, the nebula's gas flows (its warp's noise slides) and erodes under a bright star or fills under a dim one, metals warm its colours, the starfield thins out, and the globules drift with the gas while their edges evolve; every threshold fades instead of popping.
- Scripted look-development capture in `dev_tools`: offscreen renders at any size, pinned cameras, star inputs, and full-sky captures as six faces (LDR and linear HDR).
- Unit tests for the star's physics.

### Changed

- Upgraded from Bevy 0.19 to 0.20 and bevy-inspector-egui 0.37 to 0.38; every shader is now WESL (`.wesl`), Bevy's shader language since 0.20. Requires Rust 1.97.1 or later.
- Sky performance at 4K: main pass from 7.9 ms to 3.7 ms -- 4.9 ms with an identical image (a per-tier mask of where globules may show, tighter star pruning, a glow cut a tenth of the darkest sRGB step); age changes rebake only what they change (preview fields and atlas slices while the age keeps changing, then the full fields a face per frame, with no frame carrying a whole bake); the nebula is composed per frame from baked noise fields; the star dust layer, invisible in game, was removed (the starfield keeps three layers).
- The star is set by its mass alone (0..1, log scale); the system's age and metallicity age and colour it.

- Source tree reorganised by domain and mirrored between Rust and shaders: `sky/{nebula/{gas,globules},starfield}`, `star_system/star`, shared shader modules in `shaders/common/`; parent modules use the `mod.rs` layout and no file exceeds 200 lines.

- Upgraded from Bevy 0.18.1 to 0.19.
- Retuned the star/planet/orbit scale and the star's emissive color/point light intensity for the new exposure settings.

### Fixed

- The build on recent Rust, which rejects an untyped float literal falling back to `f32` (`float_literal_f32_fallback`) under `-D warnings`: the inspector sliders' speeds are typed.
- The star's point light no longer disappears when its small mesh leaves the camera's view frustum (it previously did, regardless of the light's range, because mesh and light shared a single frustum-culling bounding box on the same entity). The light now lives on its own child entity with no mesh, so it is never subject to that culling check in the first place.
- `PointLight`'s shadow flag, renamed by the Bevy 0.19 upgrade (`shadows_enabled` -> `shadow_maps_enabled`).
- The star's point light going dark whenever the orbit camera zoomed far enough out. The camera used `ClusterConfig::Single`, whose far-Z bound is, as of Bevy 0.19, computed from the previous frame's farthest visible object rather than the current frame's; a fast enough zoom-out could push the light past that lagging bound before it had a chance to grow back, after which it never recovered. The system only ever has a handful of lights, so clustering bought nothing here; removed the `ClusterConfig` component entirely.
- The orbit camera's target visibly jittering. `update_orbital` (which moves bodies) and `sync_orbit_camera_transform` (which reads a body's position for the camera) shared no execution-order guarantee, so Bevy's scheduler could run them in either order from one frame to the next -- apparently stable under 0.18, exposed as jitter by 0.19's more aggressive parallel scheduling. Body-movement systems are now grouped in a `StarSystemSet` that the camera's systems explicitly run after.

## [0.1.0] -- planned

First playable milestone targeted: a low-res planet visible in orbital view.
