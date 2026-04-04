use std::ptr::{self, NonNull};

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE,
    OPEN_EXISTING,
};

use crate::child::Child;
use crate::config::{AgentBuilder, MouseMode, SpawnConfig};
use crate::handle::{
    last_winpty_error, wide_nul, windows_io_error, OwnedHandle, OwnedSpawnConfig, OwnedWinpty,
    OwnedWinptyConfig,
};
use crate::io::{read_blocking, read_nonblocking, write_utf8};
use crate::Result;

pub struct Pty {
    pub(crate) agent: OwnedWinpty,
    pub(crate) conin: OwnedHandle,
    pub(crate) conout: OwnedHandle,
}

impl AgentBuilder {
    pub fn open(self) -> Result<Pty> {
        let mut err = ptr::null_mut();
        let cfg = unsafe {
            winptyrs_sys::winpty_config_new(self.configured_agent_flags().bits(), &mut err)
        };
        let cfg = NonNull::new(cfg).ok_or_else(|| last_winpty_error(err))?;
        let cfg = OwnedWinptyConfig(cfg);

        unsafe {
            winptyrs_sys::winpty_config_set_initial_size(
                cfg.0.as_ptr(),
                self.pty_size().cols as i32,
                self.pty_size().rows as i32,
            );
            winptyrs_sys::winpty_config_set_mouse_mode(
                cfg.0.as_ptr(),
                match self.selected_mouse_mode() {
                    MouseMode::None => winptyrs_sys::WINPTY_MOUSE_MODE_NONE,
                    MouseMode::Auto => winptyrs_sys::WINPTY_MOUSE_MODE_AUTO,
                    MouseMode::Force => winptyrs_sys::WINPTY_MOUSE_MODE_FORCE,
                },
            );
            winptyrs_sys::winpty_config_set_agent_timeout(
                cfg.0.as_ptr(),
                self.configured_timeout_ms(),
            );
        }

        let mut err = ptr::null_mut();
        let raw_agent = unsafe { winptyrs_sys::winpty_open(cfg.0.as_ptr(), &mut err) };
        let raw_agent = NonNull::new(raw_agent).ok_or_else(|| last_winpty_error(err))?;
        let agent = OwnedWinpty(raw_agent);

        let conin_name = unsafe { winptyrs_sys::winpty_conin_name(agent.0.as_ptr()) };
        let conout_name = unsafe { winptyrs_sys::winpty_conout_name(agent.0.as_ptr()) };

        let conin = unsafe {
            CreateFileW(
                conin_name,
                FILE_GENERIC_WRITE,
                FILE_SHARE_NONE,
                ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        };
        if conin == INVALID_HANDLE_VALUE {
            return Err(windows_io_error().into());
        }

        let conout = unsafe {
            CreateFileW(
                conout_name,
                FILE_GENERIC_READ,
                FILE_SHARE_NONE,
                ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        };
        if conout == INVALID_HANDLE_VALUE {
            return Err(windows_io_error().into());
        }

        Ok(Pty {
            agent,
            conin: OwnedHandle(conin),
            conout: OwnedHandle(conout),
        })
    }
}

impl Pty {
    pub fn spawn(&mut self, config: SpawnConfig) -> Result<Child> {
        let app = wide_nul(config.appname.as_os_str());
        let cmdline = config
            .cmdline
            .as_ref()
            .map(|value| wide_nul(value.as_os_str()));
        let cwd = config.cwd.as_ref().map(|value| wide_nul(value.as_os_str()));
        let env = config.env.as_ref().map(|value| value.as_wide().to_vec());

        let mut err = ptr::null_mut();
        let raw_spawn = unsafe {
            winptyrs_sys::winpty_spawn_config_new(
                winptyrs_sys::WINPTY_SPAWN_FLAG_AUTO_SHUTDOWN
                    | winptyrs_sys::WINPTY_SPAWN_FLAG_EXIT_AFTER_SHUTDOWN,
                app.as_ptr(),
                cmdline.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                cwd.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                env.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                &mut err,
            )
        };
        let raw_spawn = NonNull::new(raw_spawn).ok_or_else(|| last_winpty_error(err))?;
        let spawn_cfg = OwnedSpawnConfig(raw_spawn);

        let mut process_handle: winptyrs_sys::HANDLE = ptr::null_mut();
        let mut os_error = 0u32;
        let ok = unsafe {
            winptyrs_sys::winpty_spawn(
                self.agent.0.as_ptr(),
                spawn_cfg.0.as_ptr(),
                &mut process_handle,
                ptr::null_mut(),
                &mut os_error,
                &mut err,
            )
        };

        if !ok {
            if !err.is_null() {
                return Err(last_winpty_error(err));
            }

            return Err(crate::Error::Winpty(format!(
                "CreateProcess failed with OS error {os_error}"
            )));
        }

        Ok(Child {
            process: OwnedHandle(process_handle),
        })
    }

    pub fn read_blocking(&self) -> Result<String> {
        read_blocking(self.conout.0)
    }

    pub fn read_nonblocking(&self) -> Result<String> {
        read_nonblocking(self.conout.0)
    }

    pub fn write(&self, input: &str) -> Result<usize> {
        write_utf8(self.conin.0, input)
    }

    pub fn resize(&self, size: crate::PtySize) -> Result<()> {
        let mut err = ptr::null_mut();
        let ok = unsafe {
            winptyrs_sys::winpty_set_size(
                self.agent.0.as_ptr(),
                size.cols as i32,
                size.rows as i32,
                &mut err,
            )
        };

        if ok {
            Ok(())
        } else {
            Err(last_winpty_error(err))
        }
    }
}
