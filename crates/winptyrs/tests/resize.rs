#![cfg(windows)]

mod support;

use support::{read_until_size_report, resize_probe_exe};
use winptyrs::{AgentBuilder, PtySize, SpawnConfig};

#[test]
fn resize_changes_reported_window_size() {
    let mut pty = AgentBuilder::new().open().unwrap();
    let _child = pty.spawn(SpawnConfig::new(resize_probe_exe())).unwrap();

    let (_initial_output, initial_size) = read_until_size_report(&pty);
    assert_eq!(initial_size, (80, 24));

    pty.resize(PtySize::new(90, 30).unwrap()).unwrap();
    pty.write("\r\n").unwrap();
    let (_resized_output, resized_size) = read_until_size_report(&pty);
    assert_eq!(resized_size, (90, 30));
}

#[test]
fn builder_initial_size_is_applied_when_opening_the_agent() {
    let mut pty = AgentBuilder::new()
        .size(PtySize::new(100, 40).unwrap())
        .open()
        .unwrap();
    let _child = pty.spawn(SpawnConfig::new(resize_probe_exe())).unwrap();

    let (_output, size) = read_until_size_report(&pty);
    assert_eq!(size, (100, 40));
}
