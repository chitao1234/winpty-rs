#[allow(dead_code)]
pub fn cmd_exe() -> String {
    std::env::var("COMSPEC").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".to_owned())
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn read_until_contains(pty: &winptyrs::Pty, needle: &str) -> String {
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = String::new();

    while Instant::now() < deadline {
        let chunk = pty.read_nonblocking().unwrap();
        if !chunk.is_empty() {
            output.push_str(&chunk);
            if output.contains(needle) {
                return output;
            }
        }
        sleep(Duration::from_millis(50));
    }

    panic!("timed out waiting for output containing {needle:?}; collected output: {output:?}");
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn resize_probe_exe() -> String {
    let built = std::path::Path::new(env!("CARGO_BIN_EXE_resize_probe"));
    let file_name = built
        .file_name()
        .expect("resize_probe executable should have a file name");

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(debug_dir) = current_exe.parent().and_then(|deps| deps.parent()) {
            let candidate = debug_dir.join(file_name);
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }

    built.to_string_lossy().into_owned()
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn read_until_size_report(pty: &winptyrs::Pty) -> (String, (u16, u16)) {
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = String::new();

    while Instant::now() < deadline {
        let chunk = pty.read_nonblocking().unwrap();
        if !chunk.is_empty() {
            output.push_str(&chunk);
            if let Some(size) = parse_last_size_report(&output) {
                return (output, size);
            }
        }
        sleep(Duration::from_millis(50));
    }

    panic!("timed out waiting for SIZE report; collected output: {output:?}");
}

#[cfg(windows)]
fn parse_last_size_report(output: &str) -> Option<(u16, u16)> {
    output.rmatch_indices("SIZE ").find_map(|(offset, _)| {
        let tail = &output[offset + "SIZE ".len()..];
        if !tail.contains('\n') {
            return None;
        }

        let mut numbers = tail
            .split(|ch: char| !ch.is_ascii_digit())
            .filter(|part| !part.is_empty());
        Some((numbers.next()?.parse().ok()?, numbers.next()?.parse().ok()?))
    })
}
