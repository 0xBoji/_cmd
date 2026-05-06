# Feature Architecture & Roadmap

> Context note: this file mixes current direction with future product ideas. It
> is intended as strategy context and inspiration, not as a strict statement of
> already-shipped features.

This document outlines the features of the `_CMD` terminal dashboard,
categorizing them into current foundations and future roadmap items, heavily
inspired by Kaku and the long-term product direction.

## 1. Current Foundations

The foundational architecture of `_CMD` is already in place around a decoupled
shared core and multiple surfaces:

- shared `core` state
- TUI in `cli`
- desktop app in `desktop`
- web API skeleton in `web`

## 2. Strategic Roadmap

### Phase 1: Remote Web Workspace

`_CMD` may evolve into a richer LAN-accessible workspace.

- local network remote access
- integrated file exploration
- richer browser UI

### Phase 2: Advanced Terminal and Layout Mechanics

Ideas inspired by Kaku:

- BSP tree layout engine
- deferred PTY resize while dragging
- multiple terminal panes and split grids

### Phase 3: Shell Introspection and Premium UX

Potential future directions:

- frecency-based directory jumping
- OSC 1337 shell integration
- logical line text selection
- visual bell notifications
- theme-aware typography tuning

## 3. Important Framing

These roadmap items are intentionally documented early so they can guide future
implementation, even where the current repo only contains the first slice of the
architecture.
