# Contributing to VIEW

Thanks for contributing to VIEW.

## Workspace layout

- `crates/core` contains runtime, mesh ingestion, and shared state
- `crates/cli` contains the ratatui mission-control client
- `crates/desktop` contains the native desktop shell

## Local development

### CLI demo mode

```bash
VIEW_DEMO=1 cargo run -p cli
```

### Desktop demo mode

```bash
VIEW_DEMO=1 cargo run -p desktop
```

## Verification

Run these before opening a PR:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Contribution rules

- Keep diffs small and reversible
- Prefer extracting shared logic into `core`
- Preserve strict log color mapping for `rai_level`
- Add or update tests when behavior changes
- Avoid new dependencies unless they unlock a clear product capability

## Architectural intent

VIEW is intentionally split into:

- `core` for runtime logic
- `cli` for terminal operation
- `desktop` for the desktop-native experience

Please do not move UI-specific code into `core`.
