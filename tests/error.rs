//! Integration tests for error types.

use spnav_rs::error::Error;

// ─── Error display pins ─────────────────────────────────────────────────────

#[test]
fn io_error_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err = Error::Io(io_err);
    assert!(err.to_string().contains("I/O error"));
    assert!(err.to_string().contains("refused"));
}

#[test]
fn already_open_display() {
    let err = Error::AlreadyOpen;
    assert!(err.to_string().contains("already open"));
}

#[test]
fn not_open_display() {
    let err = Error::NotOpen;
    assert!(err.to_string().contains("not open"));
}

#[test]
fn protocol_error_display() {
    let err = Error::Protocol("invalid frame".into());
    assert!(err.to_string().contains("protocol error"));
    assert!(err.to_string().contains("invalid frame"));
}

#[test]
fn timeout_display() {
    let err = Error::Timeout;
    assert!(err.to_string().contains("timed out"));
}

#[test]
fn daemon_failure_display() {
    let err = Error::DaemonFailure;
    assert!(err.to_string().contains("failure"));
}

#[test]
fn socket_not_found_display() {
    let err = Error::SocketNotFound;
    assert!(err.to_string().contains("spacenavd"));
}

// ─── Error trait impls ──────────────────────────────────────────────────────

#[test]
fn error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Error>();
}

#[test]
fn error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Error>();
}

#[test]
fn error_is_std_error() {
    fn assert_std_error<T: std::error::Error>() {}
    assert_std_error::<Error>();
}
