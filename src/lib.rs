//! Create and spawn processes inside a pseudoterminal in Windows.
//!
//! This crate provides an abstraction over different backend implementations to spawn PTY processes in Windows.
//! Right now this library supports using [`WinPTY`] and [`ConPTY`].
//!
//! The abstraction is represented through the [`PTY`] struct, which declares methods to initialize, spawn, read,
//! write and get diverse information about the state of a process that is running inside a pseudoterminal.
//!
//! [`WinPTY`]: https://github.com/rprichard/winpty
//! [`ConPTY`]: https://docs.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session

#[cfg(windows)]
#[macro_use]
extern crate enum_primitive_derive;
#[cfg(windows)]
extern crate num_traits;

#[cfg(windows)]
pub mod pty;
#[cfg(windows)]
// mod pty_spawn;
#[cfg(windows)]
pub use pty::{AgentConfig, MouseMode, PTYArgs, PTYBackend, PTY};

#[cfg(not(windows))]
mod non_windows {
    use std::ffi::OsString;

    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum MouseMode {
        WINPTY_MOUSE_MODE_NONE,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct AgentConfig(pub u64);

    impl AgentConfig {
        pub const WINPTY_FLAG_COLOR_ESCAPES: Self = Self(0);
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum PTYBackend {
        ConPTY = 0,
        WinPTY = 1,
        Auto = 2,
        NoBackend = 3,
    }

    #[derive(Clone, Debug)]
    pub struct PTYArgs {
        pub cols: i32,
        pub rows: i32,
        pub mouse_mode: MouseMode,
        pub timeout: u32,
        pub agent_config: AgentConfig,
    }

    impl Default for PTYArgs {
        fn default() -> Self {
            Self {
                cols: 80,
                rows: 24,
                mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
                timeout: 10_000,
                agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES,
            }
        }
    }

    pub struct PTY {
        backend: PTYBackend,
    }

    impl PTY {
        pub fn new(_args: &PTYArgs) -> Result<PTY, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn new_with_backend(_args: &PTYArgs, backend: PTYBackend) -> Result<PTY, OsString> {
            Err(OsString::from(format!(
                "winpty-rs backend {:?} is only available on Windows",
                backend
            )))
        }

        pub fn spawn(
            &mut self,
            _appname: OsString,
            _cmdline: Option<OsString>,
            _cwd: Option<OsString>,
            _env: Option<OsString>,
        ) -> Result<bool, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn set_size(&self, _cols: i32, _rows: i32) -> Result<(), OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn get_backend(&self) -> PTYBackend {
            self.backend
        }

        pub fn read(&self, _blocking: bool) -> Result<OsString, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn write(&self, _buf: OsString) -> Result<u32, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn is_eof(&self) -> Result<bool, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn get_exitstatus(&self) -> Result<Option<u32>, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn is_alive(&self) -> Result<bool, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn get_pid(&self) -> u32 {
            0
        }

        pub fn get_fd(&self) -> isize {
            -1
        }

        pub fn wait_for_exit(&self) -> Result<bool, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }

        pub fn cancel_io(&self) -> Result<bool, OsString> {
            Err(OsString::from("winpty-rs is only available on Windows"))
        }
    }
}

#[cfg(not(windows))]
pub use non_windows::{AgentConfig, MouseMode, PTYArgs, PTYBackend, PTY};

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::thread::sleep;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn test_write_performance() {
        // Initialize PTY with default arguments
        let args = PTYArgs {
            cols: 80,
            rows: 24,
            ..PTYArgs::default()
        };
        let mut pty = PTY::new_with_backend(&args, PTYBackend::ConPTY).unwrap();

        // Spawn cmd.exe
        let cmd = OsString::from("c:\\windows\\system32\\cmd.exe");
        pty.spawn(cmd, None, None, None).unwrap();
        pty.write(OsString::from("\x1b[?1;0c\x1b[0;0R")).unwrap();

        // Wait for process to start
        sleep(Duration::from_millis(1000));

        // Test data
        let test_data = OsString::from("echo test\r\n");
        let iterations = 100;
        let mut total_time = Duration::from_secs(0);

        // Perform write performance test
        for i in 0..iterations {
            let start = Instant::now();
            pty.write(test_data.clone()).unwrap();
            let duration = start.elapsed();
            total_time += duration;
            println!("Write {}: {:?}", i + 1, duration);
        }

        // Calculate and print statistics
        let avg_time = total_time.as_secs_f64() / iterations as f64;
        println!("\nWrite Performance Test Results:");
        println!("Total time: {:?}", total_time);
        println!("Average time per write: {:.2}ms", avg_time * 1000.0);
        println!("Total writes: {}", iterations);
    }

