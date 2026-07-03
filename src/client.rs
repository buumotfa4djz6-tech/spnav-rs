//! High-level async client for communicating with spacenavd.
//!
//! This module provides [`SpnavClient`], the primary interface for connecting to the
//! spacenavd daemon and receiving 6DOF input events. Most users should start here
//! rather than using the lower-level [`connection`] or
//! [`protocol`] modules directly.
//!
//! # Architecture
//!
//! [`SpnavClient`] wraps a UNIX socket connection and spawns a background event loop
//! that continuously reads from the daemon. Events are broadcast to all subscribers
//! via a tokio broadcast channel, allowing multiple consumers to receive the same
//! event stream.
//!
//! The client supports two modes of operation:
//!
//! - **Polling**: Call [`SpnavClient::poll_event()`] for non-blocking event retrieval.
//! - **Async waiting**: Call [`SpnavClient::wait_event()`] to asynchronously wait for
//!   the next event.
//! - **Stream API**: Call [`SpnavClient::subscribe()`] to get a `Stream` of events.
//!
//! # Protocol Versions
//!
//! The client negotiates the highest supported protocol version during connection.
//! Protocol v1 (supported by spacenavd 1.0+) enables additional features like
//! per-client sensitivity, event masks, device queries, and the configuration API.
//! Protocol v0 is a fallback for older daemons.
//!
//! # Examples
//!
//! ```no_run
//! use spnav_rs::SpnavClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = SpnavClient::open().await?;
//!     loop {
//!         let event = client.wait_event().await?;
//!         println!("Event: {:?}", event);
//!     }
//! }
//! ```

use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::connection;
use crate::error::{Error, Result};
use crate::event::{DeviceType, EventType, SpnavEvent};
use crate::evmask::EventMask;
use crate::protocol::{self, req, ReqResp};

/// The main client handle for communicating with spacenavd.
pub struct SpnavClient {
    req_tx: mpsc::Sender<InternalRequest>,
    event_rx: broadcast::Receiver<SpnavEvent>,
    proto: u8,
    /// Socket fd for fd() method
    fd: RawFd,
    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,
}

#[derive(Debug)]
enum InternalRequest {
    Simple {
        data: [u8; 32],
        reply: oneshot::Sender<Result<ReqResp>>,
    },
    String {
        chunks: Vec<[u8; 32]>,
        reply: oneshot::Sender<Result<String>>,
    },
    RecvString {
        req_type: i32,
        reply: oneshot::Sender<Result<String>>,
    },
}

impl SpnavClient {
    /// Connect to spacenavd via UNIX domain socket.
    /// Tries: SPNAV_SOCKET env var > /etc/spnavrc > /var/run/spnav.sock
    pub async fn open() -> Result<Self> {
        let path = connection::find_socket();
        Self::open_path(&path).await
    }

    /// Connect to a specific socket path.
    pub async fn open_path(path: &Path) -> Result<Self> {
        let mut stream = connection::connect(path).await?;
        let proto = connection::handshake(&mut stream).await?;
        Self::from_stream(stream, proto).await
    }

