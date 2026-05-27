//! Integration tests for the protocol module.

use spnav_rs::event::*;
use spnav_rs::protocol::*;

// ─── ReqResp frame tests ────────────────────────────────────────────────────

#[test]
fn reqresp_zeroed_is_all_zeros() {
    let rr = ReqResp::zeroed();
    assert_eq!(rr.type_, 0);
    assert_eq!(rr.data, [0; 7]);
}

#[test]
fn reqresp_byte_roundtrip_preserves_all_fields() {
    let rr = ReqResp {
        type_: 0x7FAA1234,
        data: [100, -200, 300, -400, 500, -600, 700],
    };
    let bytes = rr.to_bytes();
    let rr2 = ReqResp::from_bytes(&bytes);
    assert_eq!(rr.type_, rr2.type_);
    assert_eq!(rr.data, rr2.data);
}

#[test]
fn reqresp_byte_roundtrip_negative_values() {
    let rr = ReqResp {
        type_: -1,
        data: [i32::MIN, i32::MAX, -1, 0, -100, 100, -999],
    };
    let bytes = rr.to_bytes();
    let rr2 = ReqResp::from_bytes(&bytes);
    assert_eq!(rr.type_, rr2.type_);
    assert_eq!(rr.data, rr2.data);
}

#[test]
fn reqresp_to_bytes_is_32_bytes() {
    let rr = ReqResp::zeroed();
    let bytes = rr.to_bytes();
    assert_eq!(bytes.len(), 32);
}

// ─── Event detection tests ──────────────────────────────────────────────────

#[test]
fn motion_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_MOTION;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn press_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_PRESS;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn release_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_RELEASE;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn dev_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_DEV;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn cfg_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_CFG;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn rawaxis_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_RAWAXIS;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn rawbutton_event_is_detected() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = UEV_RAWBUTTON;
    assert!(rr.is_event());
    assert!(!rr.is_response());
}

#[test]
fn request_tagged_is_not_event() {
    let rr = make_request(req::SET_SENS, [0; 7]);
    assert!(!rr.is_event());
    assert!(rr.is_response());
}

#[test]
fn negative_type_is_not_event() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = -1;
    assert!(!rr.is_event());
}

#[test]
fn type_above_max_uev_is_not_event() {
    let mut rr = ReqResp::zeroed();
    rr.type_ = MAX_UEV + 1;
    assert!(!rr.is_event());
}

// ─── Status tests ───────────────────────────────────────────────────────────

#[test]
fn status_ok_positive_value() {
    let mut rr = ReqResp::zeroed();
    rr.data[6] = 1;
    assert!(rr.status_ok());
}

#[test]
fn status_ok_zero_value() {
    let mut rr = ReqResp::zeroed();
    rr.data[6] = 0;
    assert!(rr.status_ok());
}

#[test]
fn status_ok_negative_value() {
    let mut rr = ReqResp::zeroed();
    rr.data[6] = -1;
    assert!(!rr.status_ok());
}

// ─── make_request tests ─────────────────────────────────────────────────────

#[test]
fn make_request_sets_tag() {
    let rr = make_request(req::SET_SENS, [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(rr.type_, REQ_TAG | req::SET_SENS);
    assert_eq!(rr.data, [1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn make_request_with_zero_data() {
    let rr = make_request(req::GET_SENS, [0; 7]);
    assert_eq!(rr.type_, REQ_TAG | req::GET_SENS);
    assert_eq!(rr.data, [0; 7]);
}

// ─── decode_event tests ─────────────────────────────────────────────────────

fn encode_motion_frame(x: i32, y: i32, z: i32, rx: i32, ry: i32, rz: i32, period: i32) -> [u8; 32] {
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

fn encode_button_frame(press: bool, bnum: i32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let evt_type = if press { UEV_PRESS } else { UEV_RELEASE };
    buf[0..4].copy_from_slice(&evt_type.to_ne_bytes());
    buf[4..8].copy_from_slice(&bnum.to_ne_bytes());
    buf
}

fn encode_dev_frame(op: i32, id: i32, devtype: i32, vendor: i32, product: i32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_DEV.to_ne_bytes());
    buf[4..8].copy_from_slice(&op.to_ne_bytes());
    buf[8..12].copy_from_slice(&id.to_ne_bytes());
    buf[12..16].copy_from_slice(&devtype.to_ne_bytes());
    buf[16..20].copy_from_slice(&vendor.to_ne_bytes());
    buf[20..24].copy_from_slice(&product.to_ne_bytes());
    buf
}

fn encode_cfg_frame(cfg: i32, data: [i32; 6]) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_CFG.to_ne_bytes());
    buf[4..8].copy_from_slice(&cfg.to_ne_bytes());
    for i in 0..6 {
        let off = 8 + i * 4;
        buf[off..off + 4].copy_from_slice(&data[i].to_ne_bytes());
    }
    buf
}

fn encode_rawaxis_frame(idx: i32, value: i32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_RAWAXIS.to_ne_bytes());
    buf[4..8].copy_from_slice(&idx.to_ne_bytes());
    buf[8..12].copy_from_slice(&value.to_ne_bytes());
    buf
}

fn encode_rawbutton_frame(bnum: i32, press: bool) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&UEV_RAWBUTTON.to_ne_bytes());
    buf[4..8].copy_from_slice(&bnum.to_ne_bytes());
    buf[8..12].copy_from_slice(&(press as i32).to_ne_bytes());
    buf
}

