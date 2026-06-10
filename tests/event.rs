//! Integration tests for event types.

use spnav_rs::event::*;
use spnav_rs::SpnavEvent;

// ─── SpnavEvent::event_type() dispatch ───────────────────────────────────────

#[test]
fn spnav_event_event_type_returns_correct_variant() {
    let cases = vec![
        (
            SpnavEvent::Motion(MotionEvent {
                x: 0,
                y: 0,
                z: 0,
                rx: 0,
                ry: 0,
                rz: 0,
                period: 0,
            }),
            EventType::Motion,
        ),
        (
            SpnavEvent::Button(ButtonEvent {
                press: false,
                bnum: 0,
            }),
            EventType::Button,
        ),
        (
            SpnavEvent::Device(DeviceEvent {
                op: DeviceOp::Add,
                id: 0,
                devtype: DeviceType::Unknown,
                usb_vendor: 0,
                usb_product: 0,
            }),
            EventType::Device,
        ),
        (
            SpnavEvent::Config(ConfigEvent {
                cfg: 0,
                data: [0; 6],
            }),
            EventType::Config,
        ),
        (
            SpnavEvent::RawAxis(RawAxisEvent { idx: 0, value: 0 }),
            EventType::RawAxis,
        ),
        (
            SpnavEvent::RawButton(RawButtonEvent {
                bnum: 0,
                press: false,
            }),
            EventType::RawButton,
        ),
    ];

    for (event, expected_type) in cases {
        assert_eq!(event.event_type(), expected_type);
    }
}

// ─── DeviceType::from_raw() mapping ─────────────────────────────────────────

#[test]
fn device_type_from_raw_unknown() {
    assert_eq!(DeviceType::from_raw(0), DeviceType::Unknown);
}

#[test]
fn device_type_from_raw_serial_devices() {
    assert_eq!(DeviceType::from_raw(0x100), DeviceType::Sb2003);
    assert_eq!(DeviceType::from_raw(0x101), DeviceType::Sb3003);
    assert_eq!(DeviceType::from_raw(0x102), DeviceType::Sb4000);
    assert_eq!(DeviceType::from_raw(0x103), DeviceType::Sm);
    assert_eq!(DeviceType::from_raw(0x104), DeviceType::Sm5000);
    assert_eq!(DeviceType::from_raw(0x105), DeviceType::SmCadman);
}

#[test]
fn device_type_from_raw_usb_devices() {
    assert_eq!(DeviceType::from_raw(0x200), DeviceType::PlusXt);
    assert_eq!(DeviceType::from_raw(0x201), DeviceType::Cadman);
    assert_eq!(DeviceType::from_raw(0x202), DeviceType::SmClassic);
    assert_eq!(DeviceType::from_raw(0x203), DeviceType::Sb5000);
    assert_eq!(DeviceType::from_raw(0x204), DeviceType::STraveller);
    assert_eq!(DeviceType::from_raw(0x205), DeviceType::SPilot);
    assert_eq!(DeviceType::from_raw(0x206), DeviceType::SNav);
    assert_eq!(DeviceType::from_raw(0x207), DeviceType::SExp);
    assert_eq!(DeviceType::from_raw(0x208), DeviceType::SNavNb);
    assert_eq!(DeviceType::from_raw(0x209), DeviceType::SPilotPro);
    assert_eq!(DeviceType::from_raw(0x20a), DeviceType::SMPro);
    assert_eq!(DeviceType::from_raw(0x20b), DeviceType::NuLooq);
    assert_eq!(DeviceType::from_raw(0x20c), DeviceType::Smw);
    assert_eq!(DeviceType::from_raw(0x20d), DeviceType::SMProW);
    assert_eq!(DeviceType::from_raw(0x20e), DeviceType::SMEnt);
    assert_eq!(DeviceType::from_raw(0x20f), DeviceType::SMComp);
    assert_eq!(DeviceType::from_raw(0x210), DeviceType::SMMod);
}

#[test]
fn device_type_from_raw_unrecognized_returns_unknown() {
    assert_eq!(DeviceType::from_raw(0x106), DeviceType::Unknown);
    assert_eq!(DeviceType::from_raw(0x211), DeviceType::Unknown);
    assert_eq!(DeviceType::from_raw(0xFFFF), DeviceType::Unknown);
    assert_eq!(DeviceType::from_raw(-1), DeviceType::Unknown);
    assert_eq!(DeviceType::from_raw(1), DeviceType::Unknown);
}
