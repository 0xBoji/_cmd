# VIEW Core Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the current single-crate VIEW project into a Cargo workspace and extract the runtime/state logic into `view-core` while keeping the app runnable.

**Architecture:** Introduce a workspace root plus two crates: `view-core` for shared domain/state/listener logic and `view-cli` for the current ratatui entrypoint. Start with a minimal extraction that preserves behavior by moving `app.rs` and `listener.rs` into `view-core`, then rewire the CLI crate to import them.

**Tech Stack:** Rust 2021, Cargo workspaces, Tokio, ratatui, crossterm, anyhow, serde, chrono

---

### Task 1: Create the Cargo Workspace Skeleton

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/view-core/Cargo.toml`
- Create: `crates/view-cli/Cargo.toml`
- Create: `crates/view-core/src/lib.rs`
- Create: `crates/view-cli/src/main.rs`

- [ ] **Step 1: Write the failing structure test**

Add a workspace test command target by expecting these commands to fail before the workspace exists:

```bash
cargo check -p view-core
cargo check -p view-cli
```

Expected: Cargo reports package(s) not found.

- [ ] **Step 2: Replace the root manifest with a workspace manifest**

Root `Cargo.toml` should become:

```toml
[workspace]
members = [
  "crates/view-core",
  "crates/view-cli",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"
repository = "https://github.com/<org-or-user>/view"
description = "VIEW mission-control workspace"
```

- [ ] **Step 3: Create the `view-core` manifest**

`crates/view-core/Cargo.toml`:

```toml
[package]
name = "view-core"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core runtime and shared state for VIEW"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 4: Create the `view-cli` manifest**

`crates/view-cli/Cargo.toml`:

```toml
[package]
name = "view-cli"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "CLI mission-control surface for VIEW"

[dependencies]
view-core = { path = "../view-core" }
ratatui = "0.26"
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 5: Create temporary entry files**

`crates/view-core/src/lib.rs`:

```rust
pub mod app;
pub mod listener;
```

`crates/view-cli/src/main.rs`:

```rust
fn main() {
    println!("view-cli bootstrap");
}
```

- [ ] **Step 6: Run workspace checks**

Run:

```bash
cargo check -p view-core
cargo check -p view-cli
```

Expected: both packages resolve, though later tasks may still be needed for full compilation.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/view-core/Cargo.toml crates/view-cli/Cargo.toml crates/view-core/src/lib.rs crates/view-cli/src/main.rs
git commit -m "refactor: introduce view workspace skeleton"
```

### Task 2: Move Runtime and State into `view-core`

**Files:**
- Create: `crates/view-core/src/app.rs`
- Create: `crates/view-core/src/listener.rs`
- Modify: `crates/view-core/src/lib.rs`
- Modify: `src/app.rs`
- Modify: `src/listener.rs`

- [ ] **Step 1: Write the failing compile check**

Run:

```bash
cargo check -p view-core
```

Expected: fails until `app.rs` and `listener.rs` exist in `view-core`.

- [ ] **Step 2: Move `src/app.rs` into `crates/view-core/src/app.rs`**

The new file should initially be a near-identical copy of the current root `src/app.rs` so behavior does not change during extraction.

- [ ] **Step 3: Move `src/listener.rs` into `crates/view-core/src/listener.rs`**

Keep imports using `crate::app::{...}` so it still composes inside `view-core`.

- [ ] **Step 4: Ensure `view-core` exports the modules**

`crates/view-core/src/lib.rs` should remain:

```rust
pub mod app;
pub mod listener;
```

- [ ] **Step 5: Turn the root copies into temporary shims or remove them**

Preferred temporary shim approach in the old root files while rewiring:

`src/app.rs`:

```rust
pub use view_core::app::*;
```

`src/listener.rs`:

```rust
pub use view_core::listener::*;
```

- [ ] **Step 6: Run core tests**

Run:

```bash
cargo test -p view-core
```

Expected: the moved tests from `app.rs` and `listener.rs` pass under `view-core`.

- [ ] **Step 7: Commit**

```bash
git add crates/view-core/src/app.rs crates/view-core/src/listener.rs crates/view-core/src/lib.rs src/app.rs src/listener.rs
git commit -m "refactor: extract view-core runtime modules"
```

### Task 3: Rewire the Existing TUI as `view-cli`

**Files:**
- Create: `crates/view-cli/src/ui.rs`
- Modify: `crates/view-cli/src/main.rs`
- Modify: `src/main.rs`
- Modify: `src/ui.rs`

- [ ] **Step 1: Write the failing CLI check**

Run:

```bash
cargo check -p view-cli
```

Expected: fails until `ui.rs` and imports are wired to `view-core`.

- [ ] **Step 2: Move the current TUI rendering file**

Copy the current root `src/ui.rs` into `crates/view-cli/src/ui.rs`.

- [ ] **Step 3: Update imports in `crates/view-cli/src/ui.rs`**

Replace:

```rust
use crate::app::{Agent, AppState, Event, ViewMode};
```

With:

```rust
use view_core::app::{Agent, AppState, Event, ViewMode};
```

- [ ] **Step 4: Replace `crates/view-cli/src/main.rs` with the real entrypoint**

The top of the file should look like:

```rust
mod ui;

use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};
use anyhow::Result;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::{mpsc, Mutex};
use view_core::{
    app::{Agent, AppState, Event},
    listener,
};
```

Then move the current `src/main.rs` body into this file with those imports.

- [ ] **Step 5: Turn the root `src/main.rs` into a compatibility shim**

Temporary root shim:

```rust
fn main() {
    eprintln!("Use `cargo run -p view-cli` during the workspace transition.");
}
```

This avoids breaking root invocation while the workspace settles.

- [ ] **Step 6: Run end-to-end checks**

Run:

```bash
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Expected: full workspace compiles, lints cleanly, and tests pass.

- [ ] **Step 7: Smoke-test the CLI**

Run:

```bash
VIEW_DEMO=1 cargo run -p view-cli
```

Expected: the current mission-control UI opens and behaves exactly as before, now backed by `view-core`.

- [ ] **Step 8: Commit**

```bash
git add crates/view-cli/src/main.rs crates/view-cli/src/ui.rs src/main.rs src/ui.rs
git commit -m "refactor: rewire tui as view-cli"
```

### Task 4: Document the New Layout

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add workspace structure documentation**

Add a short section:

```md
## Workspace Layout

- `view-core`: session runtime, mesh listener, state model
- `view-cli`: ratatui mission-control surface
- `view-desktop`: future desktop shell
```

- [ ] **Step 2: Add current run commands**

```md
## Running

- Demo mode: `VIEW_DEMO=1 cargo run -p view-cli`
- Future production mode: `cargo run -p view-cli`
```

- [ ] **Step 3: Run a final markdown sanity pass**

Run:

```bash
sed -n '1,220p' README.md
```

Expected: the new workspace docs read cleanly and match the actual commands.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: describe workspace transition"
```

## Self-Review

- Spec coverage: this plan covers the approved first execution slice only, not the full desktop app. That is intentional and matches the recommended "smallest real architectural step".
- Placeholder scan: all tasks reference exact files and commands. No `TODO` or `TBD` placeholders remain.
- Type consistency: module names are fixed as `view_core::app`, `view_core::listener`, and `view-cli` keeps `ui.rs` local.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-08-view-core-extraction.md`.

Because you already explicitly asked me to proceed, I’m taking the inline-execution path and starting Task 1 now.
