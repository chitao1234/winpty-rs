use std::ffi::OsString;

use winptyrs::{AgentBuilder, AgentFlags, EnvBlock, Error, MouseMode, PtySize};

#[test]
fn rejects_zero_dimensions() {
    let err = PtySize::new(0, 24).unwrap_err();
    assert!(matches!(err, Error::InvalidSize { cols: 0, rows: 24 }));
}

#[test]
fn builder_uses_documented_defaults() {
    let builder = AgentBuilder::new();

    assert_eq!(builder.pty_size(), PtySize::new(80, 24).unwrap());
    assert_eq!(builder.selected_mouse_mode(), MouseMode::None);
    assert_eq!(builder.configured_timeout_ms(), 10_000);
    assert_eq!(builder.configured_agent_flags(), AgentFlags::COLOR_ESCAPES);
}

#[test]
fn env_block_is_double_nul_terminated() {
    let env = EnvBlock::from_pairs([("A", "1"), ("B", "2")]);

    assert_eq!(env.as_wide().last().copied(), Some(0));
    assert_eq!(env.as_wide()[env.as_wide().len() - 2], 0);
}

#[test]
fn empty_env_block_is_double_nul_terminated() {
    let env = EnvBlock::from_pairs(std::iter::empty::<(&str, &str)>());

    assert_eq!(env.as_wide(), &[0, 0]);
}

#[test]
fn env_block_accepts_dynamic_os_string_pairs() {
    let env = EnvBlock::from_pairs(vec![
        (OsString::from("A"), OsString::from("1")),
        (OsString::from("B"), OsString::from("2")),
    ]);

    let expected = "A=1\0B=2\0\0".encode_utf16().collect::<Vec<_>>();
    assert_eq!(env.as_wide(), expected);
}
