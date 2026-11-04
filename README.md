# babydra-core

Multi-crate workspace for BabyDra headless services, the theme engine, shared GTK UI Kit, and Island widget.

## Workspace crates

- `babydra-core`: system services, configuration, and shared data models. It has no GTK, GLib, GDK, or Cairo dependencies.
- `babydra-theme`: theme package parsing and token resolution.
- `babydra-ui-kit`: reusable GTK components and adapters.
- `babydra-island`: dynamic island widget built on the UI Kit.

Default themes are in `crates/babydra-core/assets/themes/`. The SVG icon package is a separate dependency: `babydra-icons`.

```sh
cargo check --workspace
cargo test --workspace
```
