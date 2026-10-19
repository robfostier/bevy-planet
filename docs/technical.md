# Technical documentation

> Living document: updated every session by the tutor. Describes HOW the project is built and WHY these choices were made.

## Stack

- Language: Rust (edition 2024)
- Engine: Bevy 0.20 (ECS, PBR rendering, winit)
- Shaders: WESL (`.wesl`), Bevy's shader language since 0.20: modules import each other with `import package::shaders::...;`, valued shader defs are read as `constants::NAME`, and conditionals are `@if(...)`.
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

| Module / domain         | Role                                                                         |
|-------------------------|------------------------------------------------------------------------------|
| `system`                | The system's inputs (`SystemParams`: seed, age, metallicity)                 |
| `star_system` (mvp)     | Central star (light) + orbiting/spinning bodies, as `StarSystemPlugin`       |
| `star_system::star`     | The star's look: surface and corona materials, physics, style                |
| `camera` (mvp)          | Orbit camera and its cinematic HDR look, as `CameraPlugin`; submodules below |
| `camera::input`         | Continuous per-frame input: drag to rotate, scroll to zoom                   |
| `camera::picking`       | Click-to-select: double-click switches target, starts a transition           |
| `camera::sync`          | Applies `OrbitCamera` state to the `Transform` each frame                    |
| `sky` (mvp)             | The sky sphere that composites its parts, as `SkyPlugin`                     |
| `sky::nebula`           | The nebula's properties, derived from the star and the system's age          |
| `sky::nebula::gas`      | The nebula's gas: noise-field cubemaps baked on the GPU, composed per frame  |
| `sky::nebula::globules` | Bok globules: dust in the nebula, backlit by its gas; shape atlas and mask   |
| `sky::starfield`        | Per-pixel procedural starfield (`StarfieldStyle`)                            |
| `utils`                 | Small standalone helpers                                                     |
| `dev_tools` (dev)       | Diagnostics, a single egui inspector window, scripted image capture          |
| (to define)             | Procedural generation (source of truth)                                      |
| (to define)             | Orbital view (whole planet, low-res)                                         |
| (to define)             | First-person view (surface, high-res)                                        |
| (to define)             | Mapping between coordinate systems                                           |