    /// Create a client from an already-handshaked stream.
    async fn from_stream(stream: UnixStream, proto: u8) -> Result<Self> {
        let fd = stream.as_raw_fd();
        let (req_tx, req_rx) = mpsc::channel::<InternalRequest>(32);
        let (event_tx, event_rx) = broadcast::channel::<SpnavEvent>(64);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let event_loop = EventLoopState {
            stream,
            req_rx,
            event_tx,
            shutdown_rx: Some(shutdown_rx),
        };

        tokio::spawn(event_loop.run());

        Ok(Self {
            req_tx,
            event_rx,
            proto,
            fd,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Close the connection to spacenavd.
    /// This is also called automatically when the client is dropped.
    pub fn close(&mut self) {
        // Send shutdown signal to event loop
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Get the file descriptor of the underlying socket.
    /// Useful for integration with select/poll/epoll in synchronous contexts.
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    /// Check if the connection is still alive.
    pub fn is_connected(&self) -> bool {
        !self.req_tx.is_closed()
    }

    /// Remove pending events of the specified type from the queue.
    /// Returns the number of events removed.
    ///
    /// # Known limitation
    ///
    /// This implementation uses a `tokio::sync::broadcast` channel, which does not
    /// support re-inserting events after inspection. Non-matching events drained
    /// from the receiver are **lost** — they will not be delivered to subsequent
    /// [`wait_event()`]/[`poll_event()`] calls.
    ///
    /// This differs from libspnav's `spnav_remove_events()`, which uses an internal
    /// linked-list queue to preserve non-matching events.
    ///
    /// For example, if the pending queue is `[Motion, Button, Motion, Motion]` and
    /// `remove_events(Motion)` is called:
    /// - libspnav result: queue becomes `[Button]`
    /// - This implementation: queue becomes `[]` (Button is also lost)
    ///
    /// This is usually not a problem for real-time consumers, but may cause issues
    /// under high event rates with slow consumers.
    ///
    /// TODO: Replace broadcast channel with a peekable queue to match libspnav behavior.
    ///
    /// [`wait_event()`]: Self::wait_event
    /// [`poll_event()`]: Self::poll_event
    pub fn remove_events(&mut self, event_type: EventType) -> usize {
        let mut removed = 0;
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => {
                    if event.event_type() == event_type {
                        removed += 1;
                    }
                    // Non-matching events are lost — see doc comment above.
                }
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                Err(broadcast::error::TryRecvError::Closed) => break,
            }
        }
        removed
    }

    /// Remove all pending events from the queue.
    /// Returns the number of events removed.
    pub fn remove_all_events(&mut self) -> usize {
        let mut removed = 0;
        loop {
            match self.event_rx.try_recv() {
                Ok(_) => removed += 1,
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                Err(broadcast::error::TryRecvError::Closed) => break,
            }
        }
        removed
    }

    /// Get the negotiated protocol version (0 or 1).
    pub fn protocol_version(&self) -> u8 {
        self.proto
    }

    /// Return a stream of events. Each call creates a new subscriber.
    pub fn subscribe(&self) -> impl futures_core::Stream<Item = SpnavEvent> {
        let rx = self.event_rx.resubscribe();
        EventStream { rx }
    }

    /// Poll for the next event (non-blocking).
    /// Returns None if no event is immediately available.
    pub fn poll_event(&mut self) -> Option<SpnavEvent> {
        match self.event_rx.try_recv() {
            Ok(event) => Some(event),
            Err(broadcast::error::TryRecvError::Empty) => None,
            Err(broadcast::error::TryRecvError::Lagged(_)) => self.event_rx.try_recv().ok(),
            Err(broadcast::error::TryRecvError::Closed) => None,
        }
    }

    /// Wait for the next event (async, blocking).
    pub async fn wait_event(&mut self) -> Result<SpnavEvent> {
        let event = self.event_rx.recv().await.map_err(|e| match e {
            broadcast::error::RecvError::Closed => Error::Protocol("event channel closed".into()),
            broadcast::error::RecvError::Lagged(n) => {
                Error::Protocol(format!("lagged {} events", n))
            }
        })?;
        Ok(event)
    }

    /// Set per-client sensitivity (protocol v1 only).
    pub async fn set_sensitivity(&self, sens: f32) -> Result<()> {
        if self.proto == 0 {
            // Protocol v0: raw float write (handled differently, not supported via request)
            return Err(Error::Protocol(
                "sensitivity not supported on protocol v0".into(),
            ));
        }
        let mut data = [0i32; 7];
        data[0] = f32_to_i32(sens);
        self.send_request(req::SET_SENS, data).await?;
        Ok(())
    }

    /// Set client name (protocol v1 only).
    pub async fn set_client_name(&self, name: &str) -> Result<()> {
        self.send_string(req::SET_NAME, name).await?;
        Ok(())
    }

    /// Set event mask (protocol v1 only).
    pub async fn set_event_mask(&self, mask: EventMask) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = mask.bits() as i32;
        self.send_request(req::SET_EVMASK, data).await?;
        Ok(())
    }

    /// Get event mask (protocol v1 only).
    pub async fn get_event_mask(&self) -> Result<EventMask> {
        let rr = self.send_request(req::GET_EVMASK, [0; 7]).await?;
        Ok(EventMask::from_bits_truncate(rr.data[0] as u16))
    }

    // --- Device queries (protocol v1) ---

    /// Get the device name (protocol v1 only).
    pub async fn dev_name(&self) -> Result<String> {
        self.recv_string(req::DEV_NAME).await
    }

    /// Get the device path (protocol v1 only).
    pub async fn dev_path(&self) -> Result<String> {
        self.recv_string(req::DEV_PATH).await
    }

    /// Get the number of buttons on the device (protocol v1 only).
    pub async fn dev_buttons(&self) -> Result<u32> {
        let rr = self.send_request(req::DEV_NBUTTONS, [0; 7]).await?;
        Ok(rr.data[0] as u32)
    }

    /// Get the number of axes on the device (protocol v1 only).
    pub async fn dev_axes(&self) -> Result<u32> {
        let rr = self.send_request(req::DEV_NAXES, [0; 7]).await?;
        Ok(rr.data[0] as u32)
    }

    /// Get the USB vendor and product IDs (protocol v1 only).
    ///
    /// Returns `(vendor_id, product_id)`. Returns `(0, 0)` for serial devices.
    pub async fn dev_usbid(&self) -> Result<(u32, u32)> {
        let rr = self.send_request(req::DEV_USBID, [0; 7]).await?;
        Ok((rr.data[0] as u32, rr.data[1] as u32))
    }

    /// Get the device type (protocol v1 only).
    ///
    /// Returns a [`DeviceType`] enum variant identifying the specific device model.
    pub async fn dev_type(&self) -> Result<DeviceType> {
        let rr = self.send_request(req::DEV_TYPE, [0; 7]).await?;
        Ok(DeviceType::from_raw(rr.data[0]))
    }

    // --- Configuration API (protocol v1) ---

    /// Reset all configuration to defaults (protocol v1 only).
    pub async fn cfg_reset(&self) -> Result<()> {
        self.send_request(req::CFG_RESET, [0; 7]).await?;
        Ok(())
    }

    /// Restore configuration from persistent storage (protocol v1 only).
    pub async fn cfg_restore(&self) -> Result<()> {
        self.send_request(req::CFG_RESTORE, [0; 7]).await?;
        Ok(())
    }

    /// Save current configuration to persistent storage (protocol v1 only).
    pub async fn cfg_save(&self) -> Result<()> {
        self.send_request(req::CFG_SAVE, [0; 7]).await?;
        Ok(())
    }

    /// Set global sensitivity (protocol v1 only).
    pub async fn cfg_set_sens(&self, s: f32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = f32_to_i32(s);
        self.send_request(req::SCFG_SENS, data).await?;
        Ok(())
    }

    /// Get global sensitivity (protocol v1 only).
    pub async fn cfg_get_sens(&self) -> Result<f32> {
        let rr = self.send_request(req::GCFG_SENS, [0; 7]).await?;
        Ok(i32_to_f32(rr.data[0]))
    }

    /// Set per-axis sensitivity (protocol v1 only).
    ///
    /// The array contains sensitivity values for each of the 6 axes
    /// (tx, ty, tz, rx, ry, rz).
    pub async fn cfg_set_axis_sens(&self, svec: [f32; 6]) -> Result<()> {
        let mut data = [0i32; 7];
        for i in 0..6 {
            data[i] = f32_to_i32(svec[i]);
        }
        self.send_request(req::SCFG_SENS_AXIS, data).await?;
        Ok(())
    }

    /// Get per-axis sensitivity (protocol v1 only).
    ///
    /// Returns an array of sensitivity values for each of the 6 axes.
    pub async fn cfg_get_axis_sens(&self) -> Result<[f32; 6]> {
        let rr = self.send_request(req::GCFG_SENS_AXIS, [0; 7]).await?;
        let mut svec = [0.0f32; 6];
        for (i, item) in svec.iter_mut().enumerate() {
            *item = i32_to_f32(rr.data[i]);
        }
        Ok(svec)
    }

    /// Set deadzone for a device axis (protocol v1 only).
    ///
    /// `devaxis` is the device axis index (0-5), `delta` is the deadzone width.
    pub async fn cfg_set_deadzone(&self, devaxis: i32, delta: i32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = devaxis;
        data[1] = delta;
        self.send_request(req::SCFG_DEADZONE, data).await?;
        Ok(())
    }

    /// Get deadzone for a device axis (protocol v1 only).
    pub async fn cfg_get_deadzone(&self, devaxis: i32) -> Result<i32> {
        let mut data = [0i32; 7];
        data[0] = devaxis;
        let rr = self.send_request(req::GCFG_DEADZONE, data).await?;
        Ok(rr.data[1])
    }

    /// Set axis inversion bits (protocol v1 only).
    ///
    /// `invbits` is a bitmask where bit 0 inverts axis 0, bit 1 inverts axis 1, etc.
    pub async fn cfg_set_invert(&self, invbits: i32) -> Result<()> {
        let mut data = [0i32; 7];
        for (i, item) in data.iter_mut().enumerate().take(6) {
            *item = (invbits >> i) & 1;
        }
        self.send_request(req::SCFG_INVERT, data).await?;
        Ok(())
    }

    /// Get axis inversion bits (protocol v1 only).
    pub async fn cfg_get_invert(&self) -> Result<i32> {
        let rr = self.send_request(req::GCFG_INVERT, [0; 7]).await?;
        let mut res = 0i32;
        for i in 0..6 {
            res = (res >> 1) | (if rr.data[i] != 0 { 0x20 } else { 0 });
        }
        Ok(res)
    }

    /// Set axis mapping for a device axis (protocol v1 only).
    ///
    /// `devaxis` is the source device axis, `map` is the target axis.
    pub async fn cfg_set_axismap(&self, devaxis: i32, map: i32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = devaxis;
        data[1] = map;
        self.send_request(req::SCFG_AXISMAP, data).await?;
        Ok(())
    }

    /// Get axis mapping for a device axis (protocol v1 only).
    pub async fn cfg_get_axismap(&self, devaxis: i32) -> Result<i32> {
        let mut data = [0i32; 7];
        data[0] = devaxis;
        let rr = self.send_request(req::GCFG_AXISMAP, data).await?;
        Ok(rr.data[1])
    }

    /// Set button mapping (protocol v1 only).
    ///
    /// `devbn` is the source button number, `map` is the target button number.
    pub async fn cfg_set_bnmap(&self, devbn: i32, map: i32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = devbn;
        data[1] = map;
        self.send_request(req::SCFG_BNMAP, data).await?;
        Ok(())
    }

    /// Get button mapping (protocol v1 only).
    pub async fn cfg_get_bnmap(&self, devbn: i32) -> Result<i32> {
        let mut data = [0i32; 7];
        data[0] = devbn;
        let rr = self.send_request(req::GCFG_BNMAP, data).await?;
        Ok(rr.data[1])
    }

    /// Set button action (protocol v1 only).
    ///
    /// `bn` is the button number, `act` is the action code.
    pub async fn cfg_set_bnaction(&self, bn: i32, act: i32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = bn;
        data[1] = act;
        self.send_request(req::SCFG_BNACTION, data).await?;
        Ok(())
    }

    /// Get button action (protocol v1 only).
    pub async fn cfg_get_bnaction(&self, bn: i32) -> Result<i32> {
        let mut data = [0i32; 7];
        data[0] = bn;
        let rr = self.send_request(req::GCFG_BNACTION, data).await?;
        Ok(rr.data[1])
    }

    /// Set keyboard mapping for a button (protocol v1 only).
    ///
    /// `bn` is the button number, `key` is the X11 keysym.
    pub async fn cfg_set_kbmap(&self, bn: i32, key: i32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = bn;
        data[1] = key;
        self.send_request(req::SCFG_KBMAP, data).await?;
        Ok(())
    }

    /// Get keyboard mapping for a button (protocol v1 only).
    pub async fn cfg_get_kbmap(&self, bn: i32) -> Result<i32> {
        let mut data = [0i32; 7];
        data[0] = bn;
        let rr = self.send_request(req::GCFG_KBMAP, data).await?;
        Ok(rr.data[1])
    }

    /// Enable or disable Y/Z axis swap (protocol v1 only).
    pub async fn cfg_set_swapyz(&self, swap: bool) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = if swap { 1 } else { 0 };
        self.send_request(req::SCFG_SWAPYZ, data).await?;
        Ok(())
    }

