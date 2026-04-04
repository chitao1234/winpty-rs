To release a new version of the workspace:

1. Run `cargo test --workspace --all-targets`
2. Run `WINPTY_DIR="$HOME/ddev/winpty/build" cargo test -p winptyrs --test smoke`
3. Run `cargo check -p winptyrs-sys --target x86_64-pc-windows-gnu`
4. Run `WINPTY_SOURCE_DIR="$HOME/ddev/winpty" cargo check -p winptyrs-sys --target x86_64-pc-windows-gnu`
5. Update `CHANGELOG.md`
6. Publish `winptyrs-sys`
7. Wait for crates.io to index `winptyrs-sys`
8. Publish `winptyrs`
