# winpty-rs Workspace

This repository now contains two crates:

- `winptyrs-sys`: raw FFI bindings and native linking for winpty
- `winptyrs`: a safe Rust wrapper around winpty

## Installation

```toml
[dependencies]
winptyrs = "0.1"
```

Vendored mode is the default. The crate uses the bundled
`crates/winptyrs-sys/vendor/winpty` git submodule unless you point discovery at
an installed winpty layout with a non-empty override such as `WINPTY_DIR`.

If you disable default features upstream, you can re-enable vendored mode
explicitly:

```toml
[dependencies]
winptyrs = { version = "0.1", features = ["vendored"] }
```

Discovery mode accepts the following environment variables. Any non-empty value
for one of these overrides the vendored default:

- `WINPTY_DIR`
- `WINPTY_LIB_DIR`
- `WINPTY_BIN_DIR`
- `WINPTY_INCLUDE_DIR`
- static linking is the default when both layouts are available
- `WINPTY_STATIC=1` to force static linking
- `WINPTY_STATIC=0` to force dynamic linking

Vendored mode optionally accepts:

- `WINPTY_SOURCE_DIR=/path/to/winpty`

The vendored build compiles the selected winpty source tree directly from Rust build
scripts using the `cc` crate, stages the resulting artifacts into `OUT_DIR`,
and does not build in place. Vendored staging includes:

- `winpty.dll`
- the matching import library
- the static `libwinpty` archive
- `winpty-agent.exe`
- the public winpty headers

## Runtime Files

`winpty` needs both the library and the agent executable at runtime.

- `winpty-agent.exe` must live next to the module containing `libwinpty` code.
- With dynamic linking, place `winpty-agent.exe` next to `winpty.dll`.
- With static linking, place `winpty-agent.exe` next to the final executable.

This repository enables workspace-local runtime staging into `target/` for its
own Windows tests and examples via `.cargo/config.toml`.
Published crates do not force that behavior on downstream consumers. If you
ship an application that uses `winptyrs`, stage these runtime files as part of
your packaging or install process.

## Workspace Layout

- `crates/winptyrs-sys`
- `crates/winptyrs`

ConPTY support and the old backend-selection API have been removed.
