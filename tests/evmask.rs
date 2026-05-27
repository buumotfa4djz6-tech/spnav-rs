//! Integration tests for EventMask.

use spnav_rs::EventMask;

// ─── Basic bit tests ────────────────────────────────────────────────────────

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

// ─── Composite masks ────────────────────────────────────────────────────────

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

// ─── Combination tests ──────────────────────────────────────────────────────

#[test]
fn combine_motion_and_button() {
    let mask = EventMask::MOTION | EventMask::BUTTON;
    assert!(mask.contains(EventMask::MOTION));
    assert!(mask.contains(EventMask::BUTTON));
    assert!(!mask.contains(EventMask::DEV));
}

#[test]
fn combine_all_input_types() {
    let mask = EventMask::MOTION | EventMask::BUTTON | EventMask::DEV | EventMask::CFG;
    assert!(mask.contains(EventMask::MOTION));
    assert!(mask.contains(EventMask::BUTTON));
    assert!(mask.contains(EventMask::DEV));
    assert!(mask.contains(EventMask::CFG));
    assert!(!mask.contains(EventMask::RAW_AXIS));
}

#[test]
fn combine_raw_events() {
    let mask = EventMask::RAW_AXIS | EventMask::RAW_BUTTON;
    assert!(mask.contains(EventMask::RAW_AXIS));
    assert!(mask.contains(EventMask::RAW_BUTTON));
    assert!(!mask.contains(EventMask::MOTION));
}

#[test]
fn empty_mask_contains_nothing() {
    let mask = EventMask::empty();
    assert!(!mask.contains(EventMask::MOTION));
    assert!(!mask.contains(EventMask::BUTTON));
    assert!(!mask.contains(EventMask::DEV));
    assert!(!mask.contains(EventMask::CFG));
    assert!(!mask.contains(EventMask::RAW_AXIS));
    assert!(!mask.contains(EventMask::RAW_BUTTON));
}

// ─── from_bits_truncate tests ───────────────────────────────────────────────

#[test]
fn from_bits_truncate_valid() {
    let mask = EventMask::from_bits_truncate(0x03);
    assert_eq!(mask, EventMask::MOTION | EventMask::BUTTON);
}

#[test]
fn from_bits_truncate_high_bits_truncated() {
    // Bits beyond the defined ones should be truncated
    let mask = EventMask::from_bits_truncate(0xFFFF);
    assert_eq!(mask, EventMask::ALL);
}

#[test]
fn from_bits_truncate_zero() {
    let mask = EventMask::from_bits_truncate(0);
    assert_eq!(mask, EventMask::empty());
}

// ─── Equality and copy tests ────────────────────────────────────────────────

#[test]
fn event_mask_is_copy() {
    let m = EventMask::MOTION;
    let m2 = m;
    assert_eq!(m, m2);
}

#[test]
fn event_mask_is_debug() {
    let debug = format!("{:?}", EventMask::MOTION);
    assert!(debug.contains("MOTION"));
}

#[test]
fn event_mask_equality() {
    assert_eq!(EventMask::MOTION, EventMask::MOTION);
    assert_ne!(EventMask::MOTION, EventMask::BUTTON);
}

#[test]
fn combined_mask_equality() {
    let m1 = EventMask::MOTION | EventMask::BUTTON;
    let m2 = EventMask::MOTION | EventMask::BUTTON;
    assert_eq!(m1, m2);
}
