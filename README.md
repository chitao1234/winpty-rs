# winpty-rs Workspace

This repository now contains two crates:

- `winptyrs-sys`: raw FFI bindings and native linking for winpty
- `winptyrs`: a safe Rust wrapper around winpty

## Installation

```toml
[dependencies]
winptyrs = "0.1"
```

To force vendored mode:

```toml
[dependencies]
winptyrs = { version = "0.1", features = ["vendored"] }
```

Vendored mode uses the bundled `crates/winptyrs-sys/vendor/winpty` source tree
by default.

Discovery mode accepts the following environment variables:

- `WINPTY_DIR`
- `WINPTY_LIB_DIR`
- `WINPTY_BIN_DIR`
- `WINPTY_INCLUDE_DIR`
- `WINPTY_STATIC=1` to force static linking
- `WINPTY_STATIC=0` to force dynamic linking

Vendored mode optionally accepts:

- `WINPTY_SOURCE_DIR=/path/to/winpty`

The vendored build copies the selected source tree into `OUT_DIR`, builds there,
and links against the staged artifacts. It does not build in place.

## Workspace Layout

- `crates/winptyrs-sys`
- `crates/winptyrs`

ConPTY support and the old backend-selection API have been removed.
