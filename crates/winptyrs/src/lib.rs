mod config;
mod error;

#[cfg(windows)]
mod agent;
#[cfg(windows)]
mod child;
#[cfg(windows)]
mod handle;
#[cfg(windows)]
mod io;

#[cfg(windows)]
pub use agent::Pty;
#[cfg(windows)]
pub use child::Child;
pub use config::{AgentBuilder, AgentFlags, EnvBlock, MouseMode, PtySize, SpawnConfig};
pub use error::{Error, Result};
