# Changelog

Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions following [SemVer](https://semver.org) (`0.x` while the project is in unstable development).

## [Unreleased]

### Added

- 3D scene rendering a lit sphere: a unit sphere with a `StandardMaterial`, a directional sun light, and a 3D camera.
- Animated star system (`StarSystemPlugin`): a central emissive star with a point light, plus a planet that orbits it (position derived from elapsed time) and spins on its own axis.
- Orbit camera (`CameraPlugin`): middle-mouse drag rotates around its target body (azimuth/elevation, elevation clamped to avoid flipping over the poles), scroll wheel zooms in and out within bounds derived from the target's radius, cursor is grabbed and hidden while dragging. The camera tracks the target's actual position, so it keeps following a body that moves (for example the orbiting planet). Defaults to the system's star, exposed by `StarSystemPlugin` through a small `SystemBodies` resource so the two plugins stay decoupled from each other's internal types.
- Library/binary split: game logic lives in a library crate (`bevy_planet`) exposed as Bevy plugins; the binary is a thin launcher.

## [0.1.0] -- planned

First playable milestone targeted: a low-res planet visible in orbital view.
