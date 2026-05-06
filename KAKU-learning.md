# Kaku Terminal - Architecture & Features Learning

> Context note: this file is a learning/reference note for future `_cmd`
> directions. It intentionally captures ideas from `Kaku` that are not fully
> implemented in this repo yet.

This document records architectural insights and clever implementations found in
the `Kaku` terminal project. These insights serve as inspiration for building a
fast, modern, and highly optimized terminal UI dashboard.

## 1. Ultra-fast Command History Retention

Kaku achieves its "infinite" and immediate command history memory not by
rewriting a shell from scratch, but by forcefully optimizing the underlying
`zsh` configurations via its integration script
(`assets/shell-integration/setup_zsh.sh`).

It injects the following aggressive history configurations:

```bash
setopt SHARE_HISTORY
setopt APPEND_HISTORY
setopt INC_APPEND_HISTORY
setopt EXTENDED_HISTORY
```

Takeaway: by using `INC_APPEND_HISTORY` and `SHARE_HISTORY`, Kaku ensures that
as soon as the user presses Enter in one pane, the command is flushed to
`~/.zsh_history` and immediately available in all other panes.

## 2. Lightning Fast `cd` Auto-suggestions

When a user types `cd <repo-name>` and presses Tab, Kaku suggests the exact
absolute path almost instantly, even across restarts.

This is achieved by silently bundling and integrating `zsh-z`.

- Tracking: every time the user changes directories, `zsh-z` records the path
  and its frequency/recency into a hidden database file.
- Overriding completion: Kaku overrides the default Tab completion behavior for
  the `cd` command.
- The trick: when Tab is pressed after `cd`, Kaku queries the frecency database
  for the most frequent/recent matching paths and injects the best match into
  the prompt.

Takeaway: to implement a highly intelligent command input or `cd` suggestion
system, do not just rely on `.zsh_history`. Hook into frecency-based directory
databases.

## 3. Terminal-to-Shell Communication (OSC 1337)

Kaku provides features like converting natural language `# <query>` into
commands or catching failed exit codes using shell hooks that emit OSC escape
sequences.

- Capturing exit codes: shell hooks send Base64-encoded status out through
  invisible `OSC 1337 SetUserVar` sequences.
- Input parsing: if the user's input buffer starts with `#`, the shell can
  prevent execution and pass the query to the terminal instead.

Takeaway: deep integration between a terminal UI and the underlying shell does
not require parsing raw PTY output. Shell scripts can emit invisible escape
codes to pass structured state out to the terminal framework.

## 4. Seamless CLI Tool Injection

Kaku features built-in shortcuts for Lazygit and Yazi. Rather than spawning a
separate PTY or creating a complex overlay, it can run these tools directly in
the active shell pane by sending a command string plus Enter to the shell.

Takeaway: simulating keystrokes can be much simpler than hijacking PTY state,
especially for launching tools inside an active pane.

## 5. Background Tab Notification (Visual Bell Dot)

Kaku places an orange dot on inactive tabs when a long-running command finishes
by relying on the terminal `BEL` character.

Takeaway: instead of polling process status, standard terminal control
characters can drive efficient UI notifications.

## 6. Aggressive Binary Size Optimization

Kaku reduces executable size by stripping unused features from dependencies and
using aggressive release profiles.

Takeaway: modern UI apps can stay compact if dependency features and Cargo
profiles are tuned deliberately.

## 7. Deferred Lazy Initialization

To achieve instant startup, Kaku defers heavy operations until after the initial
window is painted.

Takeaway: UI code should never block on disk I/O, network resolution, or font
parsing. Render the shell immediately and push heavier work into background
tasks.

## 8. Pane Layout Management via BSP Trees

Kaku does not manage split panes using a static 2D grid. Instead, it uses a BSP
tree.

Takeaway: a binary tree structure allows split panes to resize cleanly without
overlap or gaps.

## 9. Deferred `SIGWINCH` on Live Drag

When the user drags a split divider, Kaku can update the visual grid while
suppressing PTY resize signals until the drag ends.

Takeaway: decoupling visual resize from PTY logical resize is important for
60fps multi-pane terminal UX.

## 10. Adaptive Typography Weights

Kaku shifts font weights based on the active color scheme to compensate for
optical differences between light and dark themes.

Takeaway: theme-aware font weight tuning can improve perceived readability in a
premium UI.

## 11. Logical vs. Physical Line Mapping

Kaku distinguishes between physical wrapped rows and logical lines when the user
selects terminal text.

Takeaway: wrapped-line metadata is critical if copy/paste should preserve the
original logical text instead of screen row breaks.
