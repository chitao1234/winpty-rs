use std::ffi::OsStr;
use std::ptr::NonNull;

use windows::Win32::Foundation::{CloseHandle, HANDLE};

use crate::Error;

pub(crate) struct OwnedHandle(pub(crate) HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

pub(crate) struct OwnedWinptyConfig(pub(crate) NonNull<winptyrs_sys::winpty_config_t>);
pub(crate) struct OwnedWinpty(pub(crate) NonNull<winptyrs_sys::winpty_t>);
pub(crate) struct OwnedSpawnConfig(pub(crate) NonNull<winptyrs_sys::winpty_spawn_config_t>);

impl Drop for OwnedWinptyConfig {
    fn drop(&mut self) {
        unsafe { winptyrs_sys::winpty_config_free(self.0.as_ptr()) }
    }
}

impl Drop for OwnedWinpty {
    fn drop(&mut self) {
        unsafe { winptyrs_sys::winpty_free(self.0.as_ptr()) }
    }
}

impl Drop for OwnedSpawnConfig {
    fn drop(&mut self) {
        unsafe { winptyrs_sys::winpty_spawn_config_free(self.0.as_ptr()) }
    }
}

pub(crate) fn last_winpty_error(err: winptyrs_sys::winpty_error_ptr_t) -> Error {
    unsafe {
        if err.is_null() {
            return Error::Winpty("winpty returned an unknown error".to_owned());
        }

        let msg_ptr = winptyrs_sys::winpty_error_msg(err);
        if msg_ptr.is_null() {
            winptyrs_sys::winpty_error_free(err);
            return Error::Winpty("winpty returned an empty error message".to_owned());
        }

        let mut len = 0usize;
        while *msg_ptr.add(len) != 0 {
            len += 1;
        }

        let message = String::from_utf16(std::slice::from_raw_parts(msg_ptr, len))
            .unwrap_or_else(|_| "winpty returned invalid UTF-16".to_owned());
        winptyrs_sys::winpty_error_free(err);
        Error::Winpty(message)
    }
}

pub(crate) fn wide_nul(value: &OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    let mut wide: Vec<u16> = value.encode_wide().collect();
    wide.push(0);
    wide
}

pub(crate) fn windows_io_error(err: windows::core::Error) -> std::io::Error {
    std::io::Error::other(err.to_string())
}
