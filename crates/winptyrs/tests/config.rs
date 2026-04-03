use winptyrs::{AgentBuilder, AgentFlags, EnvBlock, Error, MouseMode, PtySize};

#[test]
fn rejects_zero_dimensions() {
    let err = PtySize::new(0, 24).unwrap_err();
    assert!(matches!(err, Error::InvalidSize { cols: 0, rows: 24 }));
}

#[test]
fn builder_defaults_match_the_legacy_crate_defaults() {
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
