#[cfg(windows)]
fn main() -> winptyrs::Result<()> {
    use winptyrs::{AgentBuilder, SpawnConfig};

    let cmd = std::env::var("COMSPEC")
        .or_else(|_| std::env::var("SystemRoot").map(|root| format!("{root}\\System32\\cmd.exe")))
        .expect("COMSPEC or SystemRoot must be set on Windows");
    let mut pty = AgentBuilder::new().open()?;
    let _child = pty.spawn(SpawnConfig::new(cmd))?;
    pty.write("echo hello from winptyrs\r\n")?;
    println!("{}", pty.read_blocking()?);
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("winptyrs example is only supported on Windows targets");
}
