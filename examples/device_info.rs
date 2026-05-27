//! Device info - queries and displays device information from spacenavd.
//!
//! Run with: `cargo run --example device_info`

use spnav_rs::{LedState, SpnavClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SpnavClient::open().await?;
    let proto = client.protocol_version();
    println!("Connected to spacenavd (protocol v{proto})");

    if proto == 0 {
        eprintln!("Warning: protocol v0 has limited device query support");
        return Ok(());
    }

    // Device information
    match client.dev_name().await {
        Ok(name) => println!("Device name:    {name}"),
        Err(e) => println!("Device name:    <error: {e}>"),
    }
    match client.dev_path().await {
        Ok(path) => println!("Device path:    {path}"),
        Err(e) => println!("Device path:    <error: {e}>"),
    }
    match client.dev_type().await {
        Ok(ty) => println!("Device type:    {ty:?}"),
        Err(e) => println!("Device type:    <error: {e}>"),
    }
    match client.dev_buttons().await {
        Ok(n) => println!("Buttons:        {n}"),
        Err(e) => println!("Buttons:        <error: {e}>"),
    }
    match client.dev_axes().await {
        Ok(n) => println!("Axes:           {n}"),
        Err(e) => println!("Axes:           <error: {e}>"),
    }
    match client.dev_usbid().await {
        Ok((vid, pid)) => println!("USB ID:         {:04x}:{:04x}", vid, pid),
        Err(e) => println!("USB ID:         <error: {e}>"),
    }

    // Current configuration
    println!("\n--- Configuration ---");
    match client.cfg_get_sens().await {
        Ok(s) => println!("Sensitivity:    {:.2}", s),
        Err(e) => println!("Sensitivity:    <error: {e}>"),
    }
    match client.cfg_get_invert().await {
        Ok(inv) => println!("Invert axes:    0b{:06b}", inv),
        Err(e) => println!("Invert axes:    <error: {e}>"),
    }
    match client.cfg_get_swapyz().await {
        Ok(swap) => println!("Swap Y/Z:       {}", swap),
        Err(e) => println!("Swap Y/Z:       <error: {e}>"),
    }
    match client.cfg_get_led().await {
        Ok(LedState::Off) => println!("LED:            off"),
        Ok(LedState::On) => println!("LED:            on"),
        Ok(LedState::Auto) => println!("LED:            auto"),
        Err(e) => println!("LED:            <error: {e}>"),
    }

    // Event mask
    match client.get_event_mask().await {
        Ok(mask) => println!("Event mask:     {mask:?}"),
        Err(e) => println!("Event mask:     <error: {e}>"),
    }

    // Set client name
    if let Err(e) = client.set_client_name("spnav-rs/device_info").await {
        eprintln!("Failed to set client name: {e}");
    }

    Ok(())
}
