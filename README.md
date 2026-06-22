# bevy-planet

A Rust + [Bevy](https://bevyengine.org) 0.19 game exploring procedural planet generation at two linked scales.

> LEARNING project. The code is written by the learner; a tutor agent (`.claude/agents/tutor.md`) guides, documents and watches over quality, but never codes in his place.

## Vision

- Orbital view -- the whole planet, generated "low-res", that you can observe and rotate.
- First-person view -- you "enter" the world and end up on the surface, in a much more detailed terrain.
- Consistency -- both views share the SAME procedural generation function (single source of truth); the level of detail changes, not the relief. The core of the project is the mapping between the two coordinate systems.

## Status

Bootstrapping -- scaffolding in place. See `CHANGELOG.md`.

## Requirements

- Rust >= 1.96 (`rustc`, `cargo`) -- developed on the latest stable
- System dependencies, plus `clang` + `lld` (the build links with `lld` via `app/.cargo/config.toml`) on Linux Ubuntu 24.04+:

  ```bash
  sudo apt install -y libwayland-dev libxkbcommon-dev libasound2-dev libudev-dev libx11-dev clang lld
  ```

## Getting started

```bash
make run        # run the game (dev, dynamic linking)
make check      # fmt + clippy + tests (must pass before committing)
make help       # list all targets
```

The first build compiles all of Bevy (several minutes); subsequent builds are fast.

## Structure

```bash
bevy-planet/
|-- app/               # the Rust crate (the game, written by the learner)
|   |-- Cargo.toml     #   dependencies + build profile
|   |-- src/           #   game source code
|-- docs/              # living documentation
|   |-- technical.md   #   architecture, modules, algorithms
|   |-- functional.md  #   the gameplay experience
|   |-- logbook.md     #   session logbook
|-- .claude/agents/    # tutor agent definition
|-- Makefile           # dev workflow (drives cargo inside app/)
|-- CHANGELOG.md       # version history
|-- CLAUDE.md          # context + pedagogical contract
```

## Documentation

- [Technical doc](docs/technical.md)
- [Functional doc](docs/functional.md)
- [Logbook](docs/logbook.md)
