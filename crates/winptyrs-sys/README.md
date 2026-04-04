# winptyrs-sys

Raw FFI bindings and native linking for winpty.

Vendored mode is the default. The build script uses the bundled
`vendor/winpty` tree unless you set a non-empty discovery override such as
`WINPTY_DIR`.

Discovery mode:

- `WINPTY_DIR`
- `WINPTY_LIB_DIR`
- `WINPTY_BIN_DIR`
- `WINPTY_INCLUDE_DIR`
- `WINPTY_STATIC=1` to force static linking
- `WINPTY_STATIC=0` to force dynamic linking

Vendored mode:

- enabled by default
- keep the Cargo feature `vendored` enabled when you want vendored behavior
- vendored builds use the bundled `vendor/winpty` git submodule by default
- set `WINPTY_SOURCE_DIR` to a winpty source tree such as `~/ddev/winpty` to
  override the bundled source
- the build script compiles the selected source tree directly with the Rust
  `cc` crate
- vendored builds stage `winpty.dll`, the matching import library, the static
  `libwinpty` archive, `winpty-agent.exe`, and the public headers into `OUT_DIR`
- vendored builds do not require `sh`, `bash`, `make`, `gmake`, or the
  upstream `configure` script

Runtime notes:

- the build script validates that the selected layout includes `winpty-agent.exe`
  and, for dynamic layouts, `winpty.dll`
- `winpty-agent.exe` must live next to the module containing `libwinpty` code
- this workspace uses a repo-local Cargo config to stage runtime files into its
  own `target/` outputs for tests and examples
- downstream applications should stage or install the runtime files explicitly
  as part of their own packaging process

This crate exposes the raw ABI only. Use `winptyrs` for a safe interface.
