#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::io::Write;

    println!("BEGIN");
    println!("CWD={}", std::env::current_dir()?.display());
    for key in ["WINPTYRS_TEST_ALPHA", "WINPTYRS_TEST_BRAVO"] {
        println!("ENV:{key}={}", std::env::var(key).unwrap_or_default());
    }
    println!("END");
    std::io::stdout().flush()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("spawn_probe is only supported on Windows targets");
}
