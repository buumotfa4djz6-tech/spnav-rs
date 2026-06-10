//! Event types for spacenav input devices.
//!
//! This module defines the event types emitted by spacenavd when the device state changes.
//! The main event type is [`SpnavEvent`], a union of all possible event variants.
//!
//! # Event Categories
//!
//! - **Motion** ([`MotionEvent`]): 6DOF translation and rotation values
//! - **Button** ([`ButtonEvent`]): Button press and release events
//! - **Device** ([`DeviceEvent`]): Device connection and disconnection
//! - **Config** ([`ConfigEvent`]): Configuration parameter changes
//! - **Raw axis/button** ([`RawAxisEvent`], [`RawButtonEvent`]): Pre-mapping values
//!
//! Each event has an associated [`EventType`] enum variant for filtering and matching.
//!
//! # Usage
//!
//! Events are received via [`SpnavClient::wait_event()`](crate::SpnavClient::wait_event)
//! or [`SpnavClient::poll_event()`](crate::SpnavClient::poll_event). Use pattern matching
//! to handle specific event types.

/// Types of spnav events.
///
/// Each variant corresponds to a specific event category reported by spacenavd.
/// Use [`SpnavEvent::event_type()`] to get the type of an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// 6DOF motion event (translation + rotation).
    Motion,
    /// Button press or release event.
    Button,
    /// Device add/remove event.
    Device,
    /// Configuration change event.
    Config,
    /// Raw axis value event (before mapping).
    RawAxis,
    /// Raw button value event (before mapping).
    RawButton,
}

/// Union of all possible spnav events.
///
/// This is the main event type returned by [`SpnavClient::wait_event()`](crate::SpnavClient::wait_event)
/// and [`SpnavClient::poll_event()`](crate::SpnavClient::poll_event).
///
/// # Example
///
/// ```
/// use spnav_rs::SpnavEvent;
///
/// fn handle_event(event: SpnavEvent) {
///     match event {
///         SpnavEvent::Motion(m) => println!("Motion: {} {} {}", m.x, m.y, m.z),
///         SpnavEvent::Button(b) => println!("Button {}: {}", b.bnum, b.press),
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum SpnavEvent {
    /// 6DOF motion with translation and rotation values.
    Motion(MotionEvent),
    /// Button press or release.
    Button(ButtonEvent),
    /// Device added or removed.
    Device(DeviceEvent),
    /// Configuration changed.
    Config(ConfigEvent),
    /// Raw axis value (before axis mapping).
    RawAxis(RawAxisEvent),
    /// Raw button value (before button mapping).
    RawButton(RawButtonEvent),
}

impl SpnavEvent {
    /// Get the event type.
    ///
    /// Returns the [`EventType`] variant corresponding to this event.
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

/// 6DOF motion event.
///
/// Contains the translation (x, y, z) and rotation (rx, ry, rz) values
/// reported by the device. Values are in device-specific units and should
/// be scaled according to your application's needs.
///
/// Use [`PositionRot`](crate::PositionRot) to accumulate motion events
/// into a position vector and orientation quaternion.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionEvent {
    /// Translation along the X axis.
    pub x: i32,
    /// Translation along the Y axis.
    pub y: i32,
    /// Translation along the Z axis.
    pub z: i32,
    /// Rotation around the X axis.
    pub rx: i32,
    /// Rotation around the Y axis.
    pub ry: i32,
    /// Rotation around the Z axis.
    pub rz: i32,
    /// Time period since last motion event (in milliseconds).
    pub period: u32,
}

/// Button press or release event.
///
/// Generated when a button on the device is pressed or released.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonEvent {
    /// `true` for press, `false` for release.
    pub press: bool,
    /// Button number (0-indexed).
    pub bnum: i32,
}

/// Device operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceOp {
    /// Device was added.
    Add,
    /// Device was removed.
    Remove,
}

/// Known device types.
///
/// These correspond to the device type IDs reported by spacenavd.
/// Serial devices have IDs in the `0x100..0x200` range,
/// USB devices have IDs starting at `0x200`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Unknown or unrecognized device type.
    Unknown = 0,
    // Serial devices
    /// Spaceball 1003/2003/2003C.
    Sb2003 = 0x100,
    /// Spaceball 3003/3003C.
    Sb3003,
    /// Spaceball 4000FLX/5000FLX.
    Sb4000,
    /// Magellan SpaceMouse (serial).
    Sm,
    /// Spaceball 5000 (SpaceMouse protocol).
    Sm5000,
    /// 3Dconnexion CadMan (SpaceMouse protocol).
    SmCadman,
    // USB devices
    /// SpaceMouse Plus XT.
    PlusXt = 0x200,
    /// 3Dconnexion CadMan (USB).
    Cadman,
    /// SpaceMouse Classic.
    SmClassic,
    /// Spaceball 5000 (USB).
    Sb5000,
    /// Space Traveller.
    STraveller,
    /// Space Pilot.
    SPilot,
    /// Space Navigator.
    SNav,
    /// Space Explorer.
    SExp,
    /// Space Navigator for Notebooks.
    SNavNb,
    /// Space Pilot Pro.
    SPilotPro,
    /// SpaceMouse Pro.
    SMPro,
    /// NuLOOQ.
    NuLooq,
    /// SpaceMouse Wireless.
    Smw,
    /// SpaceMouse Pro Wireless.
    SMProW,
    /// SpaceMouse Enterprise.
    SMEnt,
    /// SpaceMouse Compact.
    SMComp,
    /// SpaceMouse Module.
    SMMod,
}

impl DeviceType {
    /// Convert a raw device type ID to a [`DeviceType`] variant.
    ///
    /// Returns [`DeviceType::Unknown`] for unrecognized IDs.
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

/// Device add/remove event.
///
/// Generated when a device is connected to or disconnected from the system.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceEvent {
    /// Whether the device was added or removed.
    pub op: DeviceOp,
    /// Device ID.
    pub id: i32,
    /// Device type.
    pub devtype: DeviceType,
    /// USB vendor ID (0 if serial device).
    pub usb_vendor: u32,
    /// USB product ID (0 if serial device).
    pub usb_product: u32,
}

/// Configuration change event.
///
/// Generated when the spacenavd configuration is modified.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigEvent {
    /// Configuration parameter that changed (matches protocol `REQ_GCFG*` constants).
    pub cfg: i32,
    /// Configuration data (6 values, same as protocol response data 0-5).
    pub data: [i32; 6],
}

/// Raw axis value event.
///
/// Reports the raw value of a device axis before axis mapping is applied.
#[derive(Debug, Clone, PartialEq)]
pub struct RawAxisEvent {
    /// Axis number (0-indexed).
    pub idx: i32,
    /// Raw axis value.
    pub value: i32,
}

/// Raw button value event.
///
/// Reports the raw state of a device button before button mapping is applied.
#[derive(Debug, Clone, PartialEq)]
pub struct RawButtonEvent {
    /// Button number (0-indexed).
    pub bnum: i32,
    /// `true` for pressed, `false` for released.
    pub press: bool,
}