#[test]
fn decode_motion_all_axes() {
    let buf = encode_motion_frame(100, -200, 300, -10, 20, -30, 16);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Motion(m) => {
            assert_eq!(m.x, 100);
            assert_eq!(m.y, -200);
            assert_eq!(m.z, 300);
            assert_eq!(m.rx, -10);
            assert_eq!(m.ry, 20);
            assert_eq!(m.rz, -30);
            assert_eq!(m.period, 16);
        }
        _ => panic!("expected motion"),
    }
}

#[test]
fn decode_motion_zero_values() {
    let buf = encode_motion_frame(0, 0, 0, 0, 0, 0, 0);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Motion(m) => {
            assert_eq!(m.x, 0);
            assert_eq!(m.y, 0);
            assert_eq!(m.z, 0);
            assert_eq!(m.rx, 0);
            assert_eq!(m.ry, 0);
            assert_eq!(m.rz, 0);
            assert_eq!(m.period, 0);
        }
        _ => panic!("expected motion"),
    }
}

#[test]
fn decode_button_press() {
    let buf = encode_button_frame(true, 3);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Button(b) => {
            assert!(b.press);
            assert_eq!(b.bnum, 3);
        }
        _ => panic!("expected button"),
    }
}

#[test]
fn decode_button_release() {
    let buf = encode_button_frame(false, 0);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Button(b) => {
            assert!(!b.press);
            assert_eq!(b.bnum, 0);
        }
        _ => panic!("expected button"),
    }
}

#[test]
fn decode_device_add() {
    let buf = encode_dev_frame(0, 1, 0x206, 0x046d, 0xc629);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Device(d) => {
            assert_eq!(d.op, DeviceOp::Add);
            assert_eq!(d.id, 1);
            assert_eq!(d.devtype, DeviceType::SNav);
            assert_eq!(d.usb_vendor, 0x046d);
            assert_eq!(d.usb_product, 0xc629);
        }
        _ => panic!("expected device"),
    }
}

#[test]
fn decode_device_remove() {
    let buf = encode_dev_frame(1, 2, 0, 0, 0);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Device(d) => {
            assert_eq!(d.op, DeviceOp::Remove);
            assert_eq!(d.id, 2);
        }
        _ => panic!("expected device"),
    }
}

#[test]
fn decode_config_event() {
    let data = [10, 20, 30, 40, 50, 60];
    let buf = encode_cfg_frame(5, data);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::Config(c) => {
            assert_eq!(c.cfg, 5);
            assert_eq!(c.data, data);
        }
        _ => panic!("expected config"),
    }
}

#[test]
fn decode_rawaxis_event() {
    let buf = encode_rawaxis_frame(3, 12345);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::RawAxis(a) => {
            assert_eq!(a.idx, 3);
            assert_eq!(a.value, 12345);
        }
        _ => panic!("expected raw axis"),
    }
}

#[test]
fn decode_rawbutton_press() {
    let buf = encode_rawbutton_frame(5, true);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::RawButton(b) => {
            assert_eq!(b.bnum, 5);
            assert!(b.press);
        }
        _ => panic!("expected raw button"),
    }
}

