// #[cfg(all(feature="conpty", feature="conpty_local"))]
// mod bindings;
#![allow(non_snake_case)]

use windows::core::Result;
#[cfg(conpty_local_available)]
use windows::core::{Error, HRESULT};
#[cfg(conpty_local_available)]
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::HPCON;
#[cfg(conpty_local_available)]
use windows::Win32::System::Console::COORD;

#[cfg(conpty_local_available)]
use std::ffi::c_void;
#[cfg(conpty_local_available)]
use std::mem::MaybeUninit;
#[cfg(conpty_local_available)]
use std::os::windows::raw;

#[cfg(all(conpty_available, not(conpty_local_available)))]
pub use windows::Win32::System::Console::{CreatePseudoConsole, ResizePseudoConsole, ClosePseudoConsole};

#[cfg(all(conpty_available, conpty_local_available))]
use super::bindings::{
    ConptyClearPseudoConsole, ConptyClosePseudoConsole, ConptyCreatePseudoConsole,
    ConptyResizePseudoConsole, ConptyShowHidePseudoConsole,
};

#[cfg(all(conpty_available, conpty_local_available))]
pub unsafe fn CreatePseudoConsole(
    size: COORD,
    hInput: HANDLE,
    hOutput: HANDLE,
    dwFlags: u32,
) -> Result<HPCON> {
    let mut console_handle_uninit = MaybeUninit::<HPCON>::uninit();
    let result_code = ConptyCreatePseudoConsole(
        size,
        hInput.0 as raw::HANDLE,
        hOutput.0 as raw::HANDLE,
        dwFlags,
        console_handle_uninit.as_mut_ptr() as *mut c_void,
    );

    let result = HRESULT::from_nt(result_code);
    if result.is_err() {
        Err(Error::from_hresult(result))
    } else {
        let console_handle = console_handle_uninit.assume_init();
        Ok(console_handle)
    }
}

#[cfg(all(conpty_available, conpty_local_available))]
pub unsafe fn ResizePseudoConsole(hPC: HPCON, size: COORD) -> Result<()> {
    let result_code = ConptyResizePseudoConsole(hPC.0 as *mut c_void, size);

    let result = HRESULT::from_nt(result_code);
    if result.is_err() {
        Err(Error::from_hresult(result))
    } else {
        Ok(())
    }
}

#[cfg(all(conpty_available, conpty_local_available))]
pub unsafe fn ClearPseudoConsole(hPC: HPCON) -> Result<()> {
    let result_code = ConptyClearPseudoConsole(hPC.0 as *mut c_void);

    let result = HRESULT::from_nt(result_code);
    if result.is_err() {
        Err(Error::from_hresult(result))
    } else {
        Ok(())
    }
}

#[cfg(all(conpty_available, conpty_local_available))]
pub unsafe fn ClosePseudoConsole(hPC: HPCON) -> Result<()> {
    let result_code = ConptyClosePseudoConsole(hPC.0 as *mut c_void);

    let result = HRESULT::from_nt(result_code);
    if result.is_err() {
        Err(Error::from_hresult(result))
    } else {
        Ok(())
    }
}

#[cfg(all(conpty_available, conpty_local_available))]
pub unsafe fn ShowHidePseudoConsole(hPC: HPCON, show: bool) -> Result<()> {
    let result_code = ConptyShowHidePseudoConsole(hPC.0 as *mut c_void, show);

    let result = HRESULT::from_nt(result_code);
    if result.is_err() {
        Err(Error::from_hresult(result))
    } else {
        Ok(())
    }
}

#[cfg(all(conpty_available, not(conpty_local_available)))]
pub unsafe fn ShowHidePseudoConsole(_hPC: HPCON, _show: bool) -> Result<()> {
    Ok(())
}
