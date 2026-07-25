//! Build script for wayle-idle-inhibit.
//!
//! libwayland-client is linked via `#[link(name = "wayland-client")]` on the
//! FFI extern block in `src/ffi.rs`, so it is always retained regardless of
//! linker `--as-needed` ordering, in every binary and test depending on this
//! crate. This build script therefore forces no link order. Binaries that also
//! use `gtk4-layer-shell` must still ensure it is linked before wayland-client
//! for its interposition to work; handle that in the final binary/shell.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
