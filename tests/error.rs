//! Integration tests for error types.

use spnav_rs::error::{Error, Result};

// ─── Error display tests ────────────────────────────────────────────────────

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

// ─── Error from conversions ─────────────────────────────────────────────────

#[test]
fn error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err: Error = io_err.into();
    match err {
        Error::Io(_) => {}
        _ => panic!("expected Io error"),
    }
}

// ─── Result alias tests ─────────────────────────────────────────────────────

#[test]
fn result_ok() {
    fn returns_ok() -> Result<u32> {
        Ok(42)
    }
    assert_eq!(returns_ok().unwrap(), 42);
}

#[test]
fn result_err() {
    fn returns_err() -> Result<u32> {
        Err(Error::Timeout)
    }
    assert!(returns_err().is_err());
}

#[test]
fn result_err_is_timeout() {
    fn returns_err() -> Result<u32> {
        Err(Error::Timeout)
    }
    match returns_err() {
        Err(Error::Timeout) => {}
        _ => panic!("expected Timeout"),
    }
}

// ─── Error trait tests ──────────────────────────────────────────────────────

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

// ─── Error equality tests ───────────────────────────────────────────────────

#[test]
fn error_debug() {
    let err = Error::Timeout;
    let debug = format!("{:?}", err);
    assert!(debug.contains("Timeout"));
}

#[test]
fn protocol_error_with_different_messages() {
    let err1 = Error::Protocol("msg1".into());
    let err2 = Error::Protocol("msg2".into());
    // Different messages should produce different error strings
    assert_ne!(err1.to_string(), err2.to_string());
}
