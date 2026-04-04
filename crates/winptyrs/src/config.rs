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
