//! Configuration tool - demonstrates the spacenavd configuration API.
//!
//! Shows how to set sensitivity, deadzone, axis mapping, and save/restore config.
//! Run with: `cargo run --example config_tool`

use spnav_rs::{EventMask, SpnavClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SpnavClient::open().await?;
    let proto = client.protocol_version();
    println!("Connected to spacenavd (protocol v{proto})");

    if proto == 0 {
        eprintln!("Error: configuration API requires protocol v1");
        eprintln!("Try restarting spacenavd with: spacenavd -d");
        return Ok(());
    }

    // Show current sensitivity
    let sens = client.cfg_get_sens().await?;
    println!("Current sensitivity: {:.2}", sens);

    // Set a new sensitivity (does NOT persist)
    let new_sens = 2.0;
    println!("Setting per-client sensitivity to {new_sens}...");
    client.cfg_set_sens(new_sens).await?;
    let verified = client.cfg_get_sens().await?;
    println!("Verified sensitivity: {:.2}", verified);

    // Show deadzone for axis 0
    println!("\nDeadzone for axis 0: {}", client.cfg_get_deadzone(0).await?);

    // Show axis mapping
    println!("\nAxis mapping (dev_axis -> map):");
    for axis in 0..6 {
        let map = client.cfg_get_axismap(axis).await?;
        println!("  axis {axis} -> {map}");
    }

    // Show button mapping
    let n_buttons = client.dev_buttons().await.unwrap_or(2);
    println!("\nButton mapping (dev_button -> map):");
    for btn in 0..n_buttons {
        let map = client.cfg_get_bnmap(btn as i32).await?;
        println!("  button {btn} -> {map}");
    }

    // Demonstrate config save/restore
    println!("\n--- Config management ---");
    println!("To reset all settings to defaults, call: client.cfg_reset().await");
    println!("To save current settings persistently: client.cfg_save().await");
    println!("To restore last-saved settings: client.cfg_restore().await");

    // Demonstrate event mask
    let mask = client.get_event_mask().await?;
    println!("\nCurrent event mask: {mask:?}");
    println!(
        "  Motion: {}, Button: {}, Device: {}, RawAxis: {}",
        mask.contains(EventMask::MOTION),
        mask.contains(EventMask::BUTTON),
        mask.contains(EventMask::DEV),
        mask.contains(EventMask::RAW_AXIS),
    );

    Ok(())
}
