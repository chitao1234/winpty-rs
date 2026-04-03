mod bindings;

pub use bindings::{
    winpty_config_free, winpty_config_new, winpty_config_set_agent_timeout,
    winpty_config_set_initial_size, winpty_config_set_mouse_mode, winpty_config_t,
    winpty_conin_name, winpty_conout_name, winpty_error_free, winpty_error_msg, winpty_error_ptr_t,
    winpty_error_t, winpty_free, winpty_open, winpty_set_size, winpty_spawn,
    winpty_spawn_config_free, winpty_spawn_config_new, winpty_spawn_config_t, winpty_t, HANDLE,
    WINPTY_FLAG_ALLOW_CURPROC_DESKTOP_CREATION, WINPTY_FLAG_COLOR_ESCAPES, WINPTY_FLAG_CONERR,
    WINPTY_FLAG_PLAIN_OUTPUT, WINPTY_MOUSE_MODE_AUTO, WINPTY_MOUSE_MODE_FORCE,
    WINPTY_MOUSE_MODE_NONE, WINPTY_SPAWN_FLAG_AUTO_SHUTDOWN, WINPTY_SPAWN_FLAG_EXIT_AFTER_SHUTDOWN,
};
