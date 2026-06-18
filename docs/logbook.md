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

Next

- Start the procedural planet generation, or extend the star system (more bodies, moons).
