use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("connection already open")]
    AlreadyOpen,
    #[error("connection not open")]
    NotOpen,
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("request timed out")]
    Timeout,
    #[error("daemon returned failure status")]
    DaemonFailure,
    #[error("socket not found (is spacenavd running?)")]
    SocketNotFound,
    #[cfg(feature = "x11")]
    #[error("X11 resource ID error: {0}")]
    X11(#[from] x11rb::errors::ReplyOrIdError),
    #[cfg(feature = "x11")]
    #[error("X11 connection error: {0}")]
    X11Conn(#[from] x11rb::errors::ConnectionError),
    #[cfg(feature = "x11")]
    #[error("X11 reply error: {0}")]
    X11Reply(#[from] x11rb::errors::ReplyError),
}

pub type Result<T> = std::result::Result<T, Error>;
