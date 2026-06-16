# Changelog

Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions following [SemVer](https://semver.org) (`0.x` while the project is in unstable development).

## [Unreleased]

### Added

- 3D scene rendering a lit sphere: a unit sphere with a `StandardMaterial`, a directional sun light, and a 3D camera.
- Animated star system (`StarSystemPlugin`): a central emissive star with a point light, plus a planet that orbits it (position derived from elapsed time) and spins on its own axis.
- Library/binary split: game logic lives in a library crate (`bevy_planet`) exposed as Bevy plugins; the binary is a thin launcher.

## [0.1.0] -- planned

First playable milestone targeted: a low-res planet visible in orbital view.
