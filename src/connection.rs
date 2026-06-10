//! Low-level UNIX socket connection management for spacenavd.
//!
//! This module handles the transport layer: discovering the spacenavd socket path,
//! establishing the connection, and performing the initial protocol handshake.
//!
//! Most users should use [`SpnavClient`](crate::SpnavClient) from the
//! [`client`](crate::client) module instead of calling these functions directly.
//! This module is exposed for advanced use cases like custom connection handling
//! or testing.
//!
//! # Socket Discovery
//!
//! The socket path is determined by [`find_socket()`], which checks in order:
//!
//! 1. `SPNAV_SOCKET` environment variable
//! 2. `/etc/spnavrc` configuration file (socket = line)
//! 3. Default path: `/var/run/spnav.sock`
//!
//! # Handshake
//!
//! After connecting, [`handshake()`] negotiates the protocol version with the daemon.
//! The daemon may support protocol v0 (legacy) or v1 (with extended features).
//! The handshake also handles sensitivity initialization for v0 connections.

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use crate::error::{Error, Result};

const DEFAULT_SOCKET: &str = "/var/run/spnav.sock";
const CONFIG_FILE: &str = "/etc/spnavrc";

/// Discover the spacenavd socket path.
/// Priority: SPNAV_SOCKET env var > /etc/spnavrc "socket =" line > DEFAULT_SOCKET
pub fn find_socket() -> PathBuf {
    if let Ok(path) = env::var("SPNAV_SOCKET") {
        return PathBuf::from(path);
    }
    if let Some(path) = parse_spnavrc() {
        return path;
    }
    PathBuf::from(DEFAULT_SOCKET)
}

fn parse_spnavrc() -> Option<PathBuf> {
    let file = File::open(CONFIG_FILE).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.ok()?;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            if key == "socket" {
                let value = trimmed[eq_pos + 1..].trim();
                if !value.is_empty() {
                    return Some(PathBuf::from(value));
                }
            }
        }
    }
    None
}

/// Connect to the daemon. The caller must perform the protocol handshake separately.
pub async fn connect(path: &Path) -> Result<UnixStream> {
    UnixStream::connect(path)
        .await
        .map_err(|_| Error::SocketNotFound)
}

/// Perform protocol handshake on an already-connected stream.
/// Returns the negotiated protocol version.
pub async fn handshake(stream: &mut UnixStream) -> Result<u8> {
    use crate::protocol::{self, req, ReqResp, REQ_TAG};
    use tokio::io::AsyncReadExt;

    // Send protocol change request as a single i32
    let cmd: u32 = (REQ_TAG | req::CHANGE_PROTO | protocol::MAX_PROTO_VER) as u32;
    stream.write_all(&cmd.to_ne_bytes()).await?;

    // Wait for response with 300ms timeout
    let mut resp_buf = [0u8; 32];
    if let Ok(Ok(_)) = tokio::time::timeout(
        std::time::Duration::from_millis(300),
        stream.read_exact(&mut resp_buf),
    )
    .await
    {
        let rr = ReqResp::from_bytes(&resp_buf);
        if rr.is_response() {
            return Ok((rr.type_ & 0xff) as u8);
        }
    }

    // Timeout or invalid - fall back to v0, restore sensitivity to 1.0
    let sens = 1.0f32;
    stream.write_all(&sens.to_ne_bytes()).await?;
    Ok(0)
}
