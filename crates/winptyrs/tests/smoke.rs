#![cfg(windows)]

mod support;

use support::cmd_exe;
use winptyrs::{AgentBuilder, SpawnConfig};

#[test]
fn opens_agent_and_spawns_cmd() {
    let mut pty = AgentBuilder::new().open().unwrap();
    let child = pty.spawn(SpawnConfig::new(cmd_exe())).unwrap();

    assert!(child.id() > 0);
    assert!(child.is_alive().unwrap());
}
