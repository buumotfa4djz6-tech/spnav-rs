use crate::error::{Error, Result};
use crate::event::{
    ButtonEvent, ConfigEvent, DeviceEvent, DeviceOp, DeviceType, MotionEvent, RawAxisEvent,
    RawButtonEvent, SpnavEvent,
};

pub const REQ_TAG: i32 = 0x7faa_0000;
pub const REQ_BASE: i32 = 0x1000;
pub const MAX_PROTO_VER: i32 = 1;

// Event types from daemon
pub const UEV_MOTION: i32 = 0;
pub const UEV_PRESS: i32 = 1;
pub const UEV_RELEASE: i32 = 2;
pub const UEV_DEV: i32 = 3;
pub const UEV_CFG: i32 = 4;
pub const UEV_RAWAXIS: i32 = 5;
pub const UEV_RAWBUTTON: i32 = 6;
pub const MAX_UEV: i32 = 7;

// Request types
pub mod req {
    use super::*;
    // Per-client settings
    pub const SET_NAME: i32 = REQ_BASE;
    pub const SET_SENS: i32 = REQ_BASE + 1;
    pub const GET_SENS: i32 = REQ_BASE + 2;
    pub const SET_EVMASK: i32 = REQ_BASE + 3;
    pub const GET_EVMASK: i32 = REQ_BASE + 4;

    // Device queries
    pub const DEV_NAME: i32 = 0x2000;
    pub const DEV_PATH: i32 = 0x2001;
    pub const DEV_NAXES: i32 = 0x2002;
    pub const DEV_NBUTTONS: i32 = 0x2003;
    pub const DEV_USBID: i32 = 0x2004;
    pub const DEV_TYPE: i32 = 0x2005;

    // Configuration settings
    pub const SCFG_SENS: i32 = 0x3000;
    pub const GCFG_SENS: i32 = 0x3001;
    pub const SCFG_SENS_AXIS: i32 = 0x3002;
    pub const GCFG_SENS_AXIS: i32 = 0x3003;
    pub const SCFG_DEADZONE: i32 = 0x3004;
    pub const GCFG_DEADZONE: i32 = 0x3005;
    pub const SCFG_INVERT: i32 = 0x3006;
    pub const GCFG_INVERT: i32 = 0x3007;
    pub const SCFG_AXISMAP: i32 = 0x3008;
    pub const GCFG_AXISMAP: i32 = 0x3009;
    pub const SCFG_BNMAP: i32 = 0x300a;
    pub const GCFG_BNMAP: i32 = 0x300b;
    pub const SCFG_BNACTION: i32 = 0x300c;
    pub const GCFG_BNACTION: i32 = 0x300d;
    pub const SCFG_KBMAP: i32 = 0x300e;
    pub const GCFG_KBMAP: i32 = 0x300f;
    pub const SCFG_SWAPYZ: i32 = 0x3010;
    pub const GCFG_SWAPYZ: i32 = 0x3011;
    pub const SCFG_LED: i32 = 0x3012;
    pub const GCFG_LED: i32 = 0x3013;
    pub const SCFG_GRAB: i32 = 0x3014;
    pub const GCFG_GRAB: i32 = 0x3015;
    pub const SCFG_SERDEV: i32 = 0x3016;
    pub const GCFG_SERDEV: i32 = 0x3017;
    pub const SCFG_REPEAT: i32 = 0x3018;
    pub const GCFG_REPEAT: i32 = 0x3019;

    // Config management
    pub const CFG_SAVE: i32 = 0x3ffe;
    pub const CFG_RESTORE: i32 = 0x3fff;
    pub const CFG_RESET: i32 = 0x3fff - 1;

    // Protocol change
    pub const CHANGE_PROTO: i32 = 0x5500;
}

pub const REQSTR_CHUNK_SIZE: usize = 24;
pub const REQSTR_CONT_BIT: i32 = 0x10000;

/// A 32-byte request/response frame (8 x i32)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ReqResp {
    pub type_: i32,
    pub data: [i32; 7],
}

