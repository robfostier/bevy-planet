# Logbook

> One entry per session: what was done, decided, learned, and what comes next. Maintained by the tutor. Entries start once development begins.

## 2026-06-16 -- First scenes and project rails

Done

- Rendered a lit sphere (window, camera, directional light, `StandardMaterial`) -- first visible milestone.
- Laid the project rails: `make` workflow, versioned git hooks (guard `main`, `fmt-check`), GitHub Actions CI running the full gate, and a library/binary split.
- Implemented the `star_system` plugin: a central emissive star with a point light, a planet that orbits it (position derived from elapsed time) and spins on its own axis.

Learned

- Required components: `Mesh3d` auto-inserts a default `Transform`, so an entity meant to sit at the origin needs no explicit `Transform`.
- ECS composition over OOP: model behaviour as small data components (`Orbital`, `Spin`) attached to entities, not as nested structs; query by data, add marker components only when a system must single an entity out.
- Orbit as a pure function of elapsed time (`angle = angular_speed * t + phase`) is deterministic and read-only, versus accumulating an angle with `delta`.
- `Dir3` encodes the unit-vector invariant of a rotation axis at the type level.
- A `Plugin` is the single public facade of a module: its `build()` registers private systems, keeping components and systems encapsulated.

Next

- Decide the next step: orbital camera controls, or start the procedural planet (the project core).
- GPU is software-rendered under WSL (see Known issues in the technical doc); acceptable for now.

## 2026-06-17 -- Dev environment migration

Done

- Moved the dev machine from WSL2 to a native Linux dual-boot setup, closing out the WSL-specific GPU and input issues recorded in the previous entry and in the technical doc.

Learned

- The WSL2 software-rendering fallback (`llvmpipe`) and the mouse-event quirks that motivated the move were environment-specific, not project bugs -- worth remembering before chasing similar symptoms as code issues in the future.

Next

- Confirm hardware acceleration holds up once heavier meshes / LOD make it relevant.

## 2026-06-18 -- Orbit camera: target, zoom clamp, click-to-select

Done

- Gave the orbit camera an optional target (`OrbitCamera.target: Option<Entity>`), defaulted to the system's star via a small `SystemBodies` resource exposed by `star_system` -- the two plugins stay decoupled (`star_system` knows nothing about cameras, `camera` only reads what it needs).
- Bounded zoom using the target's `CelestialBody.radius`, and made the camera track the target's live `Transform` (translation and `look_at`), so it keeps following a body that moves.
- Added click-to-select: double-clicking a `CelestialBody` (via `MeshPickingPlugin`, an `Observer` on `Pointer<Click>`, with a `Local` debounce window to detect the second click) reassigns the orbit camera's target.
- Fixed a lighting bug: the star's `PointLight` would vanish whenever its small `Mesh3d` left the camera frustum, regardless of the light's `range`.

Learned

- Bevy's required components (`#[require(...)]` on a `#[derive(Component)]`) let one component pull in another's default automatically (`CelestialBody` -> `Pickable`) without repeating it at every spawn site -- and an explicit value provided at spawn always overrides the required default.
- Two `Query`s in the same system that both touch the same component type (one mutably) must be provably disjoint to Bevy, via `With`/`Without` filters -- two different required components are not enough on their own; Bevy needs an explicit `Without<T>` to prove it (confirmed by hitting error `B0001` before adding the filter).
- `check_visibility` (`bevy_camera`) computes frustum culling from a single shared `Aabb` per entity. An entity that combines a small `Mesh3d` with a `PointLight` gets culled as a whole once the mesh leaves the frustum -- the light's own `range` never enters into that test.
- First attempt at a fix, `NoFrustumCulling` on the combined mesh+light entity, traded one bug for another: `Aabb` (per its own doc) is only added to entities that lack `NoFrustumCulling`, and Bevy's mesh-picking ray cast (`MeshRayCast`) requires `Read<Aabb>` (not optional) to even consider an entity -- so the star stopped being clickable, even while clearly on screen. The actual fix was to stop sharing one entity for two different concerns: the light now lives on its own child entity (`ChildOf`) with no mesh, so it never gets an `Aabb` in the first place and is never culled, while the visual mesh entity keeps its normal `Aabb` and stays pickable.
- `Option::insert` always overwrites and returns a `&mut T` (`#[must_use]`, suggesting plain assignment if that reference is unused); `Option::get_or_insert` only fills an empty option and is the idiomatic one-liner for "default if absent".
- Bevy's `Pointer<E>` events (picking) auto-propagate up the entity hierarchy to the window by default. A global `Observer` (`app.add_observer`) re-runs on every step of that bubbling, not just on the originally clicked entity -- this silently wiped a `Local` debounce state every time, since the propagated invocation (window entity) always failed the "is this a `CelestialBody`" check. `On::propagate(false)` stops the bubbling and fixes it for good, rather than special-casing the propagated invocation.
- `Local<T>` state on an `Observer` persists correctly across triggers (the system is stored once on the `Observer` component and reused), so it is a legitimate place to keep small per-observer state like a double-click timer.

