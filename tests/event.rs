//! Integration tests for event types.

use spnav_rs::event::*;
use spnav_rs::SpnavEvent;

// ─── EventType tests ────────────────────────────────────────────────────────

#[test]
fn event_type_variants_are_distinct() {
    assert_ne!(EventType::Motion, EventType::Button);
    assert_ne!(EventType::Motion, EventType::Device);
    assert_ne!(EventType::Motion, EventType::Config);
    assert_ne!(EventType::Motion, EventType::RawAxis);
    assert_ne!(EventType::Motion, EventType::RawButton);
    assert_ne!(EventType::Button, EventType::Device);
    assert_ne!(EventType::Button, EventType::Config);
    assert_ne!(EventType::Button, EventType::RawAxis);
    assert_ne!(EventType::Button, EventType::RawButton);
    assert_ne!(EventType::Device, EventType::Config);
    assert_ne!(EventType::Device, EventType::RawAxis);
    assert_ne!(EventType::Device, EventType::RawButton);
    assert_ne!(EventType::Config, EventType::RawAxis);
    assert_ne!(EventType::Config, EventType::RawButton);
    assert_ne!(EventType::RawAxis, EventType::RawButton);
}

#[test]
fn event_type_is_copy() {
    let t = EventType::Motion;
    let t2 = t;
    assert_eq!(t, t2);
}

#[test]
fn event_type_is_debug() {
    assert_eq!(format!("{:?}", EventType::Motion), "Motion");
    assert_eq!(format!("{:?}", EventType::Button), "Button");
    assert_eq!(format!("{:?}", EventType::Device), "Device");
    assert_eq!(format!("{:?}", EventType::Config), "Config");
    assert_eq!(format!("{:?}", EventType::RawAxis), "RawAxis");
    assert_eq!(format!("{:?}", EventType::RawButton), "RawButton");
}

// ─── SpnavEvent tests ───────────────────────────────────────────────────────

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

#[test]
fn spnav_event_is_clone() {
    let ev = SpnavEvent::Motion(MotionEvent {
        x: 1,
        y: 2,
        z: 3,
        rx: 4,
        ry: 5,
        rz: 6,
        period: 7,
    });
    let ev2 = ev.clone();
    assert_eq!(ev, ev2);
}

#[test]
fn spnav_event_is_debug() {
    let ev = SpnavEvent::Motion(MotionEvent {
        x: 1,
        y: 2,
        z: 3,
        rx: 4,
        ry: 5,
        rz: 6,
        period: 7,
    });
    let debug = format!("{:?}", ev);
    assert!(debug.contains("Motion"));
    assert!(debug.contains("1"));
}

// ─── MotionEvent tests ──────────────────────────────────────────────────────

#[test]
fn motion_event_zero_values() {
    let m = MotionEvent {
        x: 0,
        y: 0,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 0,
        period: 0,
    };
    assert_eq!(m.x, 0);
    assert_eq!(m.y, 0);
    assert_eq!(m.z, 0);
    assert_eq!(m.rx, 0);
    assert_eq!(m.ry, 0);
    assert_eq!(m.rz, 0);
    assert_eq!(m.period, 0);
}

#[test]
fn motion_event_equality() {
    let m1 = MotionEvent {
        x: 10,
        y: 20,
        z: 30,
        rx: 1,
        ry: 2,
        rz: 3,
        period: 16,
    };
    let m2 = MotionEvent {
        x: 10,
        y: 20,
        z: 30,
        rx: 1,
        ry: 2,
        rz: 3,
        period: 16,
    };
    assert_eq!(m1, m2);
}

#[test]
fn motion_event_inequality() {
    let m1 = MotionEvent {
        x: 10,
        y: 20,
        z: 30,
        rx: 1,
        ry: 2,
        rz: 3,
        period: 16,
    };
    let m2 = MotionEvent {
        x: 10,
        y: 20,
        z: 30,
        rx: 1,
        ry: 2,
        rz: 3,
        period: 17,
    };
    assert_ne!(m1, m2);
}

#[test]
fn motion_event_negative_values() {
    let m = MotionEvent {
        x: -100,
        y: -200,
        z: -300,
        rx: -10,
        ry: -20,
        rz: -30,
        period: 0,
    };
    assert_eq!(m.x, -100);
    assert_eq!(m.y, -200);
    assert_eq!(m.z, -300);
    assert_eq!(m.rx, -10);
    assert_eq!(m.ry, -20);
    assert_eq!(m.rz, -30);
}

// ─── ButtonEvent tests ──────────────────────────────────────────────────────

#[test]
fn button_event_zero_values() {
    let b = ButtonEvent {
        press: false,
        bnum: 0,
    };
    assert!(!b.press);
    assert_eq!(b.bnum, 0);
}

