use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{GetFileSizeEx, ReadFile, WriteFile};

use crate::handle::windows_io_error;
use crate::{Error, Result};

const BUFFER_SIZE: usize = 32 * 1024;

pub(crate) fn read_nonblocking(handle: HANDLE) -> Result<String> {
    let mut available = 0i64;
    unsafe { GetFileSizeEx(handle, &mut available) }.map_err(windows_io_error)?;

    if available == 0 {
        return Ok(String::new());
    }

    read_blocking(handle)
}

pub(crate) fn read_blocking(handle: HANDLE) -> Result<String> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut read = 0u32;
    unsafe { ReadFile(handle, Some(buffer.as_mut_slice()), Some(&mut read), None) }
        .map_err(windows_io_error)?;

    if read == 0 {
        return Err(Error::Eof);
    }

    buffer.truncate(read as usize);
    String::from_utf8(buffer).map_err(|_| Error::InvalidUtf16)
}

pub(crate) fn write_utf8(handle: HANDLE, input: &str) -> Result<usize> {
    let bytes = input.as_bytes();
    let mut written = 0u32;
    unsafe { WriteFile(handle, Some(bytes), Some(&mut written), None) }
        .map_err(windows_io_error)?;
    Ok(written as usize)
}