Done (continued)

- Animated target switching: the orbit camera now glides position and look-at point smoothly to a newly picked target (sine ease-in-out over a fixed duration) instead of snapping instantly, and its zoom resets to a default distance as part of the same motion.
- Split `camera.rs` (past 200 lines) into `camera.rs` (shared state, `Plugin`) plus `camera/input.rs`, `camera/picking.rs`, `camera/sync.rs`, grouped by kind of concern rather than by size.
- Added inline (`///`/`//!`) documentation across the camera module and `utils.rs`.

Learned (continued)

- One shared `Timer` can drive several interpolated values (position, look-at) at once, as long as they belong to the same transition -- no need for one timer per animated field. Bundling the timer with the values it drives into a single struct (`CameraTransition`) avoided several parallel `Option`-ish fields on `OrbitCamera` that had to be kept in sync by hand.
- `Transform::look_at` reads the transform's *current* translation to compute the rotation -- calling it before assigning the frame's new `translation` silently rotates the camera based on the previous frame's position, lagging one frame behind. Order matters: assign translation first, then `look_at`.
- A `Mut<T>` (the type yielded by iterating `&mut query`) is not a plain `&mut T`: each field access goes through a separate `Deref`/`DerefMut` call, so the borrow checker can fail to see two of its fields as disjoint even though they are. Reading the needed value out into a local *before* taking a `&mut` borrow on a different field sidesteps it.
- A private item is visible from the module it is defined in *and all of that module's descendants* -- so splitting a file into child submodules needs no visibility changes on the parent's types/constants; only the reverse direction (child exposing something to its parent) needs `pub(super)`.
- Process note: the tutor wrote directly into `app/src/**` twice this session, by explicit, repeated request of the learner (a large mechanical refactor, and a doc-comment pass blocked by a copy-paste limitation in the learner's terminal) -- both were one-off exceptions to the project's pedagogical contract, not a change to it.

Next

- Start the procedural planet generation, or extend the star system (more bodies, moons).

## 2026-06-20 -- Bevy 0.19 migration

Done

- Upgraded from Bevy 0.18.1 to 0.19.
- Fixed the one compile break: `PointLight::shadows_enabled` renamed to `shadow_maps_enabled` (Bevy 0.19 also split off a separate `contact_shadows_enabled`).
- Fixed two runtime regressions surfaced only after the upgrade: the star's light going dark on zoom-out, and the orbit camera's target visibly jittering.
- Enabled the `debug` Bevy Cargo feature and `ambiguity_detection: LogLevel::Warn` on the `Update` schedule, to name conflicting systems directly in the warning instead of guessing from symptoms.

Learned

- In Bevy 0.19, resources are now backed by singleton entity components internally; existing `Res`/`ResMut` system parameters keep working unchanged, the new representation mainly unlocks resource-side relationships and resource queries this project does not use yet.
- `ClusterConfig::Single` always resolves to `ClusterFarZMode::MaxClusterableObjectRange`, which as of 0.19 is computed from the previous frame's farthest visible clusterable object instead of the current frame's. A fast camera zoom can outrun that one-frame lag and push a light outside the cluster's far bound, after which the lagging estimate never grows back since the light is no longer there to extend it. For a scene with only a handful of lights, clustering is not needed at all; removing the `ClusterConfig` component entirely sidesteps the whole mechanism.
- Bevy's scheduler does not order two systems that touch the same component type unless told to, even across separate `Plugin`s -- they can run in either order from one frame to the next, and which order happens to "win" can change between engine versions as the scheduler's parallelism changes. This had apparently been incidentally stable under 0.18 and became visible jitter under 0.19's more aggressive parallel execution.
- `ScheduleBuildSettings`/`LogLevel` (ambiguity detection) live in `bevy::ecs::schedule`, not the prelude; the conflicting systems' real names only show up in the warning once the crate's `debug` Cargo feature is enabled.
- A `SystemSet` (`#[derive(SystemSet)]`) lets one `Plugin` order its systems relative to another `Plugin`'s private systems, by labeling a group of systems with a public marker type instead of exposing the systems themselves.
- Cross-plugin ordering reads more cleanly declared inside the `Plugin` that has the actual dependency (here, `CameraPlugin`, which already imports `star_system::SystemBodies`) than at the app-composition level in `main.rs`, since that plugin is already the one coupled to the other's data.

Next

- Resume work on the procedural planet generation, or extend the star system further, now back on Bevy 0.19.

## 2026-06-22 -- HDR pipeline

Done

- Added `Hdr`, `Exposure`, `Tonemapping::TonyMcMapface` and `Bloom` to the orbit camera.
- Retuned the star/planet/orbit scale and lighting for the new exposure settings: star radius 2.0 with a warm emissive tint and a point light intensity of 9e7; planet radius 0.25 with a rough, non-metallic material; orbit radius 8.6.

Learned

- `Exposure::ev100` sets photometric exposure (`exposure() = exp2(-ev100) / 1.2`); raising it dims the whole scene multiplicatively, so emissive/light intensities need recalibrating together with it rather than in isolation.
- `LinearRgba` and `Srgba` both get `Mul<f32>` from a shared `impl_componentwise_vector_space!` macro, not a per-type derive -- that is how `Srgba::rgb(..) * 10.0` compiles for a brighter emissive color.

Next

- Replace starfield HDRI texture asset with procedurally generated "primordial soup" Skybox

## 2026-06-24 -- Procedural nebula skybox

Done

- Added a `Skybox` driven by a CPU-generated 6-face cubemap (`skybox.rs`) sampling an `Fbm<Perlin>` noise field.
- Diagnosed and fixed a visible mirrored seam between the skybox's top/bottom faces and their neighbors.

Learned

- Bevy's `CUBE_MAP_FACES` `target`/`up` pairs are calibrated for rendering through a real camera, which has its own handedness corrections elsewhere in the pipeline. Deriving a basis from them via `Transform::looking_at` for direct CPU pixel writing does not guarantee adjacent faces share the same u/v winding direction -- two faces can each be internally self-consistent yet connect to each other mirrored, which produces no detectable value discontinuity (so screenshot pixel-diffing found nothing) while still looking visibly "flipped" to the eye. The fix was to implement the direction formula directly from the OpenGL/Vulkan cube-face-selection spec that Bevy's own source comment cites, instead of going through `looking_at`.
- `skybox.wgsl`'s fragment shader negates the Z component of the sampling ray to account for cubemaps' left-handed convention -- a second, independent handedness correction that has to be mirrored into the generated content as well, separate from the face/u/v formula above.

Next

- Replace the skybox's debug direction-colors with the real palette driven by `nebula_density`, then layer domain warping and a sparse star field.
- Wire up `bevy-inspector-egui`'s `ResourceInspectorPlugin` for live tuning of the nebula parameters.
- Resume after that: procedural planet generation, the project's core milestone.

## 2026-07-04 -- Nebula skybox: live inspector, dev tools, domain warping

Done

- Exposed the nebula's parameters as a `SkyboxParams` resource, edited live through a `bevy-inspector-egui` panel.
- Moved the cubemap behind a shared `SkyboxCubemap` handle, reserved at plugin-build time via `FromWorld` and filled by a `rebuild_skybox_cubemap` system gated on `resource_changed::<SkyboxParams>` (so it also bakes once on startup).
- Gave egui its own 2D overlay camera and disabled bevy_egui's automatic primary context, after a chain of rendering problems (see below).
- Extracted all dev tooling out of `main.rs` into a `dev_tools` module (`DevToolsPlugin`), and replaced the per-resource quick inspector with a single custom "Dev Tools" egui window that draws collapsible sections via `bevy_inspector::ui_for_resource`.
- Replaced the plain FBM density with a domain-warped field (after Inigo Quilez): a two-level displacement built from a smooth `warp` noise, then sampled by a detailed `density` noise -- turning blobs into swirling filaments.
- Exposed full FBM controls (seed, frequency, octaves, lacunarity, persistence) per field via a nested `NoiseParams`, plus a `warp_strength`, all grouped behind a `NebulaField` generator built once per rebuild.
- Split the growing `skybox.rs` into a `skybox/` module (`params`, `field`, `cubemap`), mirroring the `camera/` layout.
- Documented the previously bare skybox, star-system and inspector code inline.

Learned

- In bevy-inspector-egui 0.37, the quick `ResourceInspectorPlugin` registers its UI in the `EguiPrimaryContextPass` schedule and requires `EguiPlugin` to be added first -- it does not add it for you in this version.
- Adding a plugin runs its `build()` at App-construction time, not its systems: a resource set inside `build()` (here `EguiGlobalSettings::auto_create_primary_context = false`) lands before any `Startup` system spawns a camera, even if the camera's plugin was added earlier in the list. Plugin order drives `build()` order, not system execution order.
- bevy_egui attaches its primary context to the first camera created, so on the HDR orbit camera the egui pass was drawn over by the skybox pass. A dedicated overlay camera marked `PrimaryEguiContext` fixes that, but on its own it creates a second primary context and panics ("must have a unique schedule") unless `auto_create_primary_context` is turned off.
- An overlay camera that must not wipe the 3D scene needs `ClearColorConfig::None`; otherwise it repaints the whole viewport before drawing egui.
- Two cameras rendering to the same window only share the intermediate "main texture" when they agree on its format. That format is part of the texture-cache key in `prepare_view_targets`, and `Hdr` is what selects `Rgba16Float`. An HDR orbit camera plus a non-HDR overlay therefore get two separate textures, so the overlay's MSAA writeback reads an empty (black) texture and composites egui over black; giving the overlay a matching `Hdr` fixes it.
- Reading the installed Bevy source directly (the cache key in `prepare_view_targets`, the writeback condition in `msaa_writeback`) is what finally pinned the black-screen cause, after several wrong guesses drawn from symptoms alone.
- The quick `ResourceInspectorPlugin` silently bundles two things the manual `ui_for_resource` path does not: it `register_type`s the inspected type, and it adds `DefaultInspectorConfigPlugin` (which registers the `InspectorEguiImpl` egui widget for each standard type). Going manual, both become your job -- without them the panel first shows "not registered", then "u32 is #[reflect(opaque)], register an InspectorEguiImpl".
- Domain warping needs the displacement components decorrelated by distinct constant offsets; sampling the same value for each component collapses the displacement onto the (1,1,1) diagonal -- a directional smear instead of a swirl.
- The warp field and the density field want different settings (smooth vs detailed), which is why one shared `Fbm` is limiting: separate instances, each configured from its own `NoiseParams`, via the `MultiFractal` trait (`set_octaves`/`set_frequency`/`set_lacunarity`/`set_persistence`).
- A CPU-baked cubemap is fundamentally softer than Inigo Quilez's per-pixel shader renders. Matching that crispness ultimately means moving the generation into the skybox fragment shader (WGSL), resolution-independent, rather than baking a low-res texture.

Next

- Add the contrast/levels remap to carve black voids out of the mid-gray fog (the biggest remaining visual gap).
- Then the palette (color), a ridged detail layer, and a star field.
- Consider raising the bake resolution, or moving generation into `skybox.wgsl` for full sharpness.

## 2026-07-05 -- Contrast/levels: carving the voids

Done

- Added a smoothstep contrast/levels remap on the density (`density_levels_floor`/`ceil`), turning the mid-gray fog into deep black voids with the wisps standing out -- the moment the nebula reads as space.
- Factored `smoothstep` and `levels` into `utils` as a small easing/remap family alongside `ease_in_out_sine`.

Learned

- The type boundary is the `as f32` in `nebula_density`: the noise/warp geometry stays f64 (the `noise` crate's native side), everything downstream (levels, palette, pixels) is f32 (the `Rgba32Float`/`Color` side). Pick the type of the side you are on.
- `levels(value, floor, ceil)` is exactly the GLSL 3-argument `smoothstep`; keeping the 1-argument curve (`smoothstep`) in `utils` and the domain remap (`levels`) as a thin composition of it avoids duplicating the polynomial.
- Resisted making the easing helpers generic over the float type: Rust has no std `Float` trait, so it would mean a `num-traits` dependency and uglier literals -- premature until a second concrete f64 caller shows up.

Next

- The palette (density -> color) in a new `skybox/color.rs`: a cosine palette (after Inigo Quilez), tunable live.
- Then a ridged detail layer and a star field.
- Raise the bake resolution, or move generation into `skybox.wgsl` for full sharpness.

## 2026-07-12 -- Palette, warp mask, and studying the reference's slope filaments

Done

- Added the palette (`skybox/color.rs`): a `Palette` maps the density to a color by lerping an edge color to a core color, scaled by density and a global intensity -- shipped as a simple ramp rather than the planned cosine palette. The cubemap now writes color (or grayscale when the palette is off) instead of the placeholder grayscale.
- Added a warp-amplitude mask: an FBM sampled at the un-warped direction, shaped by optional levels and contrast, modulating the domain-warp displacement per region (calm vs turbulent zones).
- Reorganised `SkyboxParams` into one struct per filter stage (`Warp`/`Ridge`/`Mask`/`Palette`/`Gamma`/`Levels`/`Contrast`), each independently toggleable and tunable, and gave the ridge and masks their own optional levels/contrast/gamma shaping.
- Removed a dead import and completed the inline documentation across the whole skybox module and `utils` (module, struct, field and function docs).

Learned

- The reference (Aleksandr Chernoschokov's Substance Designer nebula) builds its filaments with a Slope Blur (an iterative directional blur along a gradient) and a flow-map-driven Vector Warp. Both are image-space, neighbourhood-based filters -- a different paradigm from our per-direction analytic sampling, where each texel is a closed-form function with no access to neighbours. Slope blur, highpass and median cannot be expressed as point functions; matching them faithfully would mean a texture-space post-process pass over the baked cubemap.
- Connected filaments come from aligning matter to a coherent vector field: slope blur advects along the gradient's streamlines, so the smeared matter forms unbroken threads. Ridged noise has no such field, so it fragments. The analytic route to connectivity is therefore a cellular structure or a coherent (curl/flow) warp -- not a "better" ridged noise.
- Worley/Voronoi cell edges (F2 - F1) give a connected wall network by construction: space is partitioned, and borders are shared between adjacent cells. The `noise` crate's `Worley` returns only F1 (nearest-site distance), and its neighbour traversal is pruned for F1, so a clean F2 - F1 needs a hand-rolled Voronoi (a full 3x3x3 scan keeping the two nearest sites). `Worley` is also not a `MultiFractal` (no octaves) and holds an `Rc`, so it is `!Send` unlike `RidgedMulti`.
- Curl noise (Bridson) yields a divergence-free, swirling displacement from the curl of a noise potential, computed by finite differences; being a static spatial field, it is exactly what a non-animated warp wants.
- A flow map is a static 2D (UV-space) warp field whose purpose is animation; on a per-direction sphere it fights the hairy-ball theorem (seams). Our `domain_warp` is already the analytic equivalent of a warp field -- what it lacks is a coherent field driving the displacement `r`.
- Verified the crate's behaviour by reading the installed `noise` source directly (the F1-only return, the F1-pruned traversal), rather than trusting memory.

Next

- Revamp the colors (next session's focus).
- If the ridged structure keeps reading as fragmented, take the analytic connectivity leads: a hand-rolled Voronoi F2 - F1 for connected walls, and/or a curl-noise warp for coherent filaments.
- Still open: raise the bake resolution, or move generation into `skybox.wgsl` for full sharpness.

## 2026-07-12 -- Color from a separate hue field

Done

- Decoupled color from density (Aleksandr's workflow): `sample_nebula` now returns a `NebulaSample` (density + hue); the hue is a separate FBM sampled at the warped point, and `color::nebula_color` lerps two palette colors by that hue, scaled by density.
- Added a `HueParams` stage (noise + levels + contrast + enabled), its levels stretching the hue's narrow FBM range onto the full palette; disabling it collapses the nebula to a single flat color.
- Made the global density mask follow the warp (sampled at the warped point) so the voids swirl with the filaments instead of floating over them.

Learned

- FBM/Perlin output clusters around its mean (central-limit; more octaves = tighter), so a raw field spans roughly 0.4-0.6, not 0-1. Gamma cannot fix this: anchored at 0 and 1, it slides the cluster darker/brighter without stretching it (slope ~1 at the midpoint). A levels remap -- Substance's "Auto Levels" / "Histogram Scan" -- is the right tool, rescaling [lo, hi] onto [0, 1].
- Vibrance has two independent axes. Saturation: a two-color RGB lerp muddies its midpoint toward gray. Hue variety: two endpoints of the same family stay monochrome. A cosine palette (Inigo Quilez, `a + b*cos(2pi(c*t+d))`) solves both -- each channel oscillates out of phase, sweeping saturated hues without the gray middle -- but its twelve abstract coefficients are hard to art-direct; a small hand-picked N-stop gradient is the intuitive middle ground.
- Decoupling hue from density is the key structural move: density gates presence (voids stay dark), a separate field carries hue, so color varies spatially independent of brightness.
- `MaskParams` did not fit the hue. A mask is multiplied into the density (identity 1 when off); a hue is a lookup input with no identity -- "off" picks a specific flat color, not "no effect". A distinct `HueParams` keeps the naming honest.

Next

- The palette lookup itself: a live-tuneable multi-stop gradient (or revisit the cosine palette) for the reference's teal/orange/magenta contrast.
- Bring the density-mask voids back into the composition, and add the star field.
- Still open: raise the bake resolution, or move generation into `skybox.wgsl` for full sharpness.

## 2026-07-12 -- Spatial hue drift of the palette

Done

- Added a `DriftParams` stage (noise + levels + contrast + enabled, plus `degrees` and `center`): `sample_nebula` now also returns a `drift` value in `[0, 1]`, and `color::nebula_color` rotates both palette colors' hue by `(drift - pivot) * degrees` before the hue lerp, the pivot being `0.5 - center * 0.5`.
- The hue rotation is done in OkLCH via Bevy's `Oklcha` (convert, add to `hue` with `rem_euclid(360.0)`, convert back), so lightness and chroma are preserved.
- Regrouped the color settings: the hue field (now a plain `MaskParams`, `HueParams` removed) and the drift both live under `PaletteParams`, giving one "palette" section in the inspector. Retuned the defaults: orange/blue palette colors, a single-octave hue field with strong contrast, an 80-degree one-way drift (`center` -1), ridge gamma 1.5, density-mask seed 18.

Learned

- Rotating hue in a perceptual space (OkLCH) instead of HSV keeps the perceived brightness stable while the tint changes; HSV hue shifts make yellows pop and blues sink.
- Applying the same rotation to both ramp endpoints preserves their local relationship (contrast, complementarity) while the global scheme drifts -- a cheap way to get large-scale color variety from a two-color palette.
- The pivot/center formulation separates amplitude (`degrees`) from direction (`center`): the rotation range stays constant, only where zero sits on the mask moves.
- The hue field ended up with exactly the mask's shape (noise + levels + contrast + enabled), so it reuses `MaskParams`; the earlier naming argument for a distinct `HueParams` was traded for less duplication, with the field name `hue` carrying the meaning.

Next

- A multi-stop gradient (or cosine palette) for the reference's teal/orange/magenta contrast.
- Add the star field; move generation into `skybox.wgsl` (see the GPU direction).

## 2026-10-18 -- GPU nebula, sky sphere, starfield and a living star

Done

- Ported the nebula bake to a GPU compute shader, reproducing the `noise` crate exactly (its permutation tables are generated on the CPU and uploaded): instant rebakes, pixel-identical to the CPU version.
- Replaced Bevy's `Skybox` with an opaque far-plane sphere and a custom material that composites the nebula and a per-pixel procedural starfield (four layers, blackbody colours, footprint anti-aliasing, faint sparkles on the brightest stars), calibrated on full-sky captures against reference photos.
- Gave the star a custom surface material and an additive corona, driven by its physics (mass, age, metallicity), and the camera a cinematic HDR look; chose the art direction (a white-hot star with a boiling limb).
- Added scripted image capture to `dev_tools` for look development (offscreen 4K renders, full-sky panoramas).
- Moved parent modules to the `mod.rs` layout and capped files at 200 lines.

Learned

- Stars smaller than a cubemap texel cannot be baked: point sampling gives random sizes and brightnesses. Computing them per screen pixel, with a width never below the pixel footprint and energy conserved, keeps them sharp and stable at any resolution.
- A 3D grid of cells over the direction sphere with one star per cell is an exhaustive 8-cell lookup only if each star stays inside its own cell.
- Look development goes faster from renders than from specifications: the art direction was chosen by comparing side-by-side captures.

Next

- Bok globules in the nebula: dust silhouettes backlit by the gas.

## 2026-10-19 -- Bok globules, architecture and performance

Done

- Built the Bok globules as a node graph of single-purpose masks (scatter, light side, shape, placement, density, rim), each validated white-on-black on full-sky panoramas before compositing.
- Baked 64 generated shapes once into an atlas with exact signed distance fields (jump flood confined to each tile), so rims are a fixed number of pixels wide on every globule and the sky no longer evaluates shapes per pixel.
- Added a rim falloff, gold gilding strokes on a share of the rims (after Hades' painted accents), front-lit globules as flats of light, and a seed.
- Reorganised the tree by domain, mirrored between Rust and shaders (globules now live in the nebula), split shared shader modules into `common/`, gave the globule uniforms named fields, passed the atlas sizes as shader defs, and added unit tests for the star's physics.
- Profiled the sky per pass and cut its main pass at 4K from 11.4 ms to 7.2 ms with no visible change.

Learned

- A rim of "N pixels" needs a true distance to the silhouette; a single-pass shader only has a noisy field, which makes rims huge on large globules. A distance field baked once (jump flood) gives an exact signed distance, and sphere tracing it towards the light finds a pixel's distance to the lit edge.
- With vsync on, `frame_time` snaps to 16.7 or 33.3 ms; real costs need GPU timestamps per pass (`RenderDiagnosticsPlugin`) with vsync off.
- The cheapest optimisations reject early without changing the result: compute per-tier constants once, draw a cell's occupancy before the rest of its hash, skip neighbour cells a star's glow cannot reach, and run the costly overlap check only under a silhouette.
- naga_oil substitutes imported identifiers everywhere, field names included: an imported function cannot share its name with a struct field.

Next

- The starfield is now the largest cost (about 3 ms at 4K): baking only the faint dust layer is the next lead.
- Back to the core: the procedural planet.

## 2026-10-19 -- A living system, and the sky's costs

Done

- Drove the whole system from one set of inputs (`SystemParams`: seed, age, metallicity, all 0..1), each part deriving its seed by its path; locked every base style in code, so the player only ever moves simple 0..1 inputs and every system is a derivation of the art-directed look.
- Made the sky evolve with the system: the gas is composed every frame from baked noise fields (evaporation, filling, warmth cost no bake), the age slides the warp's noise so the gas flows, the starfield thins out with age, and every threshold fades instead of popping.
- Redid the globules' evolution twice: a per-pixel ride through the gas tore and twisted them; they now ride the gas by their centre on the warp's broad octave only, and their shapes evolve by sliding their own tearing noise.
- Cut the sky's main pass at 4K from 7.9 ms to 4.9 ms with the image identical to the pixel (a per-tier mask of where globules may show, tighter star pruning, a higher glow cut), then to 3.7 ms by removing the star dust layer, judged invisible in game. Kept a dragged age slider near 8 ms of bakes a frame (preview fields, atlas slices) and the sky's video memory near 120 MB.

Learned

- Moving an object with a displacement field evaluated per pixel deforms it with the field's gradient; to move it rigidly, displace it by the field at one point (its centre), and keep its own evolution in its own parameters.
- A domain warp's fine octaves oscillate as its noise slides: following them makes a body tremble back and forth, the first octave alone drifts.
- Measure before and after every optimisation, with the patches checked: several "gains" this session were measurement errors (a patch script that silently stopped applying, frame-time estimates instead of GPU spans). Sharing Perlin's hash lookups and a cube-face star grid both measured slower and were reverted.
- Spread the full fields bake over six frames, a face each, the preview shown until the last: the 45 ms frame when the age settled is gone (frames stay under 11 ms). A 250 ms frame seen while measuring it was the capture tool's 4K screenshot, not the bake.
- Splitting the hero layer on the GPU, the few sparkling heroes on their own sparse grid, measured 0.4 ms slower: each layer costs a fixed setup per pixel (about 0.3 ms at 4K), more than the sparkle search it saved. Reverted.
- Baking the globules with the gas, at its 1024 per face, was tried: about 1 ms a frame saved, but a texel is 3 to 4 pixels at 4K, so silhouettes softened, thin rims became about one texel and depended on the screen's resolution, and it needed a second 50 MB map. Kept per pixel.
- A bake costs per change, a pass per frame: masks and caches that turn per-frame work into per-change work are where a static sky saves the most.

Next

- Back to the core: the procedural planet.
