//! spnav-rs: Async Rust client library for 6DOF input devices (spacenav/spacenavd).
//!
//! This library communicates with the spacenavd daemon via UNIX domain sockets,
//! providing an async-first API for receiving motion, button, device, and config events.
//!
//! # Quick Start
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
//!
//! # Examples
//!
//! - `basic_viewer` - Print all events as they arrive
//! - `device_info` - Query device capabilities and configuration
//! - `config_tool` - Demonstrate the configuration API
//!
//! ```bash
//! cargo run --example basic_viewer
//! cargo run --example device_info
//! cargo run --example config_tool
//! ```
//!
//! # Features
//!
//! - `x11` - Enable X11 (Magellan protocol) support for compatibility with 3dxsrv

pub mod client;
pub mod connection;
pub mod error;
pub mod event;
pub mod evmask;
pub mod math;
pub mod protocol;

#[cfg(feature = "x11")]
pub mod x11;

pub use client::{LedState, SpnavClient};
pub use error::{Error, Result};
pub use event::{EventType, SpnavEvent};
pub use evmask::EventMask;
pub use math::PositionRot;

#[cfg(feature = "x11")]
pub use x11::X11Spnav;
