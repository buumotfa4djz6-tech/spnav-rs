//! Integration tests for EventMask.

use spnav_rs::EventMask;

// ─── Bit value specs ─────────────────────────────────────────────────────────

#[test]
fn motion_bit_is_0x01() {
    assert_eq!(EventMask::MOTION.bits(), 0x01);
}

#[test]
fn button_bit_is_0x02() {
    assert_eq!(EventMask::BUTTON.bits(), 0x02);
}

#[test]
fn dev_bit_is_0x04() {
    assert_eq!(EventMask::DEV.bits(), 0x04);
}

#[test]
fn cfg_bit_is_0x08() {
    assert_eq!(EventMask::CFG.bits(), 0x08);
}

#[test]
fn raw_axis_bit_is_0x10() {
    assert_eq!(EventMask::RAW_AXIS.bits(), 0x10);
}

#[test]
fn raw_button_bit_is_0x20() {
    assert_eq!(EventMask::RAW_BUTTON.bits(), 0x20);
}

// ─── Composite mask specs ────────────────────────────────────────────────────

#[test]
fn input_mask_is_motion_and_button() {
    assert_eq!(EventMask::INPUT.bits(), 0x03);
    assert!(EventMask::INPUT.contains(EventMask::MOTION));
    assert!(EventMask::INPUT.contains(EventMask::BUTTON));
    assert!(!EventMask::INPUT.contains(EventMask::DEV));
}

#[test]
fn default_mask_is_input_and_dev() {
    assert_eq!(EventMask::DEFAULT.bits(), 0x07);
    assert!(EventMask::DEFAULT.contains(EventMask::MOTION));
    assert!(EventMask::DEFAULT.contains(EventMask::BUTTON));
    assert!(EventMask::DEFAULT.contains(EventMask::DEV));
    assert!(!EventMask::DEFAULT.contains(EventMask::CFG));
    assert!(!EventMask::DEFAULT.contains(EventMask::RAW_AXIS));
    assert!(!EventMask::DEFAULT.contains(EventMask::RAW_BUTTON));
}

#[test]
fn all_mask_is_0xffff() {
    assert_eq!(EventMask::ALL.bits(), 0xFFFF);
}

#[test]
fn all_mask_contains_everything() {
    assert!(EventMask::ALL.contains(EventMask::MOTION));
    assert!(EventMask::ALL.contains(EventMask::BUTTON));
    assert!(EventMask::ALL.contains(EventMask::DEV));
    assert!(EventMask::ALL.contains(EventMask::CFG));
    assert!(EventMask::ALL.contains(EventMask::RAW_AXIS));
    assert!(EventMask::ALL.contains(EventMask::RAW_BUTTON));
}
