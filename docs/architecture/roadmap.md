# Architecture Roadmap

This roadmap borrows from Kaku where it serves `view`, but keeps the passive
agent-dashboard mission as the filter.

## Current Foundation

- `core` maintains shared state, event ingestion, terminal session metadata,
  and the background engine.
- `cli` renders the ratatui dashboard.
- `desktop` renders the egui dashboard and local terminal shell surface.
- `web` exposes read-only REST and WebSocket snapshots.

## Near-term

- Keep event schemas stable and documented.
- Tighten desktop and CLI feature parity where the same state exists.
- Expand web read-only observability before adding any remote control surface.

## Later

- Extract `term` if terminal transcript handling becomes too large for `core`.
- Extract `layout` if split panes need shared geometry logic.
- Introduce a mux/server boundary only if sessions need to survive UI restarts
  or support remote attachment.

## Explicitly Deferred

- Lua runtime and user scripting.
- Shell OSC integration.
- Frecency directory jumping.
- Kaku-style package/release automation.