impl ReqResp {
    pub fn zeroed() -> Self {
        Self {
            type_: 0,
            data: [0; 7],
        }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[0..4].copy_from_slice(&self.type_.to_ne_bytes());
        for i in 0..7 {
            let off = 4 + i * 4;
            buf[off..off + 4].copy_from_slice(&self.data[i].to_ne_bytes());
        }
        buf
    }

    pub fn from_bytes(buf: &[u8; 32]) -> Self {
        let type_ = i32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let mut data = [0i32; 7];
        for (i, item) in data.iter_mut().enumerate() {
            let off = 4 + i * 4;
            *item = i32::from_ne_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        }
        Self { type_, data }
    }

    pub fn is_event(&self) -> bool {
        self.type_ >= 0 && self.type_ < MAX_UEV
    }

    pub fn is_response(&self) -> bool {
        (self.type_ & REQ_TAG) != 0
    }

    pub fn status_ok(&self) -> bool {
        self.data[6] >= 0
    }
}

/// Build a tagged request frame
pub fn make_request(req_type: i32, data: [i32; 7]) -> ReqResp {
    ReqResp {
        type_: REQ_TAG | req_type,
        data,
    }
}

/// Decode an incoming 32-byte event frame into SpnavEvent
pub fn decode_event(buf: &[u8; 32]) -> Option<SpnavEvent> {
    let rr = ReqResp::from_bytes(buf);
    if !rr.is_event() {
        return None;
    }

    let evt_type = rr.type_;
    match evt_type {
        UEV_MOTION => Some(SpnavEvent::Motion(MotionEvent {
            x: rr.data[0],
            y: rr.data[1],
            z: rr.data[2],
            rx: rr.data[3],
            ry: rr.data[4],
            rz: rr.data[5],
            period: rr.data[6] as u32,
        })),
        UEV_PRESS | UEV_RELEASE => Some(SpnavEvent::Button(ButtonEvent {
            press: evt_type == UEV_PRESS,
            bnum: rr.data[0],
        })),
        UEV_DEV => Some(SpnavEvent::Device(DeviceEvent {
            op: if rr.data[0] == 0 { DeviceOp::Add } else { DeviceOp::Remove },
            id: rr.data[1],
            devtype: DeviceType::from_raw(rr.data[2]),
            usb_vendor: rr.data[3] as u32,
            usb_product: rr.data[4] as u32,
        })),
        UEV_CFG => Some(SpnavEvent::Config(ConfigEvent {
            cfg: rr.data[0],
            data: [rr.data[1], rr.data[2], rr.data[3], rr.data[4], rr.data[5], rr.data[6]],
        })),
        UEV_RAWAXIS => Some(SpnavEvent::RawAxis(RawAxisEvent {
            idx: rr.data[0],
            value: rr.data[1],
        })),
        UEV_RAWBUTTON => Some(SpnavEvent::RawButton(RawButtonEvent {
            bnum: rr.data[0],
            press: rr.data[1] != 0,
        })),
        _ => None,
    }
}

/// Chunked string encoder: produces a sequence of ReqResp frames
pub fn encode_string_chunks(req_type: i32, s: &str) -> Vec<ReqResp> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut chunks = Vec::new();
    let mut offset = 0;
    let mut remaining = len as i32;

    while remaining > 0 {
        let chunk_len = (remaining as usize).min(REQSTR_CHUNK_SIZE);
        let mut rr = ReqResp::zeroed();
        rr.type_ = REQ_TAG | req_type;

        for i in 0..chunk_len {
            rr.data[i / 4] |= (bytes[offset + i] as i32) << ((i % 4) * 8);
        }

        if offset == 0 {
            rr.data[6] = len as i32;
        } else {
            rr.data[6] = remaining | REQSTR_CONT_BIT;
        }

        chunks.push(rr);
        offset += chunk_len;
        remaining -= chunk_len as i32;
    }

    if chunks.is_empty() {
        let mut rr = ReqResp::zeroed();
        rr.type_ = REQ_TAG | req_type;
        rr.data[6] = 0;
        chunks.push(rr);
    }

    chunks
}

