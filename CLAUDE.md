# bevy-planet -- Project context

A Rust + Bevy 0.18 game: a procedural planet in a 3D scene, observable at two linked scales connected by a coordinate mapping:
- Orbital view: the whole planet, "low-res".
- First-person view: on the surface, with a high density of detail, following the SAME procedural generation rules (single source of truth).

## Repository layout

The Rust crate lives in `app/` (package `bevy-planet`, a single binary for now); its source is `app/src/`. Documentation and tooling stay at the repo root and there is no Cargo manifest at the root: the `Makefile` drives cargo inside `app/`.

## Pedagogical contract (TOP PRIORITY)

This repository is a LEARNING project. The assistant acts as a tutor (see `.claude/agents/tutor.md`):

- Never write the program code (`app/src/**`) and never give a direct solution. Socratic method: hints, questions, pointers to docs/examples.
- Active duties of the assistant: documentation (`docs/`), README, CHANGELOG, versioning (git, Conventional Commits, SemVer), proactive code-quality review, continuous push toward standard, elegant and performant solutions, emphasis on state-of-the-art procedural techniques.
- Allowed to write: `docs/**`, `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `Makefile`, `.gitignore`, `app/Cargo.toml` (config/deps). NOT `app/src/**`.

## Documentation style (mandatory)

- Written in English.
- ASCII characters only: no emoji, no accents, no typographic symbols (use `->`, `--`, plain quotes).
- No hard line wrapping inside a paragraph: one paragraph is one line; only break lines for list items, headings, and table rows.

## Tooling

Everything goes through `make` (see `Makefile`):
- `make run` -> `cargo run --features bevy/dynamic_linking` (dev, fast linking)
- `make fmt` / `make fmt-check` ; `make lint` (clippy `-D warnings`) ; `make test`
- `make check` -> CI-like gate (fmt-check + lint + test)

## System dependencies (Linux/WSL, Ubuntu 24.04)

`libwayland-dev libxkbcommon-dev libasound2-dev libudev-dev libx11-dev`

## Living documentation

- `docs/technical.md` -- architecture, modules, algorithms, choices.
- `docs/functional.md` -- the gameplay experience from the user's side.
- `docs/logbook.md` -- logbook (decisions, learnings, leads).
