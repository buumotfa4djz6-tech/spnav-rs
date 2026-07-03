//! Shared test helpers: fake spacenavd simulator + frame encoders.
//!
//! Used by the integration tests in this directory. Include via `mod common;`
//! in any test file that needs to drive a client against a scripted daemon.

use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use spnav_rs::protocol::{
    ReqResp, MAX_PROTO_VER, REQ_TAG, UEV_CFG, UEV_DEV, UEV_MOTION, UEV_PRESS, UEV_RAWAXIS,
    UEV_RAWBUTTON, UEV_RELEASE,
};

// ─── Frame encoders ─────────────────────────────────────────────────────────
//
// These replace the per-file encoders that used to live in tests/protocol.rs.
// They build raw 32-byte daemon→client frames suitable for queueing on the
// simulator.

pub fn encode_motion_frame(
    x: i32,
    y: i32,
    z: i32,
    rx: i32,
    ry: i32,
    rz: i32,
    period: i32,
) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_MOTION.to_ne_bytes());
    buf[4..8].copy_from_slice(&x.to_ne_bytes());
    buf[8..12].copy_from_slice(&y.to_ne_bytes());
    buf[12..16].copy_from_slice(&z.to_ne_bytes());
    buf[16..20].copy_from_slice(&rx.to_ne_bytes());
    buf[20..24].copy_from_slice(&ry.to_ne_bytes());
    buf[24..28].copy_from_slice(&rz.to_ne_bytes());
    buf[28..32].copy_from_slice(&period.to_ne_bytes());
    buf
}

pub fn encode_button_frame(press: bool, bnum: i32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let evt_type = if press { UEV_PRESS } else { UEV_RELEASE };
    buf[0..4].copy_from_slice(&evt_type.to_ne_bytes());
    buf[4..8].copy_from_slice(&bnum.to_ne_bytes());
    buf
}

pub fn encode_dev_frame(op: i32, id: i32, devtype: i32, vendor: i32, product: i32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_DEV.to_ne_bytes());
    buf[4..8].copy_from_slice(&op.to_ne_bytes());
    buf[8..12].copy_from_slice(&id.to_ne_bytes());
    buf[12..16].copy_from_slice(&devtype.to_ne_bytes());
    buf[16..20].copy_from_slice(&vendor.to_ne_bytes());
    buf[20..24].copy_from_slice(&product.to_ne_bytes());
    buf
}

pub fn encode_cfg_frame(cfg: i32, data: [i32; 6]) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_CFG.to_ne_bytes());
    buf[4..8].copy_from_slice(&cfg.to_ne_bytes());
    for i in 0..6 {
        let off = 8 + i * 4;
        buf[off..off + 4].copy_from_slice(&data[i].to_ne_bytes());
    }
    buf
}

pub fn encode_rawaxis_frame(idx: i32, value: i32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_RAWAXIS.to_ne_bytes());
    buf[4..8].copy_from_slice(&idx.to_ne_bytes());
    buf[8..12].copy_from_slice(&value.to_ne_bytes());
    buf
}

pub fn encode_rawbutton_frame(bnum: i32, press: bool) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_RAWBUTTON.to_ne_bytes());
    buf[4..8].copy_from_slice(&bnum.to_ne_bytes());
    buf[8..12].copy_from_slice(&(press as i32).to_ne_bytes());
    buf
}

/// Build a handshake response advertising the given protocol version.
/// The real daemon responds with a single i32 (4 bytes), not a full 32-byte frame.
fn encode_handshake_response(proto_version: i32) -> [u8; 4] {
    let val = (REQ_TAG | proto_version) as u32;
    val.to_ne_bytes()
}

// ─── SpnavDaemonSimulator ───────────────────────────────────────────────────
//
// Binds a UNIX socket in a tempdir, accepts one connection, performs the
// spacenavd handshake, then runs full-duplex: frames queued via `send()` are
// written to the client; frames received from the client are buffered and
// retrievable via `recv()`.

pub struct SpnavDaemonSimulator {
    path: PathBuf,
    send_tx: mpsc::UnboundedSender<[u8; 32]>,
    recv_rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<[u8; 32]>>,
    _handle: tokio::task::JoinHandle<()>,
    _tmpdir: tempfile::TempDir,
}

impl SpnavDaemonSimulator {
    /// Bind a temp socket and spawn the background daemon task. The returned
    /// simulator is ready to accept one client connection.
    pub async fn start() -> Self {
        let tmpdir = tempfile::TempDir::new().expect("create tmpdir for simulator");
        let path = tmpdir.path().join("spnav.sock");
        let listener = UnixListener::bind(&path).expect("bind simulator socket");

        let (send_tx, send_rx) = mpsc::unbounded_channel::<[u8; 32]>();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel::<[u8; 32]>();

        let handle = tokio::spawn(run_daemon(listener, send_rx, recv_tx));

        Self {
            path,
            send_tx,
            recv_rx: tokio::sync::Mutex::new(recv_rx),
            _handle: handle,
            _tmpdir: tmpdir,
        }
    }

    /// Socket path to pass to `SpnavClient::open_path`.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Queue a 32-byte frame to be written to the client. Use for events or
    /// responses.
    pub fn send(&self, frame: [u8; 32]) {
        self.send_tx.send(frame).expect("daemon task alive");
    }

    /// Receive the next frame sent by the client, blocking until one arrives
    /// or the channel closes. Returns None if the daemon shut down first.
    pub async fn recv(&self) -> Option<[u8; 32]> {
        let mut guard = self.recv_rx.lock().await;
        guard.recv().await
    }

    /// Drain all currently-buffered frames received from the client, without
    /// waiting for more.
    pub async fn try_recv_all(&self) -> Vec<[u8; 32]> {
        let mut guard = self.recv_rx.lock().await;
        let mut out = Vec::new();
        while let Ok(frame) = guard.try_recv() {
            out.push(frame);
        }
        out
    }
}

async fn run_daemon(
    listener: UnixListener,
    mut send_rx: mpsc::UnboundedReceiver<[u8; 32]>,
    recv_tx: mpsc::UnboundedSender<[u8; 32]>,
) {
    let (stream, _) = match listener.accept().await {
        Ok(c) => c,
        Err(_) => return,
    };
    let (mut read_half, mut write_half) = stream.into_split();

    // Handshake: client sends 4 bytes (REQ_TAG | CHANGE_PROTO | MAX_PROTO_VER),
    // we respond with a 32-byte frame whose type_ carries the negotiated version.
    let mut cmd_buf = [0u8; 4];
    if read_half.read_exact(&mut cmd_buf).await.is_err() {
        return;
    }
    let response = encode_handshake_response(MAX_PROTO_VER);
    if write_half.write_all(&response).await.is_err() {
        return;
    }

    // Split into independent reader + writer tasks so neither direction blocks
    // the other.
    let reader = tokio::spawn(async move {
        let mut buf = [0u8; 32];
        loop {
            match read_half.read_exact(&mut buf).await {
                Ok(_) => {
                    if recv_tx.send(buf).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let writer = tokio::spawn(async move {
        while let Some(frame) = send_rx.recv().await {
            if write_half.write_all(&frame).await.is_err() {
                break;
            }
        }
    });

    let _ = reader.await;
    let _ = writer.await;
}
