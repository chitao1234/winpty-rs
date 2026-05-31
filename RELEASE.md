To release a new version of the workspace:

1. Run `git submodule update --init --recursive`
2. Run `cargo test --workspace --all-targets`
3. Run `WINPTY_STATIC=0 cargo check -p winptyrs-sys --features vendored --target x86_64-pc-windows-gnu`
4. Run `WINPTY_STATIC=1 cargo check -p winptyrs-sys --features vendored --target x86_64-pc-windows-gnu`
5. Run `WINPTY_SOURCE_DIR="$PWD/crates/winptyrs-sys/vendor/winpty" cargo check -p winptyrs-sys --features vendored --target x86_64-pc-windows-gnu`
6. Update `CHANGELOG.md`
7. Publish `winptyrs-sys`
8. Wait for crates.io to index `winptyrs-sys`
9. Publish `winptyrs`
