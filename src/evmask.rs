//! Event mask types for filtering spacenav events.

bitflags::bitflags! {
    /// Bitmask for selecting which event types to receive.
    ///
    /// Used with [`SpnavClient::set_event_mask()`](crate::SpnavClient::set_event_mask)
    /// to filter events. Only events matching the mask will be delivered.
    ///
    /// # Example
    ///
    /// ```
    /// use spnav_rs::EventMask;
    ///
    /// // Only receive motion and button events
    /// let mask = EventMask::MOTION | EventMask::BUTTON;
    ///
    /// // Receive all event types
    /// let mask = EventMask::ALL;
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventMask: u16 {
        /// 6DOF motion events.
        const MOTION     = 0x01;
        /// Button press/release events.
        const BUTTON     = 0x02;
        /// Device add/remove events.
        const DEV        = 0x04;
        /// Configuration change events.
        const CFG        = 0x08;
        /// Raw axis value events (before axis mapping).
        const RAW_AXIS   = 0x10;
        /// Raw button value events (before button mapping).
        const RAW_BUTTON = 0x20;

        /// Input events only (motion + button).
        const INPUT = Self::MOTION.bits() | Self::BUTTON.bits();
        /// Default mask (input + device events).
        const DEFAULT = Self::INPUT.bits() | Self::DEV.bits();
        /// All event types.
        const ALL = 0xFFFF;
    }
}
