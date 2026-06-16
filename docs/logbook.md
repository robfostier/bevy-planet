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
