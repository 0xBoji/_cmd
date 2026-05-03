# FAQ

## Is `view` a terminal emulator?

Not primarily. `view` is a passive dashboard for observing local AI coding
agents. The desktop surface has terminal-session UI work, but the product
contract is still observability first.

## Does `view` modify agent state?

No. The core contract is passive observation. `view` consumes structured event
streams and renders state.

## How do I run it locally?

```bash
VIEW_DEMO=1 cargo run -p cli
VIEW_DEMO=1 cargo run -p desktop
```

Use the CLI without `VIEW_DEMO` when piping a live JSON event stream.

## Why does the workspace use `crates/core`, `crates/cli`, `crates/desktop`, and `crates/web`?

That is the smallest Kaku-inspired scaffold that matches the current product:
shared state, TUI renderer, desktop renderer, and web observer API. Larger Kaku
boundaries such as `mux`, `term`, and `window` are deferred until real code
needs them.

## Is there a config file?

Not yet. Current configuration is environment-variable based. See
`docs/configuration.md`.

## Are Kaku features like Lua config, shell hooks, lazygit, yazi, and frecency directory jumping implemented?

No. They are useful reference ideas, not current `view` features.
