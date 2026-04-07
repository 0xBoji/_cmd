# AGENTS.md — visual_interception_event_window

This file governs the entire `visual_interception_event_window/` repository.

## Mission
Build `view` (Visual Interception Event Window), the passive observability layer (Pillar 8) of the RAI ecosystem. `view` provides a real-time, high-performance TUI (Terminal User Interface) dashboard to monitor mesh presence (`camp`), execution sandboxes (`wasp`), and task orchestration (`tick`) without intercepting or interfering with the processes.

## Product Contract
- **Binary name**: `view`
- **Primary command**: `view` (starts the dashboard)
- **Role**: Passive Observer. It consumes JSON event streams and renders them into a multi-panel layout.
- **Core Restrictions**:
    - **Non-blocking**: The TUI must never block the event processing or input handling.
    - **Resource Efficient**: Must target ~60fps without high CPU usage; uses `tokio::time::interval`.
    - **Zero Flicker**: Correct use of `ratatui` double-buffer diffing; avoid manual `clear()` calls.
    - **Stateless/Passive**: It does not modify the state of other agents; it only visualizes observed state.

## Required Technical Choices
- `ratatui` for layout and widgets.
- `crossterm` for terminal backend and raw mode handling.
- `tokio` for async runtime and MPSC channels.
- `serde` and `serde_json` for event parsing.
- `anyhow` for application-level error handling.
- `chrono` for precise event timestamping.

## Expected Module Layout
Keep the crate modular and functional:
- `src/main.rs` — Entry point, terminal initialization, and the main 60fps loop.
- `src/app.rs` — Thread-safe application state (`AppState`) and domain models (`Agent`, `Event`).
- `src/ui.rs` — Immediate-mode rendering logic and component definitions.
- `src/listener.rs` — Async background tasks for event consumption (simulated or real).

## Output Contract for AI Agents
- **Visual Excellence**: The TUI should be extremely premium, using modern colors, bold highlights, and clear layouts.
- **JSON Compatibility**: While primarily a visual tool, internal event schemas must remain compatible with `camp watch --json` and `wasp --json` outputs.

## Code Quality Rules
- **Panic-free**: No `unwrap()`, `expect()`, or `panic!` macros in production execution paths.
- **Terminal Hygiene**: Use RAII guards (`Drop` implementation) to ensure the terminal is restored (raw mode off, alternate screen left) even on panics or unexpected exits.
- **Concurrency**: UI thread must remain responsive; use `tokio::sync::Mutex` for state protection across thread boundaries.

## Commit and Agent-Knowledge Rules
- Treat git history as part of the agent memory for this repo.
- Every meaningful change should be committed with a Conventional Commit style subject: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`.
- For non-trivial commits, include lore-style trailers:
    - `Constraint: ...`
    - `Rejected: ...`
    - `Confidence: low|medium|high`
    - `Scope-risk: narrow|moderate|broad`
    - `Directive: ...`
    - `Tested: ...`
    - `Not-tested: ...`
- Do not combine unrelated work into one commit; preserve a searchable knowledge trail.
