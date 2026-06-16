# Functional documentation

> Describes the experience from the PLAYER's side: what you see, what you do. No implementation detail. Living document.

## Concept

Explore a procedurally generated planet at two scales:

### 1. Orbital view

- The whole planet floats in space, lit by a star.
- The player can observe it and rotate it.
- "Low-res" rendering: the silhouette and the large structures (continents, major relief) are readable, without the fine detail.

### 2. First-person view

- The player "enters" the world and ends up on the surface.
- The terrain is much more detailed, but stays faithful to what was seen from orbit (same mountains, same continents).

### Transition

- A mechanism (to define) switches from orbit to surface and back.

## Controls (to define)

| Action      | Key | Status |
|-------------|-----|--------|
| (to define) | --  | TODO   |

## Target journey (milestones)

1. DONE A window opens.
2. DONE A lit sphere is visible in 3D.
3. TODO The sphere becomes a low-res procedural planet.
4. TODO You can rotate around it (orbital view).
5. TODO You can "enter": switch to first-person on the surface.
6. TODO The surface terrain is consistent with the orbital view.
