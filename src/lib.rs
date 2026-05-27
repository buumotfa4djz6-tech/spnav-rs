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

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use crate::event::{
        ButtonEvent, ConfigEvent, DeviceEvent, DeviceOp, DeviceType, EventType, MotionEvent,
        RawAxisEvent, RawButtonEvent, SpnavEvent,
    };
    use crate::evmask::EventMask;
    use crate::math::PositionRot;
    use glam::{Quat, Vec3};

    // --- Event type tests ---

    #[test]
    fn test_event_type_mapping() {
        let motion = SpnavEvent::Motion(MotionEvent {
            x: 0,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            period: 0,
        });
        assert_eq!(motion.event_type(), EventType::Motion);

        let btn = SpnavEvent::Button(ButtonEvent {
            press: true,
            bnum: 1,
        });
        assert_eq!(btn.event_type(), EventType::Button);

        let dev = SpnavEvent::Device(DeviceEvent {
            op: DeviceOp::Add,
            id: 0,
            devtype: DeviceType::Unknown,
            usb_vendor: 0,
            usb_product: 0,
        });
        assert_eq!(dev.event_type(), EventType::Device);

        let cfg = SpnavEvent::Config(ConfigEvent {
            cfg: 0,
            data: [0; 6],
        });
        assert_eq!(cfg.event_type(), EventType::Config);

        let axis = SpnavEvent::RawAxis(RawAxisEvent { idx: 0, value: 0 });
        assert_eq!(axis.event_type(), EventType::RawAxis);

        let rbtn = SpnavEvent::RawButton(RawButtonEvent {
            bnum: 0,
            press: false,
        });
        assert_eq!(rbtn.event_type(), EventType::RawButton);
    }

    #[test]
    fn test_motion_event_fields() {
        let ev = MotionEvent {
            x: 10,
            y: -20,
            z: 30,
            rx: 1,
            ry: -2,
            rz: 3,
            period: 16,
        };
        assert_eq!(ev.x, 10);
        assert_eq!(ev.period, 16);
    }

    #[test]
    fn test_button_press_and_release() {
        let press = SpnavEvent::Button(ButtonEvent {
            press: true,
            bnum: 0,
        });
        let release = SpnavEvent::Button(ButtonEvent {
            press: false,
            bnum: 0,
        });
        assert_ne!(press, release);
    }

    // --- Device type tests ---

    #[test]
    fn test_device_type_from_raw() {
        assert_eq!(DeviceType::from_raw(0), DeviceType::Unknown);
        assert_eq!(DeviceType::from_raw(0x100), DeviceType::Sb2003);
        assert_eq!(DeviceType::from_raw(0x200), DeviceType::PlusXt);
        assert_eq!(DeviceType::from_raw(0x210), DeviceType::SMMod);
        assert_eq!(DeviceType::from_raw(0x9999), DeviceType::Unknown);
    }

    #[test]
    fn test_device_event_add_remove() {
        let add = DeviceEvent {
            op: DeviceOp::Add,
            id: 1,
            devtype: DeviceType::SMPro,
            usb_vendor: 0x046d,
            usb_product: 0xc629,
        };
        assert_eq!(add.usb_vendor, 0x046d);
        assert_eq!(add.devtype, DeviceType::SMPro);
    }

    // --- Event mask tests ---

    #[test]
    fn test_event_mask_bits() {
        assert!(EventMask::DEFAULT.contains(EventMask::MOTION));
        assert!(EventMask::DEFAULT.contains(EventMask::BUTTON));
        assert!(EventMask::DEFAULT.contains(EventMask::DEV));
    }

    #[test]
    fn test_event_mask_all() {
        assert_eq!(EventMask::ALL, EventMask::from_bits_truncate(0xFFFF));
        assert!(EventMask::ALL.contains(EventMask::RAW_AXIS));
        assert!(EventMask::ALL.contains(EventMask::RAW_BUTTON));
    }

    #[test]
    fn test_event_mask_combination() {
        let mask = EventMask::MOTION | EventMask::BUTTON;
        assert!(mask.contains(EventMask::MOTION));
        assert!(mask.contains(EventMask::BUTTON));
        assert!(!mask.contains(EventMask::DEV));
    }

    #[test]
    fn test_event_mask_from_bits_truncate() {
        let mask = EventMask::from_bits_truncate(0x03);
        assert_eq!(mask, EventMask::MOTION | EventMask::BUTTON);
    }

    // --- Position/rotation tests ---

    #[test]
    fn test_position_rot_default() {
        let pr = PositionRot::default();
        assert_eq!(pr.pos, Vec3::ZERO);
        assert!(pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
    }

    #[test]
    fn test_move_obj_accumulates() {
        let mut pr = PositionRot::new();
        let ev1 = MotionEvent {
            x: 500,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            period: 0,
        };
        let ev2 = MotionEvent {
            x: 500,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            period: 0,
        };
        pr.move_obj(&ev1);
        pr.move_obj(&ev2);
        assert!((pr.pos.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_move_view_rotates_around_origin() {
        let mut pr = PositionRot::new();
        let ev = MotionEvent {
            x: 0,
            y: 0,
            z: 0,
            rx: 100,
            ry: 0,
            rz: 0,
            period: 0,
        };
        pr.move_view(&ev);
        assert_eq!(pr.pos, Vec3::ZERO);
        assert!(!pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
    }

    #[test]
    fn test_matrices_with_rotation_and_translation() {
        let mut pr = PositionRot::new();
        pr.pos = Vec3::new(1.0, 2.0, 3.0);
        pr.rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let model = pr.to_model_matrix();
        let view = pr.to_view_matrix();
        // Model matrix: rotation * translation (column-major, translation gets rotated)
        // View matrix: translation * rotation (translation stays un-rotated)
        // These should produce different translation components
        assert!(
            (model.w_axis.x - view.w_axis.x).abs() > 0.1
                || (model.w_axis.y - view.w_axis.y).abs() > 0.1
        );
    }

    // --- Error type tests ---

    #[test]
    fn test_error_display() {
        let err = Error::SocketNotFound;
        assert!(err.to_string().contains("spacenavd"));

        let err = Error::DaemonFailure;
        assert!(err.to_string().contains("failure"));

        let err = Error::NotOpen;
        assert!(err.to_string().contains("not open"));
    }

    #[test]
    fn test_result_alias() {
        fn returns_ok() -> Result<u32> {
            Ok(42)
        }
        fn returns_err() -> Result<u32> {
            Err(Error::Timeout)
        }
        assert!(returns_ok().is_ok());
        assert!(matches!(returns_err(), Err(Error::Timeout)));
    }
}
