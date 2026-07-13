# Wayle Agent Rules

## Commands
- **Format (CRITICAL)**: MUST use nightly Rust. `cargo +nightly fmt --all`. Do not run stable `cargo fmt`.
- **Lint**: `cargo clippy --workspace --all-targets -- -D warnings`
- **Test**: `cargo test --workspace --no-fail-fast`
- **Icons Check**: `./scripts/ci/check-icons.sh` (run if touching icons)

## Architecture & Boundaries
- Workspace with multiple crates (`crates/wayle-*`) and main app (`wayle/`).
- **Main Bins**:
  - `wayle`: The core desktop shell / CLI entrypoint.
  - `crates/wayle-settings`: GTK GUI for configuration.
- **Styling**: `crates/wayle-styling/scss/main.scss` is compiled to CSS via `build.rs` on cargo build.

## Dependencies & Quirks
- **System Libs required to build**: `gtk4`, `gtk4-layer-shell`, `gtksourceview5`, `libpulse`, `fftw`, `libpipewire`, `systemd-libs`, `clang`.
- GTK/Relm4 heavily used. Ensure `gtk4` and related -dev packages are present in environment before trying to build or run tests.
