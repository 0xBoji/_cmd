# Features

## Current

- Passive agent event ingestion from newline-delimited JSON.
- Synthetic demo mode for UI development without a live agent stream.
- Shared `core` state for agents, events, terminal sessions, and UI selection.
- Ratatui TUI dashboard with grid/focus modes, filters, search, roster details,
  event feeds, and activity summaries.
- Egui desktop dashboard sharing the same core state.
- Desktop terminal shell surface with local transcript rendering and search.
- Persisted desktop shell command history with recent-unique retention across app restarts.
- `cd` suggestions can reuse recent global shell history to jump back to previously visited directories, even outside the current working directory.
- The managed local desktop shell now enables zsh history retention options and emits lightweight OSC metadata for cwd, last command, and exit code tracking.
- A non-UI setup surface exists for generating and patching managed zsh integration: `setup-shell`, `print-shell-init`, and `reset-shell`.
- Read-only Axum API and WebSocket snapshots for LAN observation.

## Kaku-inspired Roadmap

These ideas are intentionally deferred until the codebase has real pressure for
them:

- BSP-style split-pane layout shared by desktop and web.
- Dedicated `term` crate for terminal transcript and PTY mechanics.
- Dedicated `mux` crate for reattachable or remote session state.
- Embedded static web assets for a richer browser dashboard.
- Shell metadata integration through structured events rather than output
  scraping.

See `docs/architecture/` for the architecture notes behind this roadmap.
