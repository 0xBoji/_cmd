# Static Assets Architecture: Zero-Dependency Distribution

> Context note: this file describes a preferred future asset strategy for
> `_cmd`. It is not meant to imply that all of these asset pipelines already
> exist today.

To ensure `_cmd` remains a fast, standalone binary that can be deployed without
complex installers, our static asset architecture is designed around
"Zero-Dependency Distribution".

## 1. Embedded Assets

We should not rely on the user having specific icons, web files, or fonts
installed on their system when those assets become required.

- Web dashboard assets can be embedded into the binary.
- Desktop icons and PNG/SVG resources can be embedded as bytes.
- This keeps distribution simple and avoids missing-file failures.

## 2. Font Management and Typography

High-quality typography matters for a premium dashboard.

- Embedded fallback fonts can guarantee a sane default.
- Lazy glyph loading can keep memory use under control.

## 3. Dynamic Build Scripts

If the web dashboard later uses a dedicated frontend stack, a `build.rs` script
could bridge frontend builds and Rust compilation.

Takeaway: the Rust artifact and any bundled frontend assets should stay in sync.

## 4. Caching and Local State

While static assets belong in the binary, dynamic state should be stored in a
standardized local directory.

Takeaway: separate immutable bundled assets from mutable runtime state.
