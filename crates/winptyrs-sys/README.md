# winptyrs-sys

Raw FFI bindings and native linking for winpty.

Discovery mode:

- `WINPTY_DIR`
- `WINPTY_LIB_DIR`
- `WINPTY_BIN_DIR`
- `WINPTY_INCLUDE_DIR`
- `WINPTY_STATIC=1` to force static linking
- `WINPTY_STATIC=0` to force dynamic linking

Vendored mode:

- enable the Cargo feature `vendored`
- vendored builds use the bundled `vendor/winpty` source tree by default
- set `WINPTY_SOURCE_DIR` to a winpty source tree such as `~/ddev/winpty` to
  override the bundled source
- the build script copies the selected source tree into `OUT_DIR`, runs `bash
  configure`, then `make`, then `make install-bin install-lib install-include`

Runtime notes:

- the build script validates that the selected layout includes `winpty-agent.exe`
  and, for dynamic layouts, `winpty.dll`
- `winpty-agent.exe` must live next to the module containing `libwinpty` code
- this workspace uses a repo-local Cargo config to stage runtime files into its
  own `target/` outputs for tests and examples
- downstream applications should stage or install the runtime files explicitly
  as part of their own packaging process

This crate exposes the raw ABI only. Use `winptyrs` for a safe interface.
