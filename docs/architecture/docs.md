# Documentation Architecture

Documentation should describe shipped behavior clearly and separate roadmap
ideas from current features.

## Current Layout

- `README.md` is the product overview and quick start.
- `CONTRIBUTING.md` is contributor workflow and verification.
- `docs/cli.md` documents local run commands.
- `docs/configuration.md` documents environment variables and runtime knobs.
- `docs/features.md` documents implemented behavior plus explicit roadmap.
- `docs/keybindings.md` documents current CLI and desktop shortcuts.
- `docs/architecture/` stores architecture notes and Kaku learnings.
- `docs/superpowers/` stores planning artifacts from prior design work.

## Rules

- Mark future work as roadmap, not as shipped behavior.
- Keep Kaku-specific observations in architecture notes instead of user-facing
  feature docs.
- Add ADRs under `docs/architecture/decisions/` only for decisions that are
  expected to affect future contributors.
