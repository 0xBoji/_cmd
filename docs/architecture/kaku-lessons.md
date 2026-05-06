# Kaku Lessons for `_cmd`

This note captures implementation ideas from the local `Kaku/` reference tree.
They are not requirements for `_cmd`; each item must still pass the passive
observer contract before it becomes product work.

## Useful Ideas

- **Immediate shell metadata beats output scraping.** Kaku uses shell hooks and
  OSC sequences to send structured state such as cwd, command text, and exit
  status. For `_cmd`, that pattern is useful only if a future terminal session
  needs richer self-reporting; agent event streams remain the primary source of
  truth.
- **Keep render loops free of blocking work.** Kaku defers expensive work such
  as font discovery and network/domain setup. `_cmd` should keep the same shape:
  background Tokio work mutates shared state, while TUI and desktop surfaces
  read snapshots and draw.
- **Decouple visual resize from process resize.** Kaku suppresses repeated
  resize signals during live pane drags and flushes the final size once. If
  `_cmd` grows split terminal panes, this is the model to copy.
- **Use layout trees for pane splits.** BSP trees scale better than ad hoc
  grids when panes can split recursively. This is a future fit for desktop/web
  terminal panes, not the current dashboard cards.
- **Treat bundled assets as part of the binary.** Critical icons, static web
  files, and fallback fonts should be embedded when they become required so the
  app remains easy to run from one executable.
- **Optimize release artifacts deliberately.** Kaku's size-oriented release
  profile is worth mirroring once `_cmd` has release packaging. The current
  workspace already includes a `release-opt` profile.

## Guardrails

- Do not copy Kaku features that make `_cmd` an active terminal controller until
  the product contract changes.
- Do not add crates such as Lua, BSP layout, or shell integration just because
  Kaku has them.
- Prefer small crate boundaries that match active code. Add `mux`, `term`, or
  `layout` crates only when implementation pressure makes the boundary real.
