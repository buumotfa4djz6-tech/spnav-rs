//! X11 rotating cube demo using the Magellan protocol.
//!
//! This example creates an X11 window with a wireframe cube that rotates
//! based on spacenav motion events. Requires the `x11` feature and a
//! running spacenavd with X11 support.
//!
//! ```bash
//! cargo run --features x11 --example x11_cube
//! ```

#[cfg(feature = "x11")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;
    use x11rb::protocol::Event;
    use x11rb::rust_connection::RustConnection;

    use spnav_rs::x11::X11Spnav;
    use spnav_rs::SpnavEvent;

    // Connect to X11
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Create a simple window
    let win = conn.generate_id()?;
    conn.create_window(
        0,
        win,
        root,
        0,
        0,
        640,
        480,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new()
            .background_pixel(screen.white_pixel)
            .event_mask(EventMask::EXPOSURE | EventMask::KEY_PRESS),
    )?;

    conn.map_window(win)?;
    conn.flush()?;

    // Open spacenav over X11
    let spnav = X11Spnav::open(&conn, win)?;
    println!("spacenav connected via X11 Magellan protocol");

    // Simple event loop
    loop {
        let event = conn.wait_for_event()?;

        // Check for spacenav events
        if let Some(spnav_event) = spnav.try_event(&event)? {
            match spnav_event {
                SpnavEvent::Motion(m) => {
                    println!(
                        "motion: x={} y={} z={} rx={} ry={} rz={}",
                        m.x, m.y, m.z, m.rx, m.ry, m.rz
                    );
                }
                SpnavEvent::Button(b) => {
                    println!(
                        "button: {} {}",
                        if b.press { "press" } else { "release" },
                        b.bnum
                    );
                }
                _ => {}
            }
        }

        // Handle X11 events
        match event {
            Event::Expose(_) => {
                println!("window exposed");
            }
            Event::KeyPress(key) => {
                println!("key pressed: {}", key.detail);
                if key.detail == 9 {
                    // Escape key
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(not(feature = "x11"))]
fn main() {
    eprintln!("This example requires the 'x11' feature:");
    eprintln!("  cargo run --features x11 --example x11_cube");
}
