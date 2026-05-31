use std::ffi::{OsStr, OsString};

use bitflags::bitflags;

use crate::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PtySize {
    pub cols: u16,
    pub rows: u16,
}

impl PtySize {
    pub fn new(cols: u16, rows: u16) -> Result<Self> {
        if cols == 0 || rows == 0 {
            return Err(Error::InvalidSize { cols, rows });
        }

        Ok(Self { cols, rows })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseMode {
    None,
    Auto,
    Force,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct AgentFlags: u64 {
        const CONERR = winptyrs_sys::WINPTY_FLAG_CONERR;
        const PLAIN_OUTPUT = winptyrs_sys::WINPTY_FLAG_PLAIN_OUTPUT;
        const COLOR_ESCAPES = winptyrs_sys::WINPTY_FLAG_COLOR_ESCAPES;
        const ALLOW_CURPROC_DESKTOP_CREATION =
            winptyrs_sys::WINPTY_FLAG_ALLOW_CURPROC_DESKTOP_CREATION;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvBlock {
    wide: Vec<u16>,
}

impl EnvBlock {
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let mut wide = Vec::new();

        for (key, value) in pairs {
            push_os_wide(&mut wide, key.as_ref());
            wide.push(b'=' as u16);
            push_os_wide(&mut wide, value.as_ref());
            wide.push(0);
        }

        if wide.is_empty() {
            wide.push(0);
        }
        wide.push(0);
        Self { wide }
    }

    pub fn as_wide(&self) -> &[u16] {
        &self.wide
    }
}

fn push_os_wide(buffer: &mut Vec<u16>, value: &OsStr) {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        buffer.extend(value.encode_wide());
    }

    #[cfg(not(windows))]
    {
        buffer.extend(value.to_string_lossy().encode_utf16());
    }
}

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, Debug)]
pub struct SpawnConfig {
    pub(crate) appname: OsString,
    pub(crate) cmdline: Option<OsString>,
    pub(crate) cwd: Option<OsString>,
    pub(crate) env: Option<EnvBlock>,
}

impl SpawnConfig {
    pub fn new(appname: impl Into<OsString>) -> Self {
        Self {
            appname: appname.into(),
            cmdline: None,
            cwd: None,
            env: None,
        }
    }

    pub fn cmdline(mut self, cmdline: impl Into<OsString>) -> Self {
        self.cmdline = Some(cmdline.into());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<OsString>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn env(mut self, env: EnvBlock) -> Self {
        self.env = Some(env);
        self
    }
}

#[derive(Clone, Debug)]
pub struct AgentBuilder {
    size: PtySize,
    mouse_mode: MouseMode,
    timeout_ms: u32,
    agent_flags: AgentFlags,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            size: PtySize::new(80, 24).expect("default pty size is valid"),
            mouse_mode: MouseMode::None,
            timeout_ms: 10_000,
            agent_flags: AgentFlags::COLOR_ESCAPES,
        }
    }

    pub fn size(mut self, size: PtySize) -> Self {
        self.size = size;
        self
    }

    pub fn mouse_mode(mut self, mouse_mode: MouseMode) -> Self {
        self.mouse_mode = mouse_mode;
        self
    }

    pub fn timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn agent_flags(mut self, agent_flags: AgentFlags) -> Self {
        self.agent_flags = agent_flags;
        self
    }

    pub fn pty_size(&self) -> PtySize {
        self.size
    }

    pub fn selected_mouse_mode(&self) -> MouseMode {
        self.mouse_mode
    }

    pub fn configured_timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    pub fn configured_agent_flags(&self) -> AgentFlags {
        self.agent_flags
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_size_accepts_nonzero_dimensions() {
        assert_eq!(
            PtySize::new(132, 43).unwrap(),
            PtySize {
                cols: 132,
                rows: 43
            }
        );
    }

    #[test]
    fn pty_size_rejects_zero_rows() {
        let err = PtySize::new(80, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidSize { cols: 80, rows: 0 }));
    }

    #[test]
    fn agent_builder_setters_update_configuration() {
        let flags = AgentFlags::CONERR | AgentFlags::PLAIN_OUTPUT;
        let builder = AgentBuilder::new()
            .size(PtySize::new(120, 32).unwrap())
            .mouse_mode(MouseMode::Force)
            .timeout_ms(2_500)
            .agent_flags(flags);

        assert_eq!(
            builder.pty_size(),
            PtySize {
                cols: 120,
                rows: 32
            }
        );
        assert_eq!(builder.selected_mouse_mode(), MouseMode::Force);
        assert_eq!(builder.configured_timeout_ms(), 2_500);
        assert_eq!(builder.configured_agent_flags(), flags);
    }

    #[test]
    fn default_agent_builder_matches_new() {
        let default = AgentBuilder::default();
        let new = AgentBuilder::new();

        assert_eq!(default.pty_size(), new.pty_size());
        assert_eq!(default.selected_mouse_mode(), new.selected_mouse_mode());
        assert_eq!(default.configured_timeout_ms(), new.configured_timeout_ms());
        assert_eq!(
            default.configured_agent_flags(),
            new.configured_agent_flags()
        );
    }

    #[test]
    fn spawn_config_defaults_to_appname_only() {
        let config = SpawnConfig::new("cmd.exe");

        assert_eq!(config.appname, OsString::from("cmd.exe"));
        assert_eq!(config.cmdline, None);
        assert_eq!(config.cwd, None);
        assert_eq!(config.env, None);
    }

    #[test]
    fn spawn_config_setters_record_optional_fields() {
        let env = EnvBlock::from_pairs([("WINPTYRS_TEST", "1")]);
        let config = SpawnConfig::new("probe.exe")
            .cmdline("--print-env")
            .cwd("C:\\work")
            .env(env.clone());

        assert_eq!(config.appname, OsString::from("probe.exe"));
        assert_eq!(config.cmdline, Some(OsString::from("--print-env")));
        assert_eq!(config.cwd, Some(OsString::from("C:\\work")));
        assert_eq!(config.env, Some(env));
    }
}
