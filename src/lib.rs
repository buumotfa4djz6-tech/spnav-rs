//! # spnav-rs
//!
//! Async Rust client library for 6DOF input devices (spacenav/spacenavd).
//!
//! This library communicates with the [spacenavd](http://spacenav.sourceforge.net/) daemon
//! via UNIX domain sockets, providing an async-first API for receiving motion, button,
//! device, and configuration events from 3Dconnexion devices.
//!
//! # Architecture
//!
//! The crate is organized into several layers:
//!
//! - [`client`] — High-level async API ([`SpnavClient`]) for connecting to the daemon
//!   and receiving events. Most users start here.
//! - [`connection`] — Low-level UNIX socket connection management.
//! - [`protocol`] — Wire protocol encoding/decoding for the spacenavd protocol.
//! - [`event`] — Event types ([`SpnavEvent`], [`EventType`]) representing motion,
//!   button presses, device changes, and configuration updates.
//! - [`evmask`] — Event filtering via [`EventMask`] bitmasks.
//! - [`math`] — Mathematical types like [`PositionRot`] for 6DOF data.
//! - [`error`] — Error types ([`Error`], [`Result`]).
//! - [`x11`] (optional) — X11 backend using the Magellan protocol for compatibility
//!   with 3dxsrv when spacenavd is unavailable.
//!
//! # Quick Start
//!
//! ```no_run
//! use spnav_rs::SpnavClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to spacenavd
//!     let mut client = SpnavClient::open().await?;
//!
//!     // Receive events in a loop
//!     loop {
//!         let event = client.wait_event().await?;
//!         println!("Event: {:?}", event);
//!     }
//! }
//! ```
//!
//! # Examples
//!
//! The `examples/` directory contains complete working examples:
//!
//! - [`basic_viewer`](https://github.com/FreeSpacenav/libspnav/tree/master/examples) — Print all events as they arrive
//! - `device_info` — Query device capabilities and configuration
//! - `config_tool` — Demonstrate the configuration API
//! - `x11_viewer` — X11-based event viewer (requires `x11` feature)
//! - `x11_cube` — 3D cube visualization using X11 (requires `x11` feature)
//!
//! Run examples with:
//!
//! ```bash
//! cargo run --example basic_viewer
//! cargo run --example device_info
//! cargo run --example config_tool
//! cargo run --example x11_viewer --features x11
//! ```
//!
//! # Feature Flags
//!
//! - **`x11`** — Enable X11 (Magellan protocol) support via [`x11::X11Spnav`].
//!   This provides compatibility with 3dxsrv when spacenavd is unavailable.
//!   Requires the `x11rb` crate.
//!
//! # Platform Requirements
//!
//! This library requires the **spacenavd** daemon to be running on your system.
//! Install it via your package manager:
//!
//! - **Debian/Ubuntu**: `sudo apt-get install spacenavd`
//! - **Fedora**: `sudo dnf install spacenavd`
//! - **Arch**: `sudo pacman -S spacenavd`
//!
//! Ensure the daemon is running before using this library:
//!
//! ```bash
//! sudo systemctl start spacenavd
//! ```
//!
//! # Links
//!
//! - [CHANGELOG](https://github.com/FreeSpacenav/libspnav/blob/master/CHANGELOG.md)
//! - [spacenavd project](http://spacenav.sourceforge.net/)
//! - [FreeSpacenav GitHub](https://github.com/FreeSpacenav/libspnav)

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod client;
pub mod connection;
pub mod error;
pub mod event;
pub mod evmask;
pub mod math;
pub mod protocol;

#[cfg(feature = "x11")]
#[cfg_attr(docsrs, doc(cfg(feature = "x11")))]
pub mod x11;

pub use client::{LedState, SpnavClient};
pub use error::{Error, Result};
pub use event::{EventType, SpnavEvent};
pub use evmask::EventMask;
pub use math::PositionRot;

#[cfg(feature = "x11")]
#[cfg_attr(docsrs, doc(cfg(feature = "x11")))]
pub use x11::X11Spnav;
