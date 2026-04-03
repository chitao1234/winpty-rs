# winptyrs

Rust-idiomatic wrapper around winpty for spawning and controlling Windows console
programs inside a pseudo-terminal.

## Runtime Files

`winptyrs` depends on `winpty-agent.exe` at runtime.

- With dynamic linking, ship `winpty-agent.exe` next to `winpty.dll`.
- With static linking, ship `winpty-agent.exe` next to the final executable.

This repository auto-stages `winpty-agent.exe` and `winpty.dll` into its own
`target/` outputs for Windows tests and examples. Downstream consumers are
expected to stage these files deliberately as part of their own packaging or
install process.