    /// Get Y/Z axis swap state (protocol v1 only).
    pub async fn cfg_get_swapyz(&self) -> Result<bool> {
        let rr = self.send_request(req::GCFG_SWAPYZ, [0; 7]).await?;
        Ok(rr.data[0] != 0)
    }

    /// Set LED state (protocol v1 only).
    ///
    /// See [`LedState`] for available states.
    pub async fn cfg_set_led(&self, state: LedState) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = state as i32;
        self.send_request(req::SCFG_LED, data).await?;
        Ok(())
    }

    /// Get LED state (protocol v1 only).
    pub async fn cfg_get_led(&self) -> Result<LedState> {
        let rr = self.send_request(req::GCFG_LED, [0; 7]).await?;
        match rr.data[0] {
            0 => Ok(LedState::Off),
            1 => Ok(LedState::On),
            2 => Ok(LedState::Auto),
            _ => Err(Error::Protocol(format!(
                "invalid LED state: {}",
                rr.data[0]
            ))),
        }
    }

    /// Enable or disable device grabbing (protocol v1 only).
    ///
    /// When enabled, spacenavd exclusively grabs the device, preventing other
    /// applications from receiving raw input.
    pub async fn cfg_set_grab(&self, state: bool) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = if state { 1 } else { 0 };
        self.send_request(req::SCFG_GRAB, data).await?;
        Ok(())
    }

    /// Get device grabbing state (protocol v1 only).
    pub async fn cfg_get_grab(&self) -> Result<bool> {
        let rr = self.send_request(req::GCFG_GRAB, [0; 7]).await?;
        Ok(rr.data[0] != 0)
    }

    /// Set serial device path (protocol v1 only).
    ///
    /// Used for serial-connected devices to specify the serial port.
    pub async fn cfg_set_serial(&self, devpath: &str) -> Result<()> {
        self.send_string(req::SCFG_SERDEV, devpath).await?;
        Ok(())
    }

    /// Get serial device path (protocol v1 only).
    pub async fn cfg_get_serial(&self) -> Result<String> {
        self.recv_string(req::GCFG_SERDEV).await
    }

    /// Set button repeat interval in milliseconds (protocol v1 only).
    ///
    /// Set to -1 to disable button repeat.
    pub async fn cfg_set_repeat(&self, msec: i32) -> Result<()> {
        let mut data = [0i32; 7];
        data[0] = if msec < 0 { -1 } else { msec };
        self.send_request(req::SCFG_REPEAT, data).await?;
        Ok(())
    }

    /// Get button repeat interval in milliseconds (protocol v1 only).
    ///
    /// Returns -1 if button repeat is disabled.
    pub async fn cfg_get_repeat(&self) -> Result<i32> {
        let rr = self.send_request(req::GCFG_REPEAT, [0; 7]).await?;
        Ok(rr.data[0])
    }

    // --- Internal helpers ---

    /// Send a simple request and await response.
    async fn send_request(&self, req_type: i32, data: [i32; 7]) -> Result<ReqResp> {
        if self.proto < 1 {
            return Err(Error::Protocol(
                "request not supported on protocol v0".into(),
            ));
        }

        let rr = protocol::make_request(req_type, data);
        let bytes = rr.to_bytes();

        let (tx, rx) = oneshot::channel();
        self.req_tx
            .send(InternalRequest::Simple {
                data: bytes,
                reply: tx,
            })
            .await
            .map_err(|_| Error::Protocol("event loop stopped".into()))?;

        rx.await
            .map_err(|_| Error::Protocol("request cancelled".into()))?
    }

    /// Send a string request (chunked) and await string response.
    async fn send_string(&self, req_type: i32, s: &str) -> Result<()> {
        if self.proto < 1 {
            return Err(Error::Protocol(
                "string request not supported on protocol v0".into(),
            ));
        }

        let chunks: Vec<[u8; 32]> = protocol::encode_string_chunks(req_type, s)
            .into_iter()
            .map(|rr| rr.to_bytes())
            .collect();

        let (tx, rx) = oneshot::channel();
        self.req_tx
            .send(InternalRequest::String { chunks, reply: tx })
            .await
            .map_err(|_| Error::Protocol("event loop stopped".into()))?;

        // String set requests just return a status response, ignore the actual string
        let _ = rx
            .await
            .map_err(|_| Error::Protocol("request cancelled".into()))?;
        Ok(())
    }

    /// Receive a string response.
    async fn recv_string(&self, req_type: i32) -> Result<String> {
        if self.proto < 1 {
            return Err(Error::Protocol(
                "string response not supported on protocol v0".into(),
            ));
        }
        let (tx, rx) = oneshot::channel();
        self.req_tx
            .send(InternalRequest::RecvString {
                req_type,
                reply: tx,
            })
            .await
            .map_err(|_| Error::Protocol("event loop stopped".into()))?;
        rx.await
            .map_err(|_| Error::Protocol("request cancelled".into()))?
    }
}

