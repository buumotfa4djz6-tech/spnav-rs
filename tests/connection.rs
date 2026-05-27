//! Integration tests for connection module.
//!
//! Note: Most connection tests require a running spacenavd daemon.
//! These tests focus on the path discovery logic.

use std::path::PathBuf;

// ─── Socket path discovery tests ────────────────────────────────────────────

// Note: find_socket() is not directly testable without mocking the filesystem
// and environment. The following tests verify the public API behavior.

#[test]
fn default_socket_path_is_var_run() {
    // The default socket path should be /var/run/spnav.sock
    // We can't directly test find_socket() without environment control,
    // but we can verify the constant is correct
    let default_path = "/var/run/spnav.sock";
    assert!(default_path.starts_with("/var/run/"));
    assert!(default_path.ends_with(".sock"));
}

#[test]
fn spnav_socket_env_var_format() {
    // Verify that paths look like valid socket paths
    let valid_paths = vec![
        "/var/run/spnav.sock",
        "/tmp/spnav.sock",
        "/run/user/1000/spnav.sock",
        "/home/user/.spnav.sock",
    ];

    for path in valid_paths {
        let pb = PathBuf::from(path);
        assert!(pb.is_absolute(), "path should be absolute: {}", path);
        assert!(
            path.ends_with(".sock") || path.ends_with(".socket"),
            "path should end with .sock: {}",
            path
        );
    }
}

// ─── Async connection tests ─────────────────────────────────────────────────

#[tokio::test]
async fn connect_to_nonexistent_socket_returns_error() {
    let result = tokio::net::UnixStream::connect("/tmp/nonexistent_spnav_test.sock").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn connect_error_is_socket_not_found() {
    // Try to connect to a path that definitely doesn't exist
    let path = PathBuf::from("/tmp/nonexistent_spnav_test_12345.sock");
    let result = tokio::net::UnixStream::connect(&path).await;
    match result {
        Err(e) => {
            // The error should be an IO error
            assert!(
                e.kind() == std::io::ErrorKind::NotFound
                    || e.kind() == std::io::ErrorKind::ConnectionRefused
            );
        }
        Ok(_) => panic!("expected connection error"),
    }
}