#[test]
fn button_event_press() {
    let b = ButtonEvent {
        press: true,
        bnum: 5,
    };
    assert!(b.press);
    assert_eq!(b.bnum, 5);
}

#[test]
fn button_event_release() {
    let b = ButtonEvent {
        press: false,
        bnum: 3,
    };
    assert!(!b.press);
    assert_eq!(b.bnum, 3);
}

#[test]
fn button_event_equality() {
    let b1 = ButtonEvent {
        press: true,
        bnum: 1,
    };
    let b2 = ButtonEvent {
        press: true,
        bnum: 1,
    };
    assert_eq!(b1, b2);
}

#[test]
fn button_event_inequality() {
    let b1 = ButtonEvent {
        press: true,
        bnum: 1,
    };
    let b2 = ButtonEvent {
        press: false,
        bnum: 1,
    };
    assert_ne!(b1, b2);
}

// ─── DeviceOp tests ─────────────────────────────────────────────────────────

#[test]
fn device_op_variants() {
    assert_ne!(DeviceOp::Add, DeviceOp::Remove);
}

#[test]
fn device_op_is_copy() {
    let op = DeviceOp::Add;
    let op2 = op;
    assert_eq!(op, op2);
}

// ─── DeviceType tests ───────────────────────────────────────────────────────

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

#[test]
fn device_type_is_copy() {
    let dt = DeviceType::SNav;
    let dt2 = dt;
    assert_eq!(dt, dt2);
}

#[test]
fn device_type_is_debug() {
    assert_eq!(format!("{:?}", DeviceType::SNav), "SNav");
    assert_eq!(format!("{:?}", DeviceType::SMPro), "SMPro");
}

// ─── DeviceEvent tests ──────────────────────────────────────────────────────

#[test]
fn device_event_zero_values() {
    let d = DeviceEvent {
        op: DeviceOp::Add,
        id: 0,
        devtype: DeviceType::Unknown,
        usb_vendor: 0,
        usb_product: 0,
    };
    assert_eq!(d.op, DeviceOp::Add);
    assert_eq!(d.id, 0);
    assert_eq!(d.devtype, DeviceType::Unknown);
    assert_eq!(d.usb_vendor, 0);
    assert_eq!(d.usb_product, 0);
}

#[test]
fn device_event_add_with_usb() {
    let d = DeviceEvent {
        op: DeviceOp::Add,
        id: 1,
        devtype: DeviceType::SMPro,
        usb_vendor: 0x046d,
        usb_product: 0xc629,
    };
    assert_eq!(d.op, DeviceOp::Add);
    assert_eq!(d.id, 1);
    assert_eq!(d.devtype, DeviceType::SMPro);
    assert_eq!(d.usb_vendor, 0x046d);
    assert_eq!(d.usb_product, 0xc629);
}

#[test]
fn device_event_remove() {
    let d = DeviceEvent {
        op: DeviceOp::Remove,
        id: 2,
        devtype: DeviceType::Unknown,
        usb_vendor: 0,
        usb_product: 0,
    };
    assert_eq!(d.op, DeviceOp::Remove);
}

// ─── ConfigEvent tests ──────────────────────────────────────────────────────

#[test]
fn config_event_zero_values() {
    let c = ConfigEvent {
        cfg: 0,
        data: [0; 6],
    };
    assert_eq!(c.cfg, 0);
    assert_eq!(c.data, [0; 6]);
}

#[test]
fn config_event_with_data() {
    let c = ConfigEvent {
        cfg: 5,
        data: [1, 2, 3, 4, 5, 6],
    };
    assert_eq!(c.cfg, 5);
    assert_eq!(c.data, [1, 2, 3, 4, 5, 6]);
}

// ─── RawAxisEvent tests ─────────────────────────────────────────────────────

#[test]
fn raw_axis_event_zero_values() {
    let a = RawAxisEvent { idx: 0, value: 0 };
    assert_eq!(a.idx, 0);
    assert_eq!(a.value, 0);
}

#[test]
fn raw_axis_event_with_values() {
    let a = RawAxisEvent {
        idx: 3,
        value: 12345,
    };
    assert_eq!(a.idx, 3);
    assert_eq!(a.value, 12345);
}

// ─── RawButtonEvent tests ───────────────────────────────────────────────────

#[test]
fn raw_button_event_zero_values() {
    let b = RawButtonEvent {
        bnum: 0,
        press: false,
    };
    assert_eq!(b.bnum, 0);
    assert!(!b.press);
}

#[test]
fn raw_button_event_press() {
    let b = RawButtonEvent {
        bnum: 2,
        press: true,
    };
    assert_eq!(b.bnum, 2);
    assert!(b.press);
}
