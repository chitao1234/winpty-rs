#[cfg(windows)]
use std::io::{self, BufRead, Write};

#[cfg(windows)]
use windows::Win32::System::Console::{
    GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO, STD_OUTPUT_HANDLE,
};

#[cfg(windows)]
fn report_size() -> io::Result<()> {
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }.map_err(io::Error::other)?;
    let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
    unsafe { GetConsoleScreenBufferInfo(handle, &mut info) }.map_err(io::Error::other)?;

    let cols = (info.srWindow.Right - info.srWindow.Left + 1) as u16;
    let rows = (info.srWindow.Bottom - info.srWindow.Top + 1) as u16;

    println!("SIZE {cols} {rows}");
    io::stdout().flush()?;
    Ok(())
}

#[cfg(windows)]
fn main() -> io::Result<()> {
    report_size()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;

    report_size()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("resize_probe is only supported on Windows targets");
}
