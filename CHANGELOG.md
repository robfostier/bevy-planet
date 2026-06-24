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
- Procedural nebula skybox (`skybox.rs`): a 6-face cubemap generated on the CPU from an `Fbm<Perlin>` noise field, in place of a loaded HDRI.

### Changed

- Upgraded from Bevy 0.18.1 to 0.19.
- Retuned the star/planet/orbit scale and the star's emissive color/point light intensity for the new exposure settings.

### Fixed

- The star's point light no longer disappears when its small mesh leaves the camera's view frustum (it previously did, regardless of the light's range, because mesh and light shared a single frustum-culling bounding box on the same entity). The light now lives on its own child entity with no mesh, so it is never subject to that culling check in the first place.
- `PointLight`'s shadow flag, renamed by the Bevy 0.19 upgrade (`shadows_enabled` -> `shadow_maps_enabled`).
- The star's point light going dark whenever the orbit camera zoomed far enough out. The camera used `ClusterConfig::Single`, whose far-Z bound is, as of Bevy 0.19, computed from the previous frame's farthest visible object rather than the current frame's; a fast enough zoom-out could push the light past that lagging bound before it had a chance to grow back, after which it never recovered. The system only ever has a handful of lights, so clustering bought nothing here; removed the `ClusterConfig` component entirely.
- The orbit camera's target visibly jittering. `update_orbital` (which moves bodies) and `sync_orbit_camera_transform` (which reads a body's position for the camera) shared no execution-order guarantee, so Bevy's scheduler could run them in either order from one frame to the next -- apparently stable under 0.18, exposed as jitter by 0.19's more aggressive parallel scheduling. Body-movement systems are now grouped in a `StarSystemSet` that the camera's systems explicitly run after.

## [0.1.0] -- planned

First playable milestone targeted: a low-res planet visible in orbital view.