/// State machine for decoding chunked string responses
#[derive(Debug)]
pub struct StringDecoder {
    buf: Vec<u8>,
    expect: usize,
    size: usize,
}

impl Default for StringDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl StringDecoder {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            expect: 0,
            size: 0,
        }
    }

    /// Feed a response frame. Returns Some(String) when complete.
    pub fn feed(&mut self, rr: &ReqResp) -> Result<Option<String>> {
        if rr.data[6] < 0 {
            return Err(Error::DaemonFailure);
        }

        let remaining = (rr.data[6] & 0xffff) as usize;
        let is_first = (rr.data[6] & REQSTR_CONT_BIT) == 0;

        if is_first {
            self.expect = remaining;
            self.size = self.expect + 1; // +1 for null terminator
            self.buf = Vec::with_capacity(self.size);
        }

        if self.buf.capacity() == 0 {
            return Err(Error::Protocol("string decoder not initialized".into()));
        }

        let to_copy = remaining.min(REQSTR_CHUNK_SIZE);
        for i in 0..to_copy {
            let byte = ((rr.data[i / 4] >> ((i % 4) * 8)) & 0xff) as u8;
            self.buf.push(byte);
        }

        self.expect -= to_copy;

        if self.expect == 0 {
            self.buf.push(0);
            let s = String::from_utf8_lossy(&self.buf[..self.buf.len() - 1]).to_string();
            self.buf.clear();
            return Ok(Some(s));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reqresp_roundtrip() {
        let rr = ReqResp {
            type_: 0x12345678,
            data: [1, 2, 3, 4, 5, 6, 7],
        };
        let bytes = rr.to_bytes();
        let rr2 = ReqResp::from_bytes(&bytes);
        assert_eq!(rr.type_, rr2.type_);
        assert_eq!(rr.data, rr2.data);
    }

    #[test]
    fn test_is_event_vs_response() {
        let event_buf: [u8; 32] = {
            let mut b = [0u8; 32];
            b[0..4].copy_from_slice(&UEV_MOTION.to_ne_bytes());
            b
        };
        let rr = ReqResp::from_bytes(&event_buf);
        assert!(rr.is_event());
        assert!(!rr.is_response());

        let resp = make_request(req::SET_SENS, [0; 7]);
        assert!(!resp.is_event());
        assert!(resp.is_response());
    }

    #[test]
    fn test_encode_string_chunks() {
        let chunks = encode_string_chunks(req::SET_NAME, "hello");
        assert!(!chunks.is_empty());
        // First chunk should have remaining length = 5 in data[6]
        assert_eq!(chunks[0].data[6], 5);
    }

    #[test]
    fn test_decode_motion_event() {
        let mut buf = [0u8; 32];
        buf[0..4].copy_from_slice(&UEV_MOTION.to_ne_bytes());
        buf[4..8].copy_from_slice(&10i32.to_ne_bytes());  // x
        buf[8..12].copy_from_slice(&20i32.to_ne_bytes()); // y
        buf[12..16].copy_from_slice(&30i32.to_ne_bytes()); // z
        buf[16..20].copy_from_slice(&1i32.to_ne_bytes());  // rx
        buf[20..24].copy_from_slice(&2i32.to_ne_bytes());  // ry
        buf[24..28].copy_from_slice(&3i32.to_ne_bytes());  // rz
        buf[28..32].copy_from_slice(&100i32.to_ne_bytes()); // period

        let ev = decode_event(&buf).unwrap();
        match ev {
            SpnavEvent::Motion(m) => {
                assert_eq!(m.x, 10);
                assert_eq!(m.y, 20);
                assert_eq!(m.z, 30);
                assert_eq!(m.rx, 1);
                assert_eq!(m.ry, 2);
                assert_eq!(m.rz, 3);
                assert_eq!(m.period, 100);
            }
            _ => panic!("expected motion event"),
        }
    }
}