Shaders live in `app/assets/shaders/` and mirror the Rust tree: `sky/` (`sky.wesl`, `starfield.wesl`, `nebula/gas/`, `nebula/globules/`), `star_system/star/`, and `common/` for the modules every domain shares (`color.wesl`: Oklab and luminance; `gradient_noise.wesl`; `blackbody.wesl`; `cube.wesl`: a cubemap texel's direction for compute bakes). A module is a folder only when it holds several files; a parent module is `mod.rs` inside its folder. No file, Rust or WESL, exceeds 200 lines.

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

| Date       | Decision                                                                              | Reason                                                                                |
|------------|---------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|
| 2026-06-16 | Bevy 0.18, Rust edition 2024                                                          | Latest stable versions                                                                |
| 2026-06-16 | `dynamic_linking` in dev only                                                         | Fast recompiles                                                                       |
| 2026-06-16 | Deps at `opt-level=3`, our code at `opt-level=1`                                      | Fast dev build + smooth runtime                                                       |
| 2026-06-18 | Target switches animate (position and look-at eased together, one shared `Timer`)     | Avoid an instant, jarring snap when the orbit camera's target changes                 |
| 2026-06-18 | `camera.rs` split into `camera/{input,picking,sync}.rs`                               | Single file had grown past 200 lines, mixing distinct concerns                        |
| 2026-06-20 | Bevy 0.19 upgrade                                                                     | Stay on latest stable, per policy                                                     |
| 2026-06-20 | Removed `ClusterConfig::Single` (no clustering)                                       | Far-Z bound lagged a frame in 0.19, could leave the star unlit                        |
| 2026-06-20 | `StarSystemSet` orders movement before camera sync                                    | Fixed a `Transform` race causing camera jitter                                        |
| 2026-06-20 | `ambiguity_detection: Warn` enabled permanently                                       | Catch future ordering conflicts early                                                 |
| 2026-06-22 | `Hdr` + `Bloom` + `Tonemapping::TonyMcMapface` on the orbit camera                    | Realistic star bloom/exposure for the PBR pass                                        |
| 2026-06-22 | Star/planet scale retuned (star 2.0, planet 0.25, orbit 8.6)                          | Stay readable at the new exposure/bloom settings                                      |
| 2026-06-24 | Procedural cubemap `Skybox` (CPU, FBM Perlin noise), not a loaded HDRI                | Wanted a primordial-nebula look, no matching off-the-shelf asset                      |
| 2026-06-24 | `face_uv_to_direction` uses the OpenGL/Vulkan cube-face formula directly              | `CUBE_MAP_FACES`'s `looking_at`-derived basis mismatched face winding                 |
| 2026-06-24 | Skybox content Z-mirrored before writing                                              | Compensates `skybox.wgsl`'s own Z-negation at sample time                             |
| 2026-07-04 | `bevy-inspector-egui` panel exposing `SkyboxParams` (seed, frequency)                 | Tune the nebula look at runtime without recompiling                                   |
| 2026-07-04 | Nebula cubemap behind a reserved `SkyboxCubemap` handle, rebuilt on change            | Decouple generation from the camera; a single build path                              |
| 2026-07-04 | egui on a dedicated 2D overlay camera, not the orbit camera                           | Inside the HDR/`Bloom` pipeline the egui pass was drawn over by the skybox            |
| 2026-07-04 | Overlay camera carries `Hdr`, `auto_create_primary_context` disabled                  | Share the orbit camera's target format; avoid a duplicate egui context                |
| 2026-07-04 | Debug tooling extracted into a `dev_tools` module (`DevToolsPlugin`)                  | Keep `main` thin; gate all dev tooling out of release in one place                    |
| 2026-07-04 | One custom egui window via `ui_for_resource`, not per-resource quick plugins          | Single "Dev Tools" panel with collapsible sections; scales to many tools              |
| 2026-07-04 | Nebula density is a domain-warped FBM (after Inigo Quilez)                            | Blobby Perlin -> swirling filaments closer to the reference art                       |
| 2026-07-04 | Separate `warp` (smooth) and `density` (detailed) noise fields                        | The warp wants low octaves/frequency, the detail wants more                           |
| 2026-07-05 | Smoothstep contrast/levels remap on the density (`floor`/`ceil`)                      | Carve black voids out of the mid-gray FBM fog                                         |
| 2026-07-12 | Density is a ridged-multifractal structure times a smooth FBM mask                    | A single FBM read as flat fog; ridged noise gives filaments, the mask carves voids    |
| 2026-07-12 | Optional warp-amplitude mask modulating the domain warp per region                    | A uniform warp looked mechanical; masking it alternates calm and turbulent zones      |
| 2026-07-12 | `skybox::color` palette maps density to color (edge-to-core lerp x intensity)         | Replace the placeholder grayscale write with a tunable nebula color ramp              |
| 2026-07-12 | `SkyboxParams` split into per-stage structs                                           | Each filter stage stays independently toggleable and tunable in the inspector         |
| 2026-07-12 | Palette color driven by a separate hue field, not by density                          | Density-driven color is monochrome; a decoupled hue gives spatial color variety       |
| 2026-07-12 | Hue field stretched by its own levels before the palette lookup                       | FBM clusters near 0.5; the stretch fills the palette instead of a narrow slice        |
| 2026-07-12 | Global density mask sampled at the warped point (follows the warp)                    | Keep the voids coherent with the warped filaments instead of floating over them       |
| 2026-07-12 | Spatial palette hue drift rotating both colors in OkLCH                               | Vary the color scheme across the sky while keeping the local two-color relation       |
| 2026-07-12 | Hue field and drift nested under `PaletteParams`; hue reuses `MaskParams`             | Group all color information in one inspector section; hue has the mask's shape        |
| 2026-10-18 | Nebula bake ported to a GPU compute shader, reproducing the `noise` crate exactly     | The CPU bake took seconds per change; the GPU one is instant and pixel-identical      |
| 2026-10-18 | Sky drawn by an opaque far-plane sphere with a custom `Material`, not Bevy's `Skybox` | Composite the nebula, globules and per-pixel stars in one shader                      |
| 2026-10-18 | Stars computed per screen pixel from the view direction, not baked                    | Stars smaller than a cubemap texel alias when baked; per pixel they stay sharp        |
| 2026-10-18 | Star look driven by its physics (mass, age, metallicity) from a Sun-like base style   | One art-directed style, every other star derived from it                              |
| 2026-10-18 | Parent modules use the `mod.rs` layout; no file over 200 lines                        | Readable, navigable modules                                                           |
| 2026-10-19 | Bok globules as a per-pixel layer of the sky, backlit only by the nebula gas          | Dust at infinity, read in negative space, integrated with the gas behind it           |
| 2026-10-19 | Globule shapes baked once into an atlas with an exact signed distance (jump flood)    | Rims a fixed number of pixels wide whatever a globule's size; no per-pixel shape cost |
| 2026-10-19 | Sky tree by domain, mirrored between Rust and shaders (`sky/nebula/{gas,globules}`)   | Globules belong to the nebula; each domain owns its params, uniforms and shaders      |
| 2026-10-19 | Atlas sizes passed to the shaders as shader defs                                      | One source of truth instead of three hand-copied constants                            |
| 2026-10-19 | One `SystemParams` (seed, age, metallicity) drives the whole system                   | A system is reproducible from one seed; parts derive theirs by path, never by order   |
| 2026-10-19 | Base styles locked in code; the player only moves 0..1 system inputs                  | One art-directed look; every system is a derivation of it, not a new tuning           |
| 2026-10-19 | Gas composed per frame from baked noise fields (`look.wesl`)                          | Palette, evaporation and filling evolve with no bake; only the warp needs one         |
| 2026-10-19 | Age slides the warp's noise; globules ride it by their centre, shapes by their noise  | The gas flows continuously; globules follow it without being torn, and deform slowly  |
| 2026-10-19 | While the age keeps changing, preview fields (256) and an atlas slice per frame       | A dragged age slider costs ~8 ms a frame instead of ~55 ms                            |
| 2026-10-19 | A low-resolution mask of where globules may show, per tier                            | Most of the sky skips the globule search (globules 3.4 -> 1 ms at 4K)                 |
| 2026-10-19 | Bevy 0.20 upgrade, shaders ported to WESL                                             | Stay on latest stable, per policy; 0.20 replaced naga_oil with WESL                   |

## System inputs

A star system is set by `SystemParams` (`system.rs`): a seed, an age (0 at the star's birth, 1 at its end; the Sun's is 0.35) and a metallicity (0 metal-poor, 1 metal-rich; the Sun's is 2/3). They are the only inputs a player touches, all 0..1 sliders; the star's mass (`StarParams`, 0..1 on a log scale from 0.1 to 20 suns) is the player's other choice. Every part derives its own seed from the system seed by its path (`derive("sky/nebula/gas")`: FNV-1a of the path mixed with the seed through SplitMix64), so adding or reordering parts never shifts another's seed; tests pin the derived values.

Each part keeps a locked base style in code (`StarStyle`, `NebulaStyle`, `GlobuleStyle`, `StarfieldStyle`): the art-directed look of a Sun-like system at mid-life. The inputs only derive properties from it, with no other tuning: the star's `StarProperties` (temperature, luminosity, size), then the nebula's `NebulaProperties` (how far the star's light has cleared the cloud, and how warm its metals make the gas). Physical values only place the defaults; every input the player sees is a simple 0..1.

## Sky rendering

The sky is a unit sphere re-centred on the camera in its vertex shader and pushed to the far plane (reverse-Z depth 0), drawn by `SkyMaterial` (`sky/sky.wesl`) so it sits behind everything, whatever the drawing order. Its fragment shader composites, per screen pixel: the nebula gas (composed from one cubemap sample), the Bok globules in front of it, and the stars, dimmed by the gas they shine through and by the globules' transmittance. Its uniforms (`GpuSky`) gather each part's own: the starfield's, the gas's look, the globules'. The pixel's angular size (`pixel_angle`, from screen-space derivatives) drives every anti-aliasing decision, so stars and globule edges stay one pixel sharp at any resolution or field of view.

### GPU bakes

Five compute bakes feed the sky; each one runs only when its inputs change, from a job resource extracted to the render world with a generation counter:

| Bake               | Size                   | When                             | Cost (RTX 2060 Super)        |
|--------------------|------------------------|----------------------------------|------------------------------|
| Gas fields         | 1024 per face          | a face per frame once settled    | about 7.5 ms a face          |
| Gas fields preview | 256 per face           | each frame the age changes       | about 6.5 ms                 |
| Carry              | 128 per face           | with the fields                  | small                        |
| Style density      | 256 per face           | new seed, with its first preview | about 6 ms                   |
| Globule atlas      | 2048, four slices      | startup whole; a slice per frame | 7.7 ms whole, 1.9 ms a slice |
| Globule mask       | 512 per face, per tier | new seed (after the density)     | small                        |

The fields preview and the atlas slices keep a dragged age slider at about 8 ms of bakes a frame. Six frames after the age settles, the full fields are baked a face per frame, the preview shown until the last face is written, so the sky never mixes ages; a new seed goes the same way, its density baked with its first preview. No frame carries a whole bake. The render world reports the gas' and the atlas' baked generations back (an atomic shared through an extracted resource), so a face or a slice asked for while a pipeline is still compiling is never lost. The mask waits for the gas bake of the same seed (`NebulaBakeSet`, `NebulaBakeState::is_current`). Video memory: about 120 MB (fields 50 MB, atlas 34 MB and its one-slice scratch textures 21 MB, the rest small).

## Nebula gas

The gas's noise is baked by a compute shader (`sky/nebula/gas/bake.wesl`) that reproduces the `noise` crate's Perlin, Fbm and RidgedMulti exactly: their permutation tables are generated on the CPU, seeded as the crate does (from the gas's derived seed), and uploaded. The bake writes raw noise fields, not colours: the ridge, the voids mask's noise, the hue field's noise and the drift's noise, all sampled at the domain-warped direction (a two-level warp of a smooth FBM, after Inigo Quilez, its amplitude modulated per region by a warp mask).

The sky composes the gas from those fields every frame (`look.wesl`, uniforms `GpuNebulaLook`): the ridge's thin edges evaporated and its gamma, carved by the voids (levels, smoothstep, contrast), then coloured by two palette colours mixed by the hue field in Oklab (so the blend never passes through a muddy grey), both rotated in hue by the drift field, their chroma scaled by the saturation. So the nebula's evolution (evaporation, filling, warmth) costs a few sums per pixel and no bake.

The age alone needs a bake: it slides the warp's noise along a fixed direction (`SLIDE_SPAN`, the style at mid-life), so the gas flows and reshapes continuously as the system ages.

## Starfield

Stars are computed per screen pixel (`sky/starfield.wesl`): three layers (small, medium, hero) each lay a 3D grid of cells over the direction sphere, one candidate star per cell (PCG hashes). A star is kept only if its projection stays inside its own cell, which makes the 8 cells nearest to a pixel an exhaustive lookup. Each star is a Gaussian whose width never drops below the pixel footprint (energy-conserving, so small windows show faint stars dimmer rather than aliased); its colour is a blackbody tint, and only the brightest stars carry faint four-point sparkles. The layers were calibrated on full-sky captures against the Chernoschokov references (star counts per brightness bin and bright-star sizes within about 20%). The share of stars shown follows the system's age (`thinning`: more in a young system still in its star-forming region, fewer at the end); a star crossing its threshold fades in or out over its own margin, never pops.

The lookup is pruned without changing the image: a neighbour cell is visited only when the pixel lies within reach of it (its distance to the faces crossed, summed in squares), a star is rejected on distance before its brightness is drawn, then on its own glow, and a star past the layer's glow is kept only if its draw can make it sparkle. A star's reach is where its light falls below `GLOW_CUT` (a tenth of the darkest sRGB step on screen).

## Bok globules

Dust clumps at infinity, read in negative space against the nebula and backlit only by its gas (`sky/nebula/globules/`), computed per pixel as a small node graph, each node one parameter set:

- Scatter (`scatter.wesl`): three tiers of sizes, from `count_large` large globules to `count_small` small ones (geometric steps in between). Each tier is a grid over the direction sphere with one candidate per cell; a globule stays inside its cell (no two of a tier overlap) and a smaller one is dropped where a larger one stands. As the gas left changes, the globules whose draw nears its threshold fade in or out over their own margin.
- Ride (`carry.wesl`): the globules live in the style's sky (the gas at mid-life). The carry cubemap holds, per direction, where the style's gas now shows, from the warp's broad octave only; each globule moves with the gas at its centre, as one piece, so it follows the flow without being torn or trembling. The pixel searches its cells at its own point in the style's sky (`style_point`).
- Mask (`mask.wesl`, `globules::mask`): a 512-per-face cubemap, one channel per tier, 1 wherever one of its globules may show (within its reach, with the carry's margin and a texel's slack, not dropped for a larger one, over gas denser than `gas_faint`); baked for the most gas a cloud keeps, so it holds at every age. The pixel searches only the tiers it shows: most of the sky skips the search.
- Light side (`light.wesl`): the gradient of the style's density (its own cubemap, baked per seed) on a ring around the globule's centre; the globule's frame points away from it, and keeps its orientation at every age.
- Shape: one of the 64 tiles of an atlas (`atlas.wesl`, `globules::atlas`). Each tile holds a generated silhouette (`shape.wesl`: a family envelope -- compact, comet, cluster or filament -- with sub-lobes and torn-off knots, its edge torn by an fBm and plumped by round bumps) and its exact signed distance, computed by a jump flood confined to the tile, exact within `REACH` texels of the edge and clamped beyond (still a bound the rim tracing can step by). Seeds are packed as two 16-bit coordinates in one `u32`. As the system ages, the tearing noise slides (`SLIDE_SPAN`): the edges deform continuously while the family, the bumps and the size stay. The sky reads a tile (`tile.wesl`) instead of evaluating a shape per pixel.
- Placement: per pixel, the style's density behind, from `gas_faint` to `gas_full`, times `opacity`.
- Density: a thin edge, an opaque core and a grain, as an optical depth: the gas behind is absorbed (`exp(-tau)`), the dust glows dimly.
- Rim (`rim.wesl`): within `rim` pixels of the edge facing the light, found by sphere-tracing the distance field towards the light, full at the edge and fading over the `rim_falloff` share of its width; not on a hole's inner wall. Its colour is the hue of the gas behind at full saturation.
- Gilding (`gilding.wesl`): short strokes of saturated gold on a `gilding` share of the rim, as thick as the rim, cut into dashes by a noise fixed on the globule (after the painted accents of Supergiant's Hades), paler over cold gas.
- Front-lit: a `front_lit` share of the globules, drawn at random, read as lit from the front (deep in the nebula): a flat of light (`front_intensity`) instead of a rim.

## Star

The star (`star_system::star`) is a finely tessellated sphere with a custom surface material (`star_system/star/star.wesl`): a warm core-to-limb gradient, animated domain-warped granulation and dark ridged cracks detailed with proximity (`fbm_lod` adds octaves while they stay larger than a pixel), a bright chromosphere rim, and a vertex displacement so the silhouette is never a perfect circle. An additive corona billboard (`corona.wesl`) adds a soft glow that melts into the sky. Its output is display-linear HDR, so it always blooms the same.

The star is set by its mass (`StarParams`) and the system's age and metallicity. Simple fits (`physics`) give its temperature, luminosity and size (`StarProperties`) -- exactly the Sun's at the defaults, which unit tests check: it brightens slowly along its main sequence, then swells into a red giant past 0.77 of its life -- and `appearance` derives its look from the base style (`StarStyle`, a Sun-like star): colours re-centred on its blackbody colour in Oklab, brightness with its temperature, size as the entity's scale.

## Camera look

The orbit camera renders a cinematic HDR image, like an astrophotograph: `Hdr`, a fixed `Exposure`, `TonyMcMapface` tonemapping, a strong `Bloom` so the star burns beyond white, and a light `Vignette`.

## Debug tooling

The egui inspector (`bevy-inspector-egui`) lives in a dedicated `dev_tools` module (`DevToolsPlugin`), grouped so it can be gated out of release builds in one place. It draws a single "Dev Tools" `egui::Window` from one exclusive system (in the `EguiPrimaryContextPass` schedule), with a collapsible section per resource built via `bevy_inspector::ui_for_resource`; this replaces the per-resource `quick::ResourceInspectorPlugin` (one window each) and scales to many panels. Going manual means the plugin owns the pieces the quick plugin bundled: `EguiPlugin`, `DefaultInspectorConfigPlugin` (the egui widgets for standard types like `u32`/`f64`), and a `register_type` for each inspected type. It renders on a dedicated 2D overlay camera rather than on the orbit camera, because egui hooks into its host camera's render graph and inside the orbit camera's HDR + `Bloom` pipeline the egui pass ended up behind the skybox pass. The overlay camera renders after the orbit camera, does not clear, and carries `PrimaryEguiContext` while `EguiGlobalSettings::auto_create_primary_context` is turned off so bevy_egui does not also attach a context to the 3D camera. It must also carry `Hdr`: two cameras sharing a render target only share the intermediate "main texture" when they agree on its format, and `Hdr` is what selects `Rgba16Float`; that format is part of the texture-cache key in `prepare_view_targets`, so without it the two cameras get separate textures and the overlay's MSAA writeback composites egui over an empty (black) frame.

### Look-development capture

`dev_tools::capture` scripts runs through environment variables (each run exits once its capture is written): `CAPTURE_SIZE` renders the game camera offscreen at any size (4K included), `CAPTURE_CAMERA` pins it, `CAPTURE_STAR` sets the star's mass, `CAPTURE_SYSTEM` the system's seed, age and metallicity, `CAPTURE_SCREENSHOT` writes the image, and `CAPTURE_SKY` renders the whole sky as six 90-degree faces (LDR through the game's tonemapping, plus raw linear HDR) with their exact bases, so they can be reprojected to a panorama. The sky is on its own render layer (`SKY_LAYER`) so the sky cameras leave the bodies out.

### Profiling

Measure GPU time per pass with Bevy's `RenderDiagnosticsPlugin` and vsync off: with vsync, `frame_time` snaps to 16.7 or 33.3 ms and hides the real cost. A compute bake shows up only once it carries a pass span (`diagnostic_recorder().pass_span`) and runs every frame while measured. On the dev machine (RTX 2060 Super) at 4K, the main pass costs about 3.7 ms: stars about 1.5 ms, globules about 1 ms, gas about 0.2 ms, the star and planet about 0.9 ms; bloom and tonemapping under 1 ms. Check every optimisation against a full-sky panorama of the same system: it should leave it identical to the pixel.

## Known issues / environment

None currently known. The dev machine moved from WSL2 to a native Linux dual-boot setup (2026-06-17), which closed the WSL-specific GPU and input issues recorded in the logbook; hardware acceleration on the new machine still needs to be confirmed once heavier meshes / LOD make it relevant.