#[test]
fn decode_rawbutton_release() {
    let buf = encode_rawbutton_frame(0, false);
    let ev = decode_event(&buf).unwrap();
    match ev {
        SpnavEvent::RawButton(b) => {
            assert_eq!(b.bnum, 0);
            assert!(!b.press);
        }
        _ => panic!("expected raw button"),
    }
}

#[test]
fn decode_unknown_event_type_returns_none() {
    let mut buf = [0u8; 32];
    buf[0..4].copy_from_slice(&(MAX_UEV + 100).to_ne_bytes());
    // This type is above MAX_UEV, so is_event() returns false
    assert!(decode_event(&buf).is_none());
}

#[test]
fn decode_request_frame_returns_none() {
    let rr = make_request(req::SET_SENS, [0; 7]);
    let buf = rr.to_bytes();
    assert!(decode_event(&buf).is_none());
}

// ─── Request type constants ─────────────────────────────────────────────────

#[test]
fn request_constants_are_unique() {
    // Note: CFG_SAVE and CFG_RESET have the same value (0x3ffe) in the C protocol
    let constants = [
        req::SET_NAME,
        req::SET_SENS,
        req::GET_SENS,
        req::SET_EVMASK,
        req::GET_EVMASK,
        req::DEV_NAME,
        req::DEV_PATH,
        req::DEV_NAXES,
        req::DEV_NBUTTONS,
        req::DEV_USBID,
        req::DEV_TYPE,
        req::SCFG_SENS,
        req::GCFG_SENS,
        req::SCFG_SENS_AXIS,
        req::GCFG_SENS_AXIS,
        req::SCFG_DEADZONE,
        req::GCFG_DEADZONE,
        req::SCFG_INVERT,
        req::GCFG_INVERT,
        req::SCFG_AXISMAP,
        req::GCFG_AXISMAP,
        req::SCFG_BNMAP,
        req::GCFG_BNMAP,
        req::SCFG_BNACTION,
        req::GCFG_BNACTION,
        req::SCFG_KBMAP,
        req::GCFG_KBMAP,
        req::SCFG_SWAPYZ,
        req::GCFG_SWAPYZ,
        req::SCFG_LED,
        req::GCFG_LED,
        req::SCFG_GRAB,
        req::GCFG_GRAB,
        req::SCFG_SERDEV,
        req::GCFG_SERDEV,
        req::SCFG_REPEAT,
        req::GCFG_REPEAT,
        req::CFG_RESTORE,
        req::CHANGE_PROTO,
    ];

    // All constants should be unique
    for i in 0..constants.len() {
        for j in (i + 1)..constants.len() {
            assert_ne!(
                constants[i], constants[j],
                "duplicate request constant at indices {} and {}",
                i, j
            );
        }
    }
}

#[test]
fn cfg_save_and_reset_share_value() {
    // CFG_SAVE and CFG_RESET have the same value (0x3ffe) in the C protocol
    assert_eq!(req::CFG_SAVE, req::CFG_RESET);
}

#[test]
fn request_constants_are_tagged_correctly() {
    // Per-client settings should be >= REQ_BASE
    assert!(req::SET_NAME >= REQ_BASE);
    assert!(req::SET_SENS >= REQ_BASE);
    assert!(req::GET_SENS >= REQ_BASE);
    assert!(req::SET_EVMASK >= REQ_BASE);
    assert!(req::GET_EVMASK >= REQ_BASE);
}

#[test]
fn protocol_constants_match_c_values() {
    // These values must match the C libspnav protocol.h
    assert_eq!(REQ_TAG, 0x7faa_0000);
    assert_eq!(REQ_BASE, 0x1000);
    assert_eq!(MAX_PROTO_VER, 1);
    assert_eq!(UEV_MOTION, 0);
    assert_eq!(UEV_PRESS, 1);
    assert_eq!(UEV_RELEASE, 2);
    assert_eq!(UEV_DEV, 3);
    assert_eq!(UEV_CFG, 4);
    assert_eq!(UEV_RAWAXIS, 5);
    assert_eq!(UEV_RAWBUTTON, 6);
    assert_eq!(MAX_UEV, 7);
}
