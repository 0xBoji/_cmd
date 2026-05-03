# `view` Architecture

`view` is a passive observability surface for local AI coding agents. Its core
job is to ingest structured events, maintain shared state, and render that state
through terminal, desktop, and web surfaces without interfering with observed
processes.

## Current Workspace

```text
visual_interception_event_window/
├── crates/
│   ├── core/      # domain state, engine, listener, terminal session model
│   ├── cli/       # ratatui + crossterm dashboard
│   ├── desktop/   # egui/eframe desktop dashboard and terminal shell
│   └── web/       # axum REST + WebSocket API
└── docs/
```

This is the smallest useful Kaku-inspired scaffold for the current codebase.
Directories such as `mux`, `term`, `window`, `website`, and `lua-api-crates`
are intentionally not created yet because there is no owned implementation for
those boundaries.

## Runtime Shape

- `core` owns agent registry state, terminal session state, UI selection state,
  demo/live event ingestion, and background engine actions.
- `cli` and `desktop` are renderers over shared state. They should keep input
  handling responsive and avoid doing blocking work in render paths.
- `web` exposes read-only snapshots over HTTP and WebSocket so another client
  can observe the same state.

## Future Boundaries

Add new top-level crates only when code needs them:

- `crates/mux` when sessions need a client/server boundary or reattachable
  background process.
- `crates/term` when terminal parsing, PTY handling, or transcript storage
  grows beyond the current `core::terminal` module.
- `crates/layout` when split-pane geometry needs BSP-style layout tests shared
  by desktop and web.
- `website/` when a real documentation or product site exists.
- `assets/` when the app has source assets that are embedded or packaged.

## Non-goals for This Scaffold

- No Lua configuration layer yet.
- No shell integration scripts yet.
- No copied Kaku dependency tree.
- No empty tracked directories just to match the reference screenshot.
