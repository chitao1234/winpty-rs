#![allow(non_camel_case_types)]

use core::ffi::c_void;

pub type HANDLE = *mut c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_error_s {
    _unused: [u8; 0],
}

pub type winpty_error_t = winpty_error_s;
pub type winpty_error_ptr_t = *mut winpty_error_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_config_s {
    _unused: [u8; 0],
}

pub type winpty_config_t = winpty_config_s;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_s {
    _unused: [u8; 0],
}

pub type winpty_t = winpty_s;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_spawn_config_s {
    _unused: [u8; 0],
}

pub type winpty_spawn_config_t = winpty_spawn_config_s;

pub const WINPTY_FLAG_CONERR: u64 = 0x1;
pub const WINPTY_FLAG_PLAIN_OUTPUT: u64 = 0x2;
pub const WINPTY_FLAG_COLOR_ESCAPES: u64 = 0x4;
pub const WINPTY_FLAG_ALLOW_CURPROC_DESKTOP_CREATION: u64 = 0x8;

pub const WINPTY_MOUSE_MODE_NONE: i32 = 0;
pub const WINPTY_MOUSE_MODE_AUTO: i32 = 1;
pub const WINPTY_MOUSE_MODE_FORCE: i32 = 2;

pub const WINPTY_SPAWN_FLAG_AUTO_SHUTDOWN: u64 = 0x1;
pub const WINPTY_SPAWN_FLAG_EXIT_AFTER_SHUTDOWN: u64 = 0x2;

unsafe extern "C" {
    pub fn winpty_error_msg(err: winpty_error_ptr_t) -> *const u16;
    pub fn winpty_error_free(err: winpty_error_ptr_t);

    pub fn winpty_config_new(flags: u64, err: *mut winpty_error_ptr_t) -> *mut winpty_config_t;
    pub fn winpty_config_free(cfg: *mut winpty_config_t);
    pub fn winpty_config_set_initial_size(cfg: *mut winpty_config_t, cols: i32, rows: i32);
    pub fn winpty_config_set_mouse_mode(cfg: *mut winpty_config_t, mouse_mode: i32);
    pub fn winpty_config_set_agent_timeout(cfg: *mut winpty_config_t, timeout: u32);

    pub fn winpty_open(cfg: *const winpty_config_t, err: *mut winpty_error_ptr_t) -> *mut winpty_t;
    pub fn winpty_free(wp: *mut winpty_t);
    pub fn winpty_conin_name(wp: *mut winpty_t) -> *const u16;
    pub fn winpty_conout_name(wp: *mut winpty_t) -> *const u16;

    pub fn winpty_spawn_config_new(
        spawn_flags: u64,
        appname: *const u16,
        cmdline: *const u16,
        cwd: *const u16,
        env: *const u16,
        err: *mut winpty_error_ptr_t,
    ) -> *mut winpty_spawn_config_t;
    pub fn winpty_spawn_config_free(cfg: *mut winpty_spawn_config_t);
    pub fn winpty_spawn(
        wp: *mut winpty_t,
        cfg: *const winpty_spawn_config_t,
        process_handle: *mut HANDLE,
        thread_handle: *mut HANDLE,
        create_process_error: *mut u32,
        err: *mut winpty_error_ptr_t,
    ) -> bool;

    pub fn winpty_set_size(
        wp: *mut winpty_t,
        cols: i32,
        rows: i32,
        err: *mut winpty_error_ptr_t,
    ) -> bool;
}
