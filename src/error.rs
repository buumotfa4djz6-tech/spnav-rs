//! Error types for spnav-rs.

use std::io;

/// Errors that can occur when using spnav-rs.
///
/// This enum covers all error conditions across both the UNIX socket
/// and X11 Magellan protocol implementations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error (socket read/write failure).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Connection is already open (cannot open twice).
    #[error("connection already open")]
    AlreadyOpen,
    /// Connection is not open (cannot send request).
    #[error("connection not open")]
    NotOpen,
    /// Protocol-level error (invalid response, unexpected data).
    #[error("protocol error: {0}")]
    Protocol(String),
    /// Request timed out waiting for daemon response.
    #[error("request timed out")]
    Timeout,
    /// Daemon returned a failure status code.
    #[error("daemon returned failure status")]
    DaemonFailure,
    /// spacenavd socket not found (daemon may not be running).
    #[error("socket not found (is spacenavd running?)")]
    SocketNotFound,
    /// X11 resource ID error (with `x11` feature).
    #[cfg(feature = "x11")]
    #[error("X11 resource ID error: {0}")]
    X11(#[from] x11rb::errors::ReplyOrIdError),
    /// X11 connection error (with `x11` feature).
    #[cfg(feature = "x11")]
    #[error("X11 connection error: {0}")]
    X11Conn(#[from] x11rb::errors::ConnectionError),
    /// X11 reply error (with `x11` feature).
    #[cfg(feature = "x11")]
    #[error("X11 reply error: {0}")]
    X11Reply(#[from] x11rb::errors::ReplyError),
}

/// Convenience type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