/// LED state for the device.
///
/// Controls the LED indicator on spacenavd-compatible devices.
/// Use with [`SpnavClient::cfg_set_led()`] and [`SpnavClient::cfg_get_led()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedState {
    /// LED is off.
    Off = 0,
    /// LED is on.
    On = 1,
    /// LED is controlled automatically by spacenavd.
    Auto = 2,
}

struct EventLoopState {
    stream: UnixStream,
    req_rx: mpsc::Receiver<InternalRequest>,
    event_tx: broadcast::Sender<SpnavEvent>,
    shutdown_rx: Option<oneshot::Receiver<()>>,
}

impl EventLoopState {
    async fn run(mut self) {
        let (mut read_half, mut write_half) = self.stream.into_split();
        let mut frame_buf = [0u8; 32];
        let mut shutdown_rx = self.shutdown_rx.take().unwrap();

        loop {
            tokio::select! {
                // Shutdown signal
                _ = &mut shutdown_rx => {
                    break;
                }
                // Read incoming frame from daemon
                result = read_half.read_exact(&mut frame_buf) => {
                    match result {
                        Ok(_) => {
                            let rr = ReqResp::from_bytes(&frame_buf);

                            if rr.is_response() {
                                // This is a response - will be handled by the request handler below
                                // For now, discard (events loop handles responses inline)
                            } else if let Some(event) = protocol::decode_event(&frame_buf) {
                                let _ = self.event_tx.send(event);
                            }
                        }
                        Err(_) => {
                            // Connection closed
                            break;
                        }
                    }
                }
                // Outgoing request from client
                Some(msg) = self.req_rx.recv() => {
                    match msg {
                        InternalRequest::Simple { data, reply } => {
                            if write_half.write_all(&data).await.is_err() {
                                let _ = reply.send(Err(Error::Io(std::io::Error::new(
                                    std::io::ErrorKind::BrokenPipe,
                                    "connection closed",
                                ))));
                                continue;
                            }

                            // Read response frames until we get a response
                            let result = Self::read_response(&mut read_half, &self.event_tx).await;
                            let _ = reply.send(result);
                        }
                        InternalRequest::String { chunks, reply } => {
                            // Send all chunks
                            let mut send_ok = true;
                            for chunk in &chunks {
                                if write_half.write_all(chunk).await.is_err() {
                                    send_ok = false;
                                    break;
                                }
                            }

                            if !send_ok {
                                let _ = reply.send(Err(Error::Io(std::io::Error::new(
                                    std::io::ErrorKind::BrokenPipe,
                                    "connection closed",
                                ))));
                                continue;
                            }

                            // Read string response frames
                            let mut decoder = protocol::StringDecoder::new();
                            loop {
                                let rr = match Self::read_response_or_event(&mut read_half, &self.event_tx).await {
                                    Some(rr) => rr,
                                    None => {
                                        let _ = reply.send(Err(Error::Io(std::io::Error::new(
                                            std::io::ErrorKind::BrokenPipe,
                                            "connection closed",
                                        ))));
                                        break;
                                    }
                                };

                                if rr.data[6] < 0 {
                                    let _ = reply.send(Err(Error::DaemonFailure));
                                    break;
                                }

                                match decoder.feed(&rr) {
                                    Ok(Some(s)) => {
                                        let _ = reply.send(Ok(s));
                                        break;
                                    }
                                    Ok(None) => {
                                        // More chunks expected
                                    }
                                    Err(e) => {
                                        let _ = reply.send(Err(e));
                                        break;
                                    }
                                }
                            }
                        }
                        InternalRequest::RecvString { req_type, reply } => {
                            // Send the request
                            let rr = protocol::make_request(req_type, [0; 7]);
                            let data = rr.to_bytes();
                            if write_half.write_all(&data).await.is_err() {
                                let _ = reply.send(Err(Error::Io(std::io::Error::new(
                                    std::io::ErrorKind::BrokenPipe,
                                    "connection closed",
                                ))));
                                continue;
                            }

                            // Read string response frames
                            let mut decoder = protocol::StringDecoder::new();
                            loop {
                                let rr = match Self::read_response_or_event(&mut read_half, &self.event_tx).await {
                                    Some(rr) => rr,
                                    None => {
                                        let _ = reply.send(Err(Error::Io(std::io::Error::new(
                                            std::io::ErrorKind::BrokenPipe,
                                            "connection closed",
                                        ))));
                                        break;
                                    }
                                };

                                if rr.data[6] < 0 {
                                    let _ = reply.send(Err(Error::DaemonFailure));
                                    break;
                                }

                                match decoder.feed(&rr) {
                                    Ok(Some(s)) => {
                                        let _ = reply.send(Ok(s));
                                        break;
                                    }
                                    Ok(None) => {
                                        // More chunks expected
                                    }
                                    Err(e) => {
                                        let _ = reply.send(Err(e));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Read frames until we get a response, forwarding events along the way.
    async fn read_response(
        read_half: &mut tokio::net::unix::OwnedReadHalf,
        event_tx: &broadcast::Sender<SpnavEvent>,
    ) -> Result<ReqResp> {
        let mut buf = [0u8; 32];
        loop {
            match read_half.read_exact(&mut buf).await {
                Ok(_) => {
                    let rr = ReqResp::from_bytes(&buf);
                    if rr.is_response() {
                        if rr.status_ok() {
                            return Ok(rr);
                        } else {
                            return Err(Error::DaemonFailure);
                        }
                    } else if let Some(event) = protocol::decode_event(&buf) {
                        let _ = event_tx.send(event);
                    }
                }
                Err(_) => {
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "connection closed",
                    )));
                }
            }
        }
    }

    /// Read frames until we get a response, returning the response frame (not checking status).
    async fn read_response_or_event(
        read_half: &mut tokio::net::unix::OwnedReadHalf,
        event_tx: &broadcast::Sender<SpnavEvent>,
    ) -> Option<ReqResp> {
        let mut buf = [0u8; 32];
        loop {
            match read_half.read_exact(&mut buf).await {
                Ok(_) => {
                    let rr = ReqResp::from_bytes(&buf);
                    if rr.is_response() {
                        return Some(rr);
                    } else if let Some(event) = protocol::decode_event(&buf) {
                        let _ = event_tx.send(event);
                    }
                }
                Err(_) => return None,
            }
        }
    }
}

impl Drop for SpnavClient {
    fn drop(&mut self) {
        self.close();
    }
}

/// A simple Stream impl over a broadcast receiver.
struct EventStream {
    rx: broadcast::Receiver<SpnavEvent>,
}

impl futures_core::Stream for EventStream {
    type Item = SpnavEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.rx.try_recv() {
            Ok(event) => std::task::Poll::Ready(Some(event)),
            Err(broadcast::error::TryRecvError::Empty) => {
                // Need to wait for new events. We use a simple approach:
                // spawn a task that waits for an event and then wakes the waker.
                let mut rx = self.rx.resubscribe();
                let waker = cx.waker().clone();
                tokio::spawn(async move {
                    let _ = rx.recv().await;
                    waker.wake();
                });
                std::task::Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Lagged(_)) => {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => std::task::Poll::Ready(None),
        }
    }
}

fn f32_to_i32(f: f32) -> i32 {
    f.to_bits() as i32
}

fn i32_to_f32(i: i32) -> f32 {
    f32::from_bits(i as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_roundtrip() {
        let f = 1.5f32;
        assert_eq!(i32_to_f32(f32_to_i32(f)), f);
    }
}
