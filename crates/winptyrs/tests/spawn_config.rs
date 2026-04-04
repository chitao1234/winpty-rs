#![cfg(windows)]

mod support;

use std::path::Path;

use support::{normalize_windows_path, read_probe_report, spawn_probe_exe};
use winptyrs::{AgentBuilder, EnvBlock, SpawnConfig};

#[test]
fn spawn_config_sets_child_working_directory() {
    let mut pty = AgentBuilder::new().open().unwrap();
    let cwd = "C:\\Windows";
    let _child = pty
        .spawn(SpawnConfig::new(spawn_probe_exe()).cwd(cwd))
        .unwrap();

    let report = read_probe_report(&pty);
    assert_eq!(
        normalize_windows_path(&report.cwd),
        normalize_windows_path(Path::new(cwd))
    );
}

#[test]
fn spawn_config_sets_child_environment() {
    let mut pty = AgentBuilder::new().open().unwrap();
    let env = EnvBlock::from_pairs([
        ("WINPTYRS_TEST_ALPHA", "one"),
        ("WINPTYRS_TEST_BRAVO", "two words"),
    ]);
    let _child = pty
        .spawn(SpawnConfig::new(spawn_probe_exe()).env(env))
        .unwrap();

    let report = read_probe_report(&pty);
    assert_eq!(
        report.env.get("WINPTYRS_TEST_ALPHA").map(String::as_str),
        Some("one")
    );
    assert_eq!(
        report.env.get("WINPTYRS_TEST_BRAVO").map(String::as_str),
        Some("two words")
    );
}
