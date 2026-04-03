//! This module provides a [`super::PTY`] backend that uses
//! [conpty](https://docs.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session) as its implementation.
//! This backend is available on Windows 10 starting from build number 1809.

// Actual implementation if winpty is available
mod calls;
#[cfg(conpty_available)]
mod pty_impl;

#[cfg(conpty_available)]
mod win_bindings;

#[cfg(all(conpty_available, conpty_local_available))]
mod bindings;

#[cfg(conpty_available)]
pub use pty_impl::ConPTY;

// Default implementation if winpty is not available
#[cfg(not(conpty_available))]
mod default_impl;

#[cfg(not(conpty_available))]
pub use default_impl::ConPTY;
