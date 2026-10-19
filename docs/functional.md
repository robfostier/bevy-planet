# Functional documentation

> Describes the experience from the PLAYER's side: what you see, what you do. No implementation detail. Living document.

## Concept

Explore a procedurally generated planet at two scales:

### 1. Orbital view

- The whole planet floats in space, lit by a star.
- The player can observe it and rotate it.
- "Low-res" rendering: the silhouette and the large structures (continents, major relief) are readable, without the fine detail.
- The backdrop is a procedurally generated sky, not a fixed image: a nebula of colourful gas with oil-paint volutes and dark voids, a fine starfield (thousands of faint stars, a few bright ones with faint sparkles), and dark Bok globules -- clumps of dust silhouetted against the gas, a thin rim of light on the side facing the brightest gas, some touched with strokes of gold, a few lit from the front as flat shapes of light.
- The sky belongs to its star system, and the whole system follows from three inputs: a seed (each seed draws another nebula, other globules, other stars), an age and a metallicity. As the system ages, the gas slowly flows and reshapes, the globules drift with it while their torn edges evolve, a bright star erodes its nebula while a dim one leaves it thick, and the starfield thins out; metal-rich systems have warmer gas. Every change is continuous: nothing pops in or out.
- The star is alive: a white-hot disc with boiling granulation and dark cracks that sharpen as you come closer, an irregular, shimmering limb and a soft corona. It burns beyond white in a cinematic image (bloom, light vignette). Its colour, brightness and size follow its mass and age, from a red dwarf to a blue giant or a swollen red giant.

### 2. First-person view

- The player "enters" the world and ends up on the surface.
- The terrain is much more detailed, but stays faithful to what was seen from orbit (same mountains, same continents).

### Transition

- A mechanism (to define) switches from orbit to surface and back.

## Controls

| Action                      | Key                          | Status |
|-----------------------------|------------------------------|--------|
| Orbit the camera            | Middle mouse + drag          | DONE   |
| Zoom in / out               | Mouse wheel                  | MVP    |
| Switch the camera's target  | Double left click on a body  | DONE   |

> Switching target is not an instant cut: the camera glides smoothly from its previous position and orientation to the new target, and its distance resets to a default zoom level as part of the same motion.

## Target journey (milestones)

1. DONE A window opens.
2. DONE A lit sphere is visible in 3D.
3. DONE You can rotate around it (orbital view).
4. TODO The sphere becomes a low-res procedural planet.
5. TODO You can "enter": switch to first-person on the surface.
6. TODO The surface terrain is consistent with the orbital view.

> Current sandbox (a learning step, may change): a central star lights the scene while a planet orbits it and spins on its own axis.
