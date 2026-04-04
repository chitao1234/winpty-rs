#![cfg(windows)]

mod support;

use support::{cmd_exe, read_until_contains};
use winptyrs::{AgentBuilder, Error, SpawnConfig};

#[test]
fn reads_and_writes_cmd_output() {
    let mut pty = AgentBuilder::new().open().unwrap();
    let child = pty.spawn(SpawnConfig::new(cmd_exe())).unwrap();
    assert!(child.is_alive().unwrap());

    pty.write("echo integration-test\r\n").unwrap();
    let output = read_until_contains(&pty, "integration-test");
    assert!(output.contains("integration-test"));
}

#[test]
fn reports_exit_code_after_wait() {
    let mut pty = AgentBuilder::new().open().unwrap();
    let child = pty
        .spawn(SpawnConfig::new(cmd_exe()).cmdline("/c exit 7"))
        .unwrap();

    let exit = child.wait().unwrap();
    assert_eq!(exit, 7);
    assert_eq!(child.try_wait().unwrap(), Some(7));
}

#[test]
fn read_nonblocking_returns_empty_when_no_output_is_available() {
    let pty = AgentBuilder::new().open().unwrap();

    assert_eq!(pty.read_nonblocking().unwrap(), "");
}

#[test]
fn spawn_reports_missing_executable_as_an_error() {
    let mut pty = AgentBuilder::new().open().unwrap();
    match pty.spawn(SpawnConfig::new(
        "C:\\definitely-missing\\winptyrs-missing.exe",
    )) {
        Ok(_) => panic!("spawning a missing executable should fail"),
        Err(err) => assert!(matches!(err, Error::Winpty(_))),
    }
}
