use windows::Win32::Foundation::{STILL_ACTIVE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessId, WaitForSingleObject, INFINITE,
};

use crate::handle::{windows_io_error, OwnedHandle};
use crate::{Error, Result};

pub struct Child {
    pub(crate) process: OwnedHandle,
}

impl Child {
    pub fn id(&self) -> u32 {
        unsafe { GetProcessId(self.process.0) }
    }

    pub fn is_alive(&self) -> Result<bool> {
        Ok(self.try_wait()?.is_none())
    }

    pub fn try_wait(&self) -> Result<Option<u32>> {
        let mut exit_code = 0u32;
        unsafe { GetExitCodeProcess(self.process.0, &mut exit_code) }.map_err(windows_io_error)?;

        if exit_code == STILL_ACTIVE.0 as u32 {
            Ok(None)
        } else {
            Ok(Some(exit_code))
        }
    }

    pub fn wait(&self) -> Result<u32> {
        match unsafe { WaitForSingleObject(self.process.0, INFINITE) } {
            WAIT_OBJECT_0 => self.try_wait()?.ok_or(Error::ProcessNotSpawned),
            _ => Err(Error::Winpty(
                "waiting for the child process failed".to_owned(),
            )),
        }
    }
}
