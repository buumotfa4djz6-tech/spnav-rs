//! X11 event viewer - demonstrates the Magellan X11 protocol.
//!
//! This example shows how to use `X11Spnav` to receive spacenav events
//! through the X11 Magellan protocol, which is compatible with both
//! spacenavd and 3Dconnexion's 3dxsrv.
//!
//! Run with: `cargo run --example x11_viewer --features x11`
//!
//! Note: This example requires an X11 display and spacenavd (or 3dxsrv) running.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt;

use spnav_rs::x11::X11Spnav;
use spnav_rs::PositionRot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the X server
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Create a simple window
    let win = conn.generate_id()?;
    let wm_protocols = conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom;
    let wm_delete_window = conn.intern_atom(false, b"WM_DELETE_WINDOW")?.reply()?.atom;

    let win_aux = CreateWindowAux::new()
        .background_pixel(screen.white_pixel)
        .event_mask(
            EventMask::EXPOSURE
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::KEY_PRESS
                | EventMask::BUTTON_PRESS,
        );

    conn.create_window(
        32,   // depth
        win,  // window id
        root, // parent
        0,
        0, // x, y
        512,
        512, // width, height
        0,   // border width
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &win_aux,
    )?;

    // Set window title
    conn.change_property8(
        PropMode::REPLACE,
        win,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        b"spnav-rs X11 Viewer",
    )?;

    // Register for WM_DELETE_WINDOW
    conn.change_property32(
        PropMode::REPLACE,
        win,
        wm_protocols,
        AtomEnum::ATOM,
        &[wm_delete_window],
    )?;

    conn.map_window(win)?;
    conn.flush()?;

    // Open spacenav connection
    println!("Connecting to spacenavd via X11 Magellan protocol...");
    let spnav = X11Spnav::open(&conn, win)?;
    println!("Connected! Daemon window: {:?}", spnav.daemon_window());

    // Initialize position/rotation accumulator
    let mut posrot = PositionRot::new();

    println!("Waiting for events... (press Escape or close window to exit)");

    // Event loop
    loop {
        let event = conn.wait_for_event()?;

        match &event {
            Event::ClientMessage(cev) => {
                // Check for WM_DELETE_WINDOW
                if cev.type_ == wm_protocols {
                    let data = cev.data.as_data32();
                    if data[0] == wm_delete_window {
                        println!("Window close requested.");
                        break;
                    }
                }
            }
            Event::KeyPress(k) => {
                // Escape key
                if k.detail == 9 {
                    println!("Escape pressed.");
                    break;
                }
            }
            _ => {}
        }

        // Try to decode as spnav event
        if let Some(spnav_event) = spnav.try_event(&event)? {
            match &spnav_event {
                spnav_rs::SpnavEvent::Motion(m) => {
                    posrot.move_obj(m);
                    println!(
                        "Motion: x={:+4} y={:+4} z={:+4}  rx={:+4} ry={:+4} rz={:+4}  period={}ms",
                        m.x, m.y, m.z, m.rx, m.ry, m.rz, m.period
                    );
                    println!(
                        "  Position: ({:.3}, {:.3}, {:.3})",
                        posrot.pos.x, posrot.pos.y, posrot.pos.z
                    );
                }
                spnav_rs::SpnavEvent::Button(b) => {
                    println!(
                        "Button {} {}",
                        b.bnum,
                        if b.press { "pressed" } else { "released" }
                    );
                    // Reset position on button 0 press
                    if b.press && b.bnum == 0 {
                        posrot = PositionRot::new();
                        println!("  Position reset!");
                    }
                }
                _ => {
                    println!("Other event: {:?}", spnav_event);
                }
            }
        }
    }

    println!("Done.");
    Ok(())
}
