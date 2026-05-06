<div align="center">
  <img src="https://github.com/user-attachments/assets/9976b2da-2edd-4604-a36c-8fd53719c6d4" width="800" alt="_cmd logo" />
  <h1>_cmd</h1>
  <p><em>The operator's glass cockpit for local AI coding agent swarms.</em></p>
</div>

<p align="center">
  <a href="https://github.com/0xBoji/_cmd/stargazers"><img src="https://img.shields.io/github/stars/0xBoji/_cmd?style=flat-square" alt="Stars"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/rust-stable-orange.svg?style=flat-square" alt="Rust"></a>
  <a href="https://github.com/0xBoji/_cmd/actions"><img src="https://img.shields.io/github/check-runs/0xBoji/_cmd/main?style=flat-square" alt="Build Status"></a>
</p>

---

> `_cmd` is a **Command Center for AI Agents** — a passive, real-time observability surface for monitoring local AI coding swarms. It subscribes to live JSON event streams and transforms them into a premium, multi-panel TUI and desktop dashboard—allowing you to inspect what your agents are doing, where they are stuck, and how they are spending tokens, without ever interfering with their execution.

## 🔭 Why _cmd?

Once you start running multiple agents, the bottleneck shifts from **execution** to **visibility**. 

*   **Scattered Logs**: Tailing five different terminal panes to find one error is a nightmare.
*   **Guesswork**: You know agents are running, but are they healthy or just "stalled"?
*   **Information Overload**: Raw JSON is great for machines, but terrible for human split-second decisions.

`_cmd` solves this by providing a dedicated, high-performance cockpit that is **passive** (never mutates state), **real-time** (60fps rendering), and **high-density** (sparklines, cards, and feeds in one _cmd).

## ✨ Features

- **🚀 Dual-Surface Engine**: Choose between a blazing-fast **Ratatui TUI** for the terminal or a premium **egui Desktop UI** for native macOS feel.
- **⚡ Passive Observation**: Zero interference. Subscribes to stdin or WebSocket streams without blocking the observed processes.
- **📊 Real-time Analytics**: Tracks agent activity over a rolling 50-sample window with live sparklines.
- **🛡️ Error Awareness**: Highlights status, error levels, and warnings with level-aware color systems (`info`, `warn`, `error`, `success`).
- **🧩 Multi-Agent Orchestration**: Seamlessly toggle between **Grid** (high-density wall) and **Focus** (deep-dive drill-down) modes.
- **🔍 Global Search & Filter**: Instantly filter by Agent ID, Project, Role, Branch, or Status across the entire swarm.
- **💾 LAN-Ready**: Built-in Axum web server and WebSocket fan-out for remote monitoring across the local network.

## 🏁 Quick Start

### 1. The Demo (Rehearsal Mode)
The fastest way to see `_cmd` in action without any live agents:

```bash
# Start the TUI Dashboard
VIEW_DEMO=1 cargo run -p cli

# Start the Desktop Dashboard
VIEW_DEMO=1 cargo run -p desktop
```

### 2. Live Ingestion
Pipe any newline-delimited JSON stream (NDJSON) directly into the dashboard:

```bash
# Pipe your agent logs
tail -f agent_events.json | cargo run -p cli
```

## ⌨️ Global Shortcuts

| Action | Shortcut |
| :--- | :--- |
| **Quit** | `q` / `Ctrl + C` |
| **Toggle Mode (Grid/Focus)** | `Tab` |
| **Navigate Selection** | `j` / `k` or `Arrows` |
| **Jump Page** | `PgUp` / `PgDn` |
| **Filter Status** | `f` (Cycle All/Busy/Active/Offline) |
| **Search Swarm** | `/` (type to filter) |
| **Clear Search/Focus** | `Esc` |

## 🏗️ Repository Architecture

`_cmd` is built with a strictly decoupled architecture, ensuring the core engine remains UI-agnostic.

```bash
crates/
├── core/      — Domain state, engine, listener, and event schemas (No UI deps)
├── cli/       — TUI surface via Ratatui + Crossterm
├── desktop/   — Native Desktop shell via egui + eframe
└── web/       — Web API + WebSocket server via Axum
```

## 🛠️ Technical Stack

- **Rendering**: [Ratatui](https://github.com/ratatui/ratatui) (TUI) & [egui](https://github.com/emilk/egui) (Desktop)
- **Async Runtime**: [Tokio](https://github.com/tokio-rs/tokio)
- **Networking**: [Axum](https://github.com/tokio-rs/axum) (Web/WS)
- **Serialization**: [Serde JSON](https://github.com/serde-rs/json)
- **Terminal Hygiene**: RAII-guarded raw mode handling via `crossterm`.

## 📡 Event Contract

`_cmd` consumes two main types of JSON payloads. If you want your agent to be "visible", just emit these to stdout or a socket:

<details>
<summary><b>Snapshot Payload</b> (Seeds the roster)</summary>

```json
{
  "kind": "snapshot",
  "agents": [
    {
      "id": "agent-01",
      "instance_name": "executor-alpha",
      "role": "executor",
      "project": "_cmd-dashboard",
      "branch": "main",
      "status": "busy",
      "metadata": { "tokens": "24500", "rai_level": "info" }
    }
  ]
}
```
</details>

<details>
<summary><b>Lifecycle Payload</b> (Live updates)</summary>

```json
{
  "kind": "updated",
  "current": {
    "id": "agent-01",
    "status": "idle",
    "metadata": { "rai_level": "success", "log": "Task completed successfully" }
  }
}
```
</details>

## 🗺️ Roadmap

- [x] High-performance Ratatui TUI
- [x] egui Desktop Shell with macOS polish
- [x] Demo mode for UI iteration
- [ ] Pluggable event adapters (Socket, Tail, HTTP)
- [ ] Persistent event history export
- [ ] Project-level aggregate analytics
- [ ] Global CLI publish as `_cmd`

## 📄 License

MIT License. Built for the operator, the builder, and the dreamer.

---

<p align="center">
  <em>Part of a trilogy of tools for the agentic era. See also <a href="#">Kaku</a> and <a href="#">Warp</a>.</em>
</p>
