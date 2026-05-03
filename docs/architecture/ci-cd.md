# CI/CD Architecture

This repository does not yet have a release pipeline checked in. The CI/CD
shape below is the target once distribution work begins.

## Validation Tiers

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo check --workspace --all-targets`

## Release Direction

- Build release artifacts with the workspace `release-opt` profile when binary
  size matters.
- Add platform matrix builds only when Linux, Windows, or packaged macOS
  support is actively maintained.
- Add dependency auditing after the dependency graph stabilizes enough for the
  signal to be useful.

## Guardrails

- Do not add CI-only tools or new dependencies before there is a checked-in
  workflow that uses them.
- Keep local verification commands documented in `CONTRIBUTING.md`.
