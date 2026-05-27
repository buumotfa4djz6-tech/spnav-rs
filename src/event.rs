/// Types of spnav events (mirrors C API SPNAV_EVENT_* constants)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Motion,
    Button,
    Device,
    Config,
    RawAxis,
    RawButton,
}

/// Union of all possible spnav events
#[derive(Debug, Clone, PartialEq)]
pub enum SpnavEvent {
    Motion(MotionEvent),
    Button(ButtonEvent),
    Device(DeviceEvent),
    Config(ConfigEvent),
    RawAxis(RawAxisEvent),
    RawButton(RawButtonEvent),
}

impl SpnavEvent {
    pub fn event_type(&self) -> EventType {
        match self {
            SpnavEvent::Motion(_) => EventType::Motion,
            SpnavEvent::Button(_) => EventType::Button,
            SpnavEvent::Device(_) => EventType::Device,
            SpnavEvent::Config(_) => EventType::Config,
            SpnavEvent::RawAxis(_) => EventType::RawAxis,
            SpnavEvent::RawButton(_) => EventType::RawButton,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub period: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonEvent {
    pub press: bool,
    pub bnum: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceOp {
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Unknown = 0,
    // Serial devices
    Sb2003 = 0x100,
    Sb3003,
    Sb4000,
    Sm,
    Sm5000,
    SmCadman,
    // USB devices
    PlusXt = 0x200,
    Cadman,
    SmClassic,
    Sb5000,
    STraveller,
    SPilot,
    SNav,
    SExp,
    SNavNb,
    SPilotPro,
    SMPro,
    NuLooq,
    Smw,
    SMProW,
    SMEnt,
    SMComp,
    SMMod,
}

impl DeviceType {
    pub fn from_raw(val: i32) -> Self {
        match val {
            0 => DeviceType::Unknown,
            0x100 => DeviceType::Sb2003,
            0x101 => DeviceType::Sb3003,
            0x102 => DeviceType::Sb4000,
            0x103 => DeviceType::Sm,
            0x104 => DeviceType::Sm5000,
            0x105 => DeviceType::SmCadman,
            0x200 => DeviceType::PlusXt,
            0x201 => DeviceType::Cadman,
            0x202 => DeviceType::SmClassic,
            0x203 => DeviceType::Sb5000,
            0x204 => DeviceType::STraveller,
            0x205 => DeviceType::SPilot,
            0x206 => DeviceType::SNav,
            0x207 => DeviceType::SExp,
            0x208 => DeviceType::SNavNb,
            0x209 => DeviceType::SPilotPro,
            0x20a => DeviceType::SMPro,
            0x20b => DeviceType::NuLooq,
            0x20c => DeviceType::Smw,
            0x20d => DeviceType::SMProW,
            0x20e => DeviceType::SMEnt,
            0x20f => DeviceType::SMComp,
            0x210 => DeviceType::SMMod,
            _ => DeviceType::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceEvent {
    pub op: DeviceOp,
    pub id: i32,
    pub devtype: DeviceType,
    pub usb_vendor: u32,
    pub usb_product: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigEvent {
    pub cfg: i32,
    pub data: [i32; 6],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawAxisEvent {
    pub idx: i32,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawButtonEvent {
    pub bnum: i32,
    pub press: bool,
}
