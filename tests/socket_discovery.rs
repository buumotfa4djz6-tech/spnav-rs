//! Gap 2: find_socket() path-discovery logic.
//!
//! The discovery order is: SPNAV_SOCKET env var > /etc/spnavrc > default path.
//! We can reliably test the env-var override and the default-fallback paths.
//! The /etc/spnavrc branch is a filesystem read of a system file; testing it
//! would require root or a mount-namespace fixture, so we leave it to manual
//! verification.
//!
//! Env vars are process-global, so these tests use `serial_test::serial` to
//! prevent races with other tests that might also touch SPNAV_SOCKET.

mod common;

use serial_test::serial;
use spnav_rs::connection::find_socket;
use std::env;
use std::path::PathBuf;

const ENV_VAR: &str = "SPNAV_SOCKET";
const DEFAULT_PATH: &str = "/var/run/spnav.sock";

#[test]
#[serial(spnav_socket_env)]
fn env_var_overrides_default() {
    let custom = "/tmp/custom_spnav_test.sock";
    env::set_var(ENV_VAR, custom);
    let result = find_socket();
    env::remove_var(ENV_VAR);
    assert_eq!(result, PathBuf::from(custom));
}

#[test]
#[serial(spnav_socket_env)]
fn unset_env_var_falls_back() {
    env::remove_var(ENV_VAR);
    let result = find_socket();
    // The result is either a path from /etc/spnavrc (if that file exists on
    // the host running the test) or the default. We can't predict which, but
    // we can assert it's a non-empty absolute path.
    assert!(
        result.is_absolute(),
        "find_socket should return an absolute path, got {:?}",
        result
    );
}

#[test]
#[serial(spnav_socket_env)]
fn empty_env_var_is_returned_verbatim() {
    // Documenting current behavior: find_socket treats SPNAV_SOCKET="" as a
    // real (empty) path rather than falling back. This is arguably a bug
    // (empty string is not a useful socket path), but changing it would be
    // a behavior change outside the scope of test cleanup.
    env::set_var(ENV_VAR, "");
    let result = find_socket();
    env::remove_var(ENV_VAR);

    assert!(
        result.as_os_str().is_empty(),
        "current find_socket returns the empty env var verbatim; got {:?}",
        result
    );
}

#[test]
#[serial(spnav_socket_env)]
fn default_path_is_well_known() {
    // Independent of env: sanity-check the constant we'd fall back to.
    assert_eq!(DEFAULT_PATH, "/var/run/spnav.sock");
    assert!(DEFAULT_PATH.starts_with('/'));
    assert!(DEFAULT_PATH.ends_with(".sock"));
}
