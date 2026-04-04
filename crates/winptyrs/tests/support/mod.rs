#[allow(dead_code)]
pub fn cmd_exe() -> String {
    std::env::var("COMSPEC").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".to_owned())
}

#[cfg(windows)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct ProbeReport {
    pub cwd: String,
    pub env: std::collections::BTreeMap<String, String>,
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
    helper_exe("resize_probe")
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn spawn_probe_exe() -> String {
    helper_exe("spawn_probe")
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
#[allow(dead_code)]
pub fn read_probe_report(pty: &winptyrs::Pty) -> ProbeReport {
    let output = read_until_contains(pty, "END");
    parse_probe_report(&output)
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn normalize_windows_path(path: impl AsRef<std::path::Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

#[cfg(windows)]
fn helper_exe(name: &str) -> String {
    let built = std::path::Path::new(match name {
        "resize_probe" => env!("CARGO_BIN_EXE_resize_probe"),
        "spawn_probe" => env!("CARGO_BIN_EXE_spawn_probe"),
        _ => panic!("unknown helper binary"),
    });
    let file_name = built
        .file_name()
        .expect("helper executable should have a file name");

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

#[cfg(windows)]
fn parse_probe_report(output: &str) -> ProbeReport {
    let mut cwd = None;
    let mut env = std::collections::BTreeMap::new();

    for line in output.lines() {
        let line = strip_ansi(line);
        let line = line.trim_end_matches('\r');

        if let Some(value) = line.strip_prefix("CWD=") {
            cwd = Some(value.to_owned());
        } else if let Some(rest) = line.strip_prefix("ENV:") {
            let (key, value) = rest
                .split_once('=')
                .expect("probe env line should contain '='");
            env.insert(key.to_owned(), value.to_owned());
        }
    }

    ProbeReport {
        cwd: cwd.expect("probe output should contain CWD"),
        env,
    }
}

#[cfg(windows)]
fn strip_ansi(line: &str) -> String {
    let mut out = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }

        out.push(ch);
    }

    out
}
