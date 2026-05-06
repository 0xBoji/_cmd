# Assets Architecture

`_cmd` should stay easy to run from source and easy to distribute as a compact
binary. Asset handling should therefore be added only when a surface has a real
asset dependency.

## Current State

There is no required runtime asset directory today. The CLI, desktop, and web
surfaces build from Rust source only.

## When Assets Are Needed

- Embed critical static web files with `include_str!`, `include_bytes!`, or a
  small embedding crate once the web surface has real HTML/CSS/JS assets.
- Embed desktop icons and fallback fonts only when the desktop UI actually uses
  them.
- Keep generated build output out of source control unless release packaging
  explicitly needs it.

## Future `assets/` Layout

```text
assets/
├── icons/
├── fonts/
└── web/
```

Create this directory when at least one checked-in asset exists.
