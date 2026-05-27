bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventMask: u16 {
        const MOTION     = 0x01;
        const BUTTON     = 0x02;
        const DEV        = 0x04;
        const CFG        = 0x08;
        const RAW_AXIS   = 0x10;
        const RAW_BUTTON = 0x20;

        const INPUT = Self::MOTION.bits() | Self::BUTTON.bits();
        const DEFAULT = Self::INPUT.bits() | Self::DEV.bits();
        const ALL = 0xFFFF;
    }
}
