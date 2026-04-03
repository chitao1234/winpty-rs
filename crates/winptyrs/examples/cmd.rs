#[cfg(windows)]
fn main() -> winptyrs::Result<()> {
    use winptyrs::{AgentBuilder, SpawnConfig};

    let mut pty = AgentBuilder::new().open()?;
    let _child = pty.spawn(SpawnConfig::new("C:\\Windows\\System32\\cmd.exe"))?;
    pty.write("echo hello from winptyrs\r\n")?;
    println!("{}", pty.read_blocking()?);
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("winptyrs example is only supported on Windows targets");
}
