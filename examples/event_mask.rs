//! Event mask demonstration - shows how to configure which events spacenavd delivers.
//!
//! The event mask is a per-client filter that controls which event types are sent
//! over the socket. By default only MOTION, BUTTON, and DEV events are delivered
//! (the `DEFAULT` preset). This example shows how to read, modify, and apply the mask
//! to enable additional event types like RAW_AXIS, RAW_BUTTON, and CFG.
//!
//! Run with: `cargo run --example event_mask`

use spnav_rs::{EventMask, SpnavClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SpnavClient::open().await?;
    let proto = client.protocol_version();
    println!("Connected to spacenavd (protocol v{proto})");

    if proto == 0 {
        eprintln!("Error: event mask API requires protocol v1");
        eprintln!("Try restarting spacenavd with: spacenavd -d");
        return Ok(());
    }

    // --- 1. Read the current mask ---
    let current = client.get_event_mask().await?;
    println!("\n--- Current event mask ---");
    println!("  Raw value: 0x{:04x}", current.bits());
    println!("  Flags:     {current:?}");
    print_flag("MOTION",     current, EventMask::MOTION);
    print_flag("BUTTON",     current, EventMask::BUTTON);
    print_flag("DEV",        current, EventMask::DEV);
    print_flag("CFG",        current, EventMask::CFG);
    print_flag("RAW_AXIS",   current, EventMask::RAW_AXIS);
    print_flag("RAW_BUTTON", current, EventMask::RAW_BUTTON);

    // --- 2. Show the predefined presets ---
    println!("\n--- Predefined presets ---");
    println!("  INPUT   (0x{:04x}): {:?}", EventMask::INPUT.bits(),   EventMask::INPUT);
    println!("  DEFAULT (0x{:04x}): {:?}", EventMask::DEFAULT.bits(), EventMask::DEFAULT);
    println!("  ALL     (0x{:04x}): {:?}", EventMask::ALL.bits(),     EventMask::ALL);

    // --- 3. Enable all event types ---
    //
    // You can set any combination using the bitflags operators:
    //
    //   // Union of individual flags
    //   let mask = EventMask::MOTION | EventMask::BUTTON | EventMask::RAW_AXIS;
    //
    //   // Add a flag to an existing mask
    //   let mask = current | EventMask::CFG;
    //
    //   // Remove a flag from an existing mask
    //   let mask = current - EventMask::DEV;
    //
    //   // Use a preset directly
    //   let mask = EventMask::ALL;

    println!("\nSetting event mask to ALL (enable every event type)...");
    client.set_event_mask(EventMask::ALL).await?;

    let verified = client.get_event_mask().await?;
    println!("Verified mask: {verified:?}");

    // --- 4. Listen for events ---
    println!("\nWaiting for events (Ctrl+C to exit)...");
    println!("Try moving the device, pressing buttons, or disconnecting/reconnecting it.\n");

    loop {
        let event = client.wait_event().await?;
        match &event {
            spnav_rs::event::SpnavEvent::Motion(m) => {
                println!(
                    "[MOTION] x={:+4} y={:+4} z={:+4}  rx={:+4} ry={:+4} rz={:+4}  dt={}ms",
                    m.x, m.y, m.z, m.rx, m.ry, m.rz, m.period
                );
            }
            spnav_rs::event::SpnavEvent::Button(b) => {
                println!(
                    "[BUTTON] btn={} {}",
                    b.bnum,
                    if b.press { "pressed" } else { "released" }
                );
            }
            spnav_rs::event::SpnavEvent::Device(d) => {
                println!(
                    "[DEV] {:?} id={} type={:?} usb={:04x}:{:04x}",
                    d.op, d.id, d.devtype, d.usb_vendor, d.usb_product
                );
            }
            spnav_rs::event::SpnavEvent::Config(c) => {
                println!("[CFG] cfg=0x{:04x} data={:?}", c.cfg, c.data);
            }
            spnav_rs::event::SpnavEvent::RawAxis(a) => {
                println!("[RAW_AXIS] idx={} value={}", a.idx, a.value);
            }
            spnav_rs::event::SpnavEvent::RawButton(b) => {
                println!(
                    "[RAW_BUTTON] btn={} {}",
                    b.bnum,
                    if b.press { "pressed" } else { "released" }
                );
            }
        }
    }
}

fn print_flag(name: &str, mask: EventMask, flag: EventMask) {
    let on = if mask.contains(flag) { "✓" } else { "✗" };
    println!("  {on} {name}");
}
