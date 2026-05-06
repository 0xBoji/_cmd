# _CMD Architecture: A Blueprint for Scalable Terminal UIs

> Context note: this is a future-oriented architecture note, not a claim that
> every part below already exists in `_cmd` today. It is a target/reference
> document influenced by Kaku.

This document outlines the macro-architecture of the `_cmd` terminal dashboard.
Inspired by modern terminal multiplexers, `_cmd` is designed to handle massive
scale, remote connections, and complex UI layouts without sacrificing
performance.

## Macro Project Structure (Future Architecture Suggestion)

```text
visual_interception_event_window/
├── crates/
│   ├── mux/         # Multiplexer: Client/Server Sync, Domain Sockets, Remote Sessions
│   ├── term/        # Terminal Engine: VT Parser, Logical Line Buffer, PTY bindings
│   ├── layout/      # Window Management: BSP Tree Grid, Split Panes, Resize Logic
│   ├── lua-api/     # Extensibility: Lua Event Bus, Shell Integration (OSC 1337)
│   ├── desktop/     # Native Client: egui + wgpu GPU renderer
│   ├── cli/         # TUI Client: ratatui terminal observer
│   └── web/         # Remote Client: Axum WebSocket + Embedded Web Dashboard
├── assets/          # Auto-built Web UI, SVGs, fallback fonts
├── docs/            # ADRs and manuals
└── Cargo.toml       # Optimized workspace manifest
```

## 1. The Multiplexer Model

Unlike a simple terminal app, Kaku is built more like `tmux` with a GUI.

- Decoupled state: a client renders while a background server owns session state.
- Local sockets: even local UI instances talk to a server boundary.
- Why it scales: crash resilience, reattach, and remote-ready architecture.

## 2. Asynchronous Lazy Concurrency

Kaku separates blocking operations from the 60fps render loop.

- Main thread: UI and drawing only.
- Background runtime: PTY I/O, parsing, fonts, networking.
- State sync: background tasks update shared state, UI reads snapshots.

## 3. BSP Tree Layout Engine

Handling complex split panes is easier with a binary tree than a static grid.

Takeaway: if `_cmd` evolves into richer terminal layouts, BSP is a strong
candidate.

## 4. Lua Event Bus

Kaku embeds Lua for deep customization.

Takeaway: a future extensibility layer could separate stable Rust internals from
user customization. This is not part of the current `_cmd` implementation.

## 5. Shell Integration via OSC 1337

Kaku injects shell scripts that emit invisible OSC sequences containing metadata
such as cwd, command text, or exit code.

Takeaway: if `_cmd` later needs richer shell awareness, structured shell-to-UI
signals are preferable to regex scraping.

## 6. Physical vs. Logical Buffer Separation

Kaku tracks soft wraps separately from logical output lines.

Takeaway: terminal copy/paste and transcript selection become much more correct
when the system preserves logical line identity.

## Application to `_cmd`

These ideas suggest a direction:

1. `core` acts as the shared product brain.
2. `cli`, `desktop`, and `web` become surfaces over shared state.
3. More specialized crates such as `mux`, `term`, or `layout` should appear
   only when the codebase actually needs those boundaries.
