use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("winpty error: {0}")]
    Winpty(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid pty size: cols={cols}, rows={rows}")]
    InvalidSize { cols: u16, rows: u16 },
    #[error("invalid UTF-16 data returned by winpty")]
    InvalidUtf16,
    #[error("process has not been spawned yet")]
    ProcessNotSpawned,
    #[error("pty output reached EOF")]
    Eof,
}
