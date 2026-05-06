# CLI Reference

`_cmd` is not published as a standalone binary yet. Use Cargo package commands
from the workspace root.

## TUI Dashboard

```bash
cargo run -p cli
```

Live mode reads newline-delimited JSON events from stdin.

```bash
some-agent-command | cargo run -p cli
```

## Demo Mode

```bash
VIEW_DEMO=1 cargo run -p cli
```

`VIEW_DEMO` accepts `1`, `true`, `yes`, `on`, or `demo`.

## Desktop App

```bash
VIEW_DEMO=1 cargo run -p desktop
```

The desktop app starts the local web API in the background. Override its port
with `VIEW_WEB_PORT`.

```bash
VIEW_WEB_PORT=23800 VIEW_DEMO=1 cargo run -p desktop
```

## Shell Setup Surface

Generate and install the managed zsh integration block:

```bash
cargo run -p desktop -- setup-shell
```

Print the managed source block without modifying files:

```bash
cargo run -p desktop -- print-shell-init
```

Remove the managed block and delete the generated shell file:

```bash
cargo run -p desktop -- reset-shell
```

## Web API

The web crate is currently a library used by the desktop app. It exposes:

- `GET /`
- `GET /api/agents`
- `GET /api/events`
- `GET /api/snapshot`
- `GET /ws`
