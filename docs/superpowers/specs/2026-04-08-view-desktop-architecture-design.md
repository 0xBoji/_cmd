# _CMD Desktop Architecture Design

Date: 2026-04-08
Status: Draft for review

## Goal

Evolve `_cmd` from a high-signal observability TUI into an open-source desktop-grade terminal workspace for AI agents and RAI mesh operations.

The system should support:

- A single mission-control surface for many concurrent agent/workspace sessions
- Deep drill-down into one selected session without losing global awareness
- A desktop-native experience that can eventually replace fragmented use of Terminal, iTerm, Warp, and ad hoc monitoring panes
- A headless or SSH-friendly operating mode through a lightweight CLI/TUI

## Product Direction

`_cmd` should not stop at being "a nice dashboard". It should become a session orchestration cockpit with three delivery surfaces built on one shared core:

1. `core`
2. `cli`
3. `desktop`

This preserves current momentum while creating a clean path to a true desktop product.

## Workspace Shape

The repository should evolve into a Cargo workspace with these crates:

```text
_cmd/
  Cargo.toml                # workspace root
  crates/
    core/
    cli/
    desktop/
```

Recommended short-term migration path:

- Keep the current root crate working while extracting internals into `core`
- Introduce `cli` once the TUI depends only on `core`
- Add `desktop` only after `core` has stable session/state APIs

## Tier 1: `core`

`core` is the product brain. It owns runtime behavior and shared state. It must be UI-agnostic.

### Responsibilities

- PTY engine for real shell/session lifecycle
- Session registry for workspaces, tabs, panes, and agent lanes
- Mesh listener integration with `camp`
- Event ingestion and normalization
- State store for session metadata, recent output, token/cost metrics, and activity summaries
- Command API for actions like spawn, kill, focus, rename, resize, attach, and open
- Stream API for state snapshots and incremental updates

### Input Boundary

`core` accepts:

- user/system commands
- PTY bytes
- mesh events
- timer ticks

### Output Boundary

`core` emits:

- immutable state snapshots
- delta/update streams
- normalized PTY output frames
- structured event history

There must be no dependency on:

- terminal text layout
- GUI widgets
- browser rendering
- window management APIs

### Core Data Model

The core should eventually manage first-class concepts:

- Workspace
- Session
- Pane
- Agent
- TerminalBuffer
- EventLog
- ActivitySample
- FocusState
- FilterState

### Non-Goals

`core` should not:

- draw pixels
- format TUI borders
- decide desktop CSS
- depend on React/Svelte/egui widgets

## Tier 2: `cli`

`cli` is the lightweight terminal surface for operators, SSH access, and rapid validation.

### Responsibilities

- render `core` state into a TUI
- translate key input into `core` commands
- provide fallback mission-control access in headless environments
- validate layout, filters, focus, and session models before desktop polish

### UX Role

`cli` should be:

- fast to iterate
- reliable over SSH
- operationally dense
- good enough for daily power use

It does not need to win on:

- pixel-perfect visuals
- rich media embedding
- advanced drag-and-drop interactions

## Tier 3: `desktop`

`desktop` is the flagship application and the long-term "terminal replacement" surface.

### Responsibilities

- native windowing
- tabs and pane splitting
- mouse interaction
- smooth resizing
- richer layout composition
- visual polish
- optional embedded rich media or browser-powered panels when justified

### Recommended Delivery Path

Preferred path for speed and product leverage:

- Tauri v2 shell
- Rust backend connected to `core`
- Web UI frontend
- terminal rendering via `xterm.js` or equivalent

Alternative path for a pure Rust stack:

- `egui` or `Iced`
- terminal emulation integration through a Rust-native terminal engine

### Recommendation

Choose the pragmatic path first:

- `desktop = Tauri v2 + frontend UI + core`

Reason:

- fastest route to a polished desktop experience
- easiest path to tabbed workspaces and multi-pane layouts similar to the target preview
- easiest path to open-source adoption because contributors can work on frontend and Rust backend separately

Pure-Rust desktop can remain a future branch if performance or platform control becomes a defining advantage.

## Open Source Strategy

`_cmd` should be designed as an open-source product from the start.

### What to Open Source

- all workspace/session orchestration in `core`
- the full CLI crate
- the desktop shell and UI
- demo mode and mock datasets
- integration contracts for mesh listeners and future adapters

### What to Keep Stable

- event schema contracts
- command protocol between frontend and core
- state snapshot format
- session lifecycle model

### Open Source Design Principles

- easy local startup
- example/demo mode available without private infrastructure
- clear boundaries for contributors
- no unnecessary coupling between GUI and runtime logic

## Current Repo to Future Workspace Mapping

Current files map naturally into the new system:

- `src/app.rs` -> mostly `core` state and domain model material
- `src/listener.rs` -> `core` mesh/event ingestion layer
- `src/ui.rs` -> `cli` rendering layer
- `src/main.rs` -> temporary composition root, later split into CLI/desktop entry points

## Migration Plan

### Phase 1: Stabilize Current TUI

Goals:

- keep improving the current `ratatui` mission-control layout
- validate grid mode, focus mode, filters, search, and example-data workflows
- keep event ingestion and demo-mode loops stable

Success signal:

- the current single-crate project behaves like a believable session cockpit

### Phase 2: Extract `core`

Goals:

- move state, listeners, PTY/session runtime, and command model into `core`
- leave `cli` responsible only for drawing and input translation
- make the current TUI depend on `core` as if it were already a public library

Success signal:

- deleting the TUI should not break runtime/session logic tests

### Phase 3: Create `cli`

Goals:

- turn the existing TUI into its own crate
- cleanly bind it to `core`
- preserve SSH/headless usability

Success signal:

- `cli` can run standalone with real mesh input and demo input

### Phase 4: Create `desktop`

Goals:

- scaffold the desktop app shell
- connect desktop events to `core`
- reproduce the same state model already proven in CLI

Success signal:

- desktop app shows live multi-session data from `core` without rewriting the runtime model

### Phase 5: Desktop-First Experience

Goals:

- tabs/workspaces
- pane management
- session actions
- richer per-session history and tools
- terminal replacement posture

Success signal:

- users can run most daily agent workflows from `desktop` instead of juggling multiple terminal apps

## UX Modes

The product should support at least two stable modes across CLI and desktop:

### Grid Mode

- many live sessions at once
- quick recognition of status and activity
- session-wall / mission-control feel

### Focus Mode

- detailed inspection of one selected session
- recent logs, tools, metadata, output context, token/cost summaries

These modes already exist conceptually in the current TUI and should be preserved as core product primitives.

## Design Guardrails

- Do not couple PTY runtime logic to any single UI surface
- Do not make desktop-specific assumptions inside `core`
- Do not block current iteration speed by over-engineering the workspace split too early
- Do not abandon the CLI; it is the fastest validation surface and the headless surface
- Do not treat the desktop app as a rewrite; it must be a second consumer of the same core

## Recommended Next Step

The next implementation step should be:

1. convert the repo root into a Cargo workspace
2. create `crates/core`
3. move current state/listener code into `core`
4. keep the current TUI working by consuming `core`

This is the smallest move that unlocks the desktop future without slowing down product iteration.

## Decision

Adopt the 3-tier architecture:

- `core` as the runtime and state platform
- `cli` as the lightweight operational surface
- `desktop` as the flagship open-source desktop application

Desktop implementation recommendation:

- Tauri v2 + frontend UI, backed by `core`

This is the recommended architecture for _CMD moving forward.