    #[test]
    fn test_read_performance() {
        // Initialize PTY with default arguments
        let args = PTYArgs {
            cols: 80,
            rows: 24,
            ..PTYArgs::default()
        };
        let mut pty = PTY::new_with_backend(&args, PTYBackend::ConPTY).unwrap();

        // Spawn cmd.exe with a command that produces continuous output
        let cmd = OsString::from("c:\\windows\\system32\\cmd.exe");
        pty.spawn(cmd, Some("/c echo test".into()), None, None)
            .unwrap();
        pty.write(OsString::from("\x1b[?1;0c\x1b[0;0R")).unwrap();

        // Wait for process to start
        sleep(Duration::from_millis(1000));

        // Test parameters
        let iterations = 100;
        let mut total_time = Duration::from_secs(0);
        let mut total_bytes = 0;
        let mut successful_reads = 0;

        // Perform read performance test
        for i in 0..iterations {
            let start = Instant::now();
            match pty.read(false) {
                Ok(data) => {
                    let duration = start.elapsed();
                    total_time += duration;
                    total_bytes += data.len();
                    successful_reads += 1;
                    println!("Read {}: {:?}, bytes: {}", i + 1, duration, data.len());
                }
                Err(e) => {
                    println!("Read {} failed: {:?}", i + 1, e);
                }
            }
        }

        // Calculate and print statistics
        let avg_time = if successful_reads > 0 {
            total_time.as_secs_f64() / successful_reads as f64
        } else {
            0.0
        };
        let avg_bytes = if successful_reads > 0 {
            total_bytes as f64 / successful_reads as f64
        } else {
            0.0
        };

        println!("\nRead Performance Test Results:");
        println!("Total time: {:?}", total_time);
        println!("Average time per read: {:.2}ms", avg_time * 1000.0);
        println!("Total bytes read: {}", total_bytes);
        println!("Average bytes per read: {:.2}", avg_bytes);
        println!("Successful reads: {}", successful_reads);
    }

    #[test]
    fn test_nonblocking_read_performance() {
        // Initialize PTY with default arguments
        let args = PTYArgs {
            cols: 80,
            rows: 24,
            ..PTYArgs::default()
        };

        let mut pty = PTY::new_with_backend(&args, PTYBackend::ConPTY).unwrap();
        pty.spawn("cmd.exe".into(), Some("/c echo test".into()), None, None)
            .unwrap();

        pty.write(OsString::from("\x1b[?1;0c\x1b[0;0R")).unwrap();

        // Wait for process to start
        std::thread::sleep(std::time::Duration::from_millis(100));

        let start = Instant::now();
        let mut total_bytes = 0;
        let mut read_count = 0;
        let mut empty_reads = 0;

        // Read 100 times in non-blocking mode
        for _ in 0..100 {
            match pty.read(false) {
                Ok(data) => {
                    if data.is_empty() {
                        empty_reads += 1;
                    } else {
                        total_bytes += data.len();
                        read_count += 1;
                    }
                }
                Err(_) => break,
            }
        }

        let duration = start.elapsed();
        println!("Non-blocking read performance test:");
        println!("Total time: {:?}", duration);
        println!(
            "Average time per read: {:?}ms",
            duration.as_secs_f64() * 1000.0 / (read_count + empty_reads) as f64
        );
        println!("Total bytes read: {}", total_bytes);
        println!("Successful reads: {}", read_count);
        println!("Empty reads: {}", empty_reads);
        println!(
            "Average bytes per successful read: {}",
            if read_count > 0 {
                total_bytes / read_count
            } else {
                0
            }
        );
    }
}
