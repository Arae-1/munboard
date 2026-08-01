# build.rs — Build Script

## Purpose

This build script tells Cargo when to recompile the server binary. It does not compile any code itself.

## Function: `main()`

The build script's `main` function prints two `cargo::rerun-if-changed` directives:

1. `cargo::rerun-if-changed=migrations/` — The Diesel embedded migrations are compiled into the binary. The binary must be rebuilt if migrations change.
2. `cargo::rerun-if-changed=config.toml.dist` — The default configuration is compiled into the binary via `include_str!`. The binary must be rebuilt if the default config changes.
