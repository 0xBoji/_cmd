# visual_interception_event_window (`view`)

Passive, high-signal terminal observability for the Rust Agent Infrastructure (RAI) ecosystem.

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](#installation)

> Think of `visual_interception_event_window` as the operator's glass cockpit for a local agent swarm:
> it subscribes to live mesh state, turns newline-delimited JSON events into a premium multi-panel TUI,
> and lets you inspect who is online, what they are doing, and where errors are emerging without interfering with execution.

---

## Table of Contents

- [What this crate is](#what-this-crate-is)
- [Why it exists](#why-it-exists)
- [Who should use it](#who-should-use-it)
- [Who should not use it](#who-should-not-use-it)
- [Status](#status)
- [TL;DR Quickstart](#tldr-quickstart)
- [Installation](#installation)
- [Running the dashboard](#running-the-dashboard)
- [The mental model](#the-mental-model)
- [UI layout and interaction model](#ui-layout-and-interaction-model)
- [Keyboard controls](#keyboard-controls)
- [Event ingestion contract](#event-ingestion-contract)
- [Demo mode](#demo-mode)
- [Repository layout](#repository-layout)
- [Development and verification](#development-and-verification)
- [Limitations and non-goals](#limitations-and-non-goals)
- [Roadmap / likely next improvements](#roadmap--likely-next-improvements)

---

## What this crate is

`visual_interception_event_window` is the **Observability Layer (Pillar 8)** of the local **Rust Agent Infrastructure (RAI)** ecosystem.

The repo is now organized as a Cargo workspace:

- `view-core` — runtime, mesh ingestion, state, demo mode, and session logic
- `view-cli` — terminal mission-control surface
- `view-desktop` — desktop shell bootstrap for the future GUI app

It is a Tokio-driven `ratatui` dashboard that:

- watches live agent presence from the RAI mesh,
- renders a zero-flicker TUI with overview cards, roster views, drill-down panes, and event feeds,
- tracks agent activity over a rolling 50-sample window,
- highlights status, error level, branch, role, token usage, and metadata in real time,
- supports grid and focus workflows for swarm-scale monitoring.

Today the CLI surface lives in `view-cli`. The product contract names the eventual operator-facing binary as **`view`**; until that rename/publish step lands, the project is best run locally with `cargo run -p view-cli`.
The desktop shell now lives in `view-desktop` and shares the same backend through `view-core`.

---

## Why it exists

Once the rest of the RAI stack is online, the next bottleneck stops being execution and starts being visibility.

Without a dedicated observability surface, a local agent swarm quickly becomes opaque:

- you know agents are running, but not which ones are healthy,
- logs are scattered across terminals and panes,
- "busy" vs "offline" vs "stalled" becomes guesswork,
- event streams from `camp`, `tick`, `garc`, or `wasp` are hard to correlate mentally,
- operators end up tailing raw JSON when they should be making decisions.

`view` exists to make swarm observability:

- **passive** instead of control-plane invasive,
- **real-time** instead of post-hoc,
- **high-density** without becoming unreadable,
- **operator-friendly** while still grounded in strict machine-readable event contracts.

---

## Who should use it

This crate is a good fit if you are building or operating:

- local multi-agent coding systems,
- RAI-based workflows using `camp`, `garc`, `tick`, or `wasp`,
- operator consoles for LAN-first autonomous tooling,
- terminal-native demos where live system state matters,
- swarm debugging flows where fast status inspection beats raw log tailing.

It is especially useful when you need to answer questions like:

- “Which agents are alive right now?”
- “Which branch or project is this worker on?”
- “Where did the latest warning/error come from?”
- “Is the mesh active, or are we only seeing stale state?”

---

## Who should not use it

This is **not** the right tool if you need:

- a control plane that mutates remote agents,
- a browser-based telemetry dashboard,
- long-term metrics retention or historical querying,
- a centralized observability backend with auth and multi-tenant access,
- a replacement for structured log storage or tracing systems.

`view` is intentionally a **passive local observer**. It visualizes what the mesh already knows; it does not orchestrate, persist, or enforce policy.

---

## Status

Current implementation includes:

- a 60 FPS async render loop built with `ratatui` + `crossterm`,
- RAII terminal cleanup to restore raw mode and alternate screen on exit,
- live CAMP ingestion via a background `camp watch` child process,
- a built-in demo dataset (`VIEW_DEMO=1`) for UI iteration without a live mesh,
- dual presentation modes: **Grid** and **Focus**,
- filter cycling across **all / busy / active / offline**,
- inline search across agent id, project, role, branch, and instance name,
- agent drill-down for role, branch, tokens, addresses, and selected metadata keys,
- recent-event feed with level-aware colors (`info`, `warn`, `error`, `success`),
- rolling activity sparklines and event buffering,
- unit tests covering listener parsing, view-state behavior, and key rendering invariants.

The current repo is intended for **local development and local-network operation**.

---

## TL;DR Quickstart

If you just want the shortest path to seeing the UI:

```bash
cd visual_interception_event_window
VIEW_DEMO=1 cargo run -p view-cli
```

That gets you:

- a fully populated demo swarm,
- live-updating overview cards,
- multi-agent tiles and feed panels,
- keyboard navigation for filters, search, and focus changes.

If you already have `camp` running on your machine and want real mesh data instead:

```bash
cd visual_interception_event_window
cargo run -p view-cli
```

If you want the new native desktop shell instead:

```bash
cd visual_interception_event_window
VIEW_DEMO=1 cargo run -p view-desktop
```

---

## Installation

This crate is currently easiest to use directly from the repository.

**Run in-place:**
```bash
cargo run -p view-cli
```

**Run the desktop shell:**
```bash
cargo run -p view-desktop
```

**Install locally from source:**
```bash
cargo install --path crates/view-cli
```

**Useful companion dependency:**
- `camp` should be installed and available in `PATH` for live mesh mode.

If you are only iterating on the interface, demo mode avoids all external runtime dependencies.

---

## Running the dashboard

### 1. Demo mode (fastest path)

```bash
VIEW_DEMO=1 cargo run -p view-cli
```

`VIEW_DEMO` accepts common truthy values such as:

- `1`
- `true`
- `yes`
- `on`
- `demo`

Demo mode publishes a synthetic swarm containing multiple agent roles, projects, statuses, event levels, token counts, and metadata fields such as model, cwd, last file, and last tool.

### 2. Live mesh mode

```bash
cargo run -p view-cli
```

### 3. Desktop shell

```bash
VIEW_DEMO=1 cargo run -p view-desktop
```

The desktop shell is currently an early native GUI backed by `view-core`. It already supports:

- mission-control grid mode
- focus mode
- shared demo/live backend wiring
- clickable session selection
- filter and search controls

In live mode, the dashboard spawns:

```bash
camp watch
```

and consumes its stdout as a newline-delimited event stream.

That means:

- `camp` must be installed and in your `PATH`,
- CAMP output must stay machine-readable on stdout,
- stderr noise is intentionally suppressed so the TUI stays clean,
- incoming records are interpreted as either snapshot payloads or lifecycle events.

If `camp` is missing, startup fails with an explicit process-launch error.

---

## The mental model

The easiest way to reason about `view` is:

1. **Subscribe to a live swarm stream**
   - either from `camp watch` or from the built-in demo source.

2. **Project mesh state into operator state**
   - agents become rows / tiles / drill-down targets,
   - events become live feed entries,
   - metadata becomes context for decisions.

3. **Render the dashboard every frame without blocking ingestion**
   - input, state updates, and redraws are decoupled,
   - the UI stays responsive even while events continue flowing.

4. **Navigate between swarm overview and per-agent focus**
   - use filters, search, and selection to shrink noise,
   - move from fleet health to individual diagnosis quickly.

5. **Observe — do not intervene**
   - `view` never mutates peer state,
   - it only reflects observed state and recent signals.

---

## UI layout and interaction model

The interface is split into four major zones:

### 1. Header bar
Shows current stream state (`AWAITING`, `TRACKING`, `LIVE`) plus online/busy/offline counts, total events, and current mode.

### 2. Overview cards
Four stat panels summarize:

- mesh health,
- event level distribution,
- focused agent/filter context,
- latest signal source and timestamp.

### 3. Workspace area
Two presentation modes are available:

- **Grid mode** — a high-density multi-agent wall for quick scanning.
- **Focus mode** — a roster on the left, activity sparkline + drill-down summary + scoped live feed on the right.

### 4. Footer help bar
Keeps the most important controls visible at all times.

The dashboard also keeps:

- a rolling **50-sample activity timeline** per agent,
- a bounded **100-event** recent-event buffer,
- metadata prioritization for `cwd`, `model`, `last_file`, `last_tool`, `messages`, and `cost`.

---

## Keyboard controls

### Global navigation

- `q` — quit
- `Ctrl+C` — quit
- `Tab` — toggle **Grid / Focus** mode
- `j` or `↓` — move selection forward
- `k` or `↑` — move selection backward
- `PageDown` — jump selection forward by one page block
- `PageUp` — jump selection backward by one page block
- `Home` — select first visible agent
- `End` — select last visible agent
- `f` — cycle filters: `all -> busy -> active -> offline -> all`
- `Esc` — clear search query

### Search mode

- `/` — enter search mode
- type text — filter by **agent id / project / role / branch / instance name**
- `Backspace` — delete one character
- `Enter` — leave search mode while keeping the current query applied
- `Esc` — clear the query and exit search mode

---

## Event ingestion contract

The runtime understands two CAMP-oriented shapes from stdin/stdout line payloads:

### Snapshot payload
Used to seed the roster with all currently visible agents.

```json
{
  "kind": "snapshot",
  "agents": [
    {
      "id": "agent-01",
      "instance_name": "agent-01.rai",
      "role": "executor",
      "project": "wasp",
      "branch": "main",
      "status": "busy",
      "capabilities": ["observe", "stream-json"],
      "port": 4100,
      "addresses": ["127.0.0.1:4100"],
      "metadata": {
        "tokens": "24000",
        "rai_component": "wasp",
        "rai_level": "info",
        "log": "Execution window validated successfully"
      }
    }
  ]
}
```

### Lifecycle event payload
Used for joined / updated / left transitions.

```json
{
  "kind": "updated",
  "origin": "camp",
  "reason": null,
  "previous": null,
  "current": {
    "id": "agent-01",
    "instance_name": "agent-01.rai",
    "role": "executor",
    "project": "tick",
    "branch": "feature/live-feed",
    "status": "busy",
    "capabilities": ["observe", "stream-json"],
    "port": 4100,
    "addresses": ["127.0.0.1:4100"],
    "metadata": {
      "rai_component": "tick",
      "rai_level": "warn",
      "log": "Queue is backing up",
      "tokens": "24000"
    }
  }
}
```

The dashboard currently derives per-event presentation from `current.metadata` using these conventions:

- `rai_component` → logical source label (`camp`, `garc`, `tick`, `wasp`, etc.)
- `rai_level` → colorized level (`info`, `warn`, `error`, `success`)
- `log` → human-facing event payload
- `tokens` → parsed into numeric token count for the drill-down panels

If these fields are absent, the listener falls back to safe defaults.

Example sample payloads also live under:

- `schemas/camp.json`
- `schemas/wasp.json`

---

## Demo mode

Demo mode is meant for interface development, screenshots, and operator rehearsals.

It simulates:

- four agents with mixed statuses (`busy`, `idle`, `offline`),
- multiple RAI components (`tick`, `wasp`, `garc`, `camp`),
- rotating event levels,
- rolling activity sparkline data,
- realistic metadata such as token counts, model names, file paths, and per-agent cost.

This is useful when:

- you are polishing the UI before `camp` is available,
- you want deterministic visual states for QA,
- you need to validate the layout without creating real swarm traffic.

---

## Repository layout

```text
.
├── crates/
│   ├── view-core/
│   ├── view-cli/
│   └── view-desktop/
├── docs/
└── schemas/
```

- `crates/view-core` holds the shared runtime and state engine
- `crates/view-cli` holds the ratatui mission-control surface
- `crates/view-desktop` is the native desktop shell lane

```text
visual_interception_event_window/
├── AGENTS.md
├── Cargo.toml
├── schemas/
│   ├── camp.json
│   └── wasp.json
└── src/
    ├── app.rs
    ├── listener.rs
    ├── main.rs
    └── ui.rs
```

### Important modules

- `src/main.rs` — terminal bootstrap, raw-mode guard, input polling, frame loop
- `src/app.rs` — application state, filtering, search, selection, summaries, activity windows
- `src/listener.rs` — demo stream + CAMP watcher integration
- `src/ui.rs` — immediate-mode rendering and panel composition

---

## Development and verification

From the repo root:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

For quick manual UI checks:

```bash
VIEW_DEMO=1 cargo run
```

The existing test suite covers:

- agent/event summary calculations,
- filter + search visibility behavior,
- selection and grid paging,
- view-mode toggling,
- listener metadata mapping,
- demo-mode truthy parsing,
- rendering invariants for the live feed and multi-agent grid.

---

## Limitations and non-goals

Current non-goals or not-yet-built pieces include:

- sending control commands back into the swarm,
- persisted event history beyond the in-memory recent buffer,
- direct ingestion of independent `wasp`/`tick` sockets without CAMP mediation,
- configurable theming or layout presets,
- remote/web observability surfaces,
- alert routing or notification fan-out.

This crate currently optimizes for **fast local situational awareness**, not for telemetry warehousing.

---

## Roadmap / likely next improvements

- [ ] Rename/publish the operator-facing binary cleanly as `view`
- [ ] Add dedicated CLI flags for selecting demo/live modes without env vars
- [ ] Support more event sources beyond the current CAMP watcher path
- [ ] Add richer aggregation panels for project-level and branch-level hot spots
- [ ] Introduce persistence/export for recent events and session snapshots
- [ ] Add screenshot/demo automation for release docs and regression review
