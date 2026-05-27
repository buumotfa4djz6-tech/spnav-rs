//! X11 (Magellan protocol) support for compatibility with 3dxsrv.
//!
//! This module implements the Magellan X11 protocol used by spacenavd and the
//! proprietary 3Dconnexion 3dxsrv driver. Applications using this protocol
//! receive spacenav events as X11 ClientMessage events, which can be
//! integrated into an existing X11 event loop.
//!
//! # Protocol overview
//!
//! The daemon advertises its window via the `CommandEvent` atom on the root
//! window. The client registers its application window by sending a
//! ClientMessage with `CMD_APP_WINDOW` to the daemon window. After that,
//! motion and button events arrive as ClientMessage events with the
//! `MotionEvent`, `ButtonPressEvent`, or `ButtonReleaseEvent` atoms.
//!
//! # Example
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use x11rb::rust_connection::RustConnection;
//! use x11rb::connection::Connection;
//! use spnav_rs::x11::X11Spnav;
//!
//! let (conn, screen_num) = RustConnection::connect(None)?;
//! let screen = &conn.setup().roots[screen_num];
//! let win = conn.generate_id()?;
//!
//! // Create your window...
//!
//! let spnav = X11Spnav::open(&conn, win)?;
//!
//! // In your X11 event loop:
//! // if let Some(event) = spnav.try_event(&x_event)? {
//! //     println!("Spnav event: {:?}", event);
//! // }
//! # Ok(())
//! # }
//! ```

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

use crate::error::Result;
use crate::event::{ButtonEvent, MotionEvent, SpnavEvent};

/// Magellan command codes sent via ClientMessage format 16.
const CMD_APP_WINDOW: i16 = 27695;
const CMD_APP_SENS: i16 = 27696;

/// X11-based spacenav client using the Magellan protocol.
///
/// This client communicates with spacenavd (or 3dxsrv) via X11
/// ClientMessage events, providing compatibility with the proprietary
/// 3Dconnexion driver protocol.
///
/// Unlike the socket-based [`SpnavClient`](crate::SpnavClient), this
/// implementation only supports motion and button events plus sensitivity
/// changes. Device queries and the configuration API are not available
/// over the X11 protocol.
pub struct X11Spnav<C: Connection> {
    conn: C,
    app_win: Window,
    daemon_win: Window,
    motion_atom: Atom,
    button_press_atom: Atom,
    button_release_atom: Atom,
    command_atom: Atom,
}

impl<C: Connection> X11Spnav<C> {
    /// Open an X11 connection to spacenavd using the Magellan protocol.
    ///
    /// This finds the daemon window via the `CommandEvent` atom on the root
    /// window, verifies its identity, and registers `win` as the application
    /// window to receive spnav events.
    ///
    /// Returns an error if the daemon is not running or the atoms cannot be
    /// interned.
    pub fn open(conn: C, win: Window) -> Result<Self> {
        let setup = conn.setup();
        let screen = &setup.roots[0];
        let root = screen.root;

        // Intern the four Magellan protocol atoms
        let motion_atom = intern_atom(&conn, b"MotionEvent")?;
        let button_press_atom = intern_atom(&conn, b"ButtonPressEvent")?;
        let button_release_atom = intern_atom(&conn, b"ButtonReleaseEvent")?;
        let command_atom = intern_atom(&conn, b"CommandEvent")?;

        // Find the daemon window
        let daemon_win = find_daemon_window(&conn, root, command_atom)?;

        let this = Self {
            conn,
            app_win: win,
            daemon_win,
            motion_atom,
            button_press_atom,
            button_release_atom,
            command_atom,
        };

        // Register our window with the daemon
        this.send_app_window(win)?;

        Ok(this)
    }

    /// Change the application window that receives spnav events.
    ///
    /// Note: when using spacenavd (the free daemon), multiple windows can
    /// be registered. The proprietary 3dxsrv may only support one window
    /// at a time.
    pub fn set_window(&self, win: Window) -> Result<()> {
        self.send_app_window(win)
    }

    /// Set the application sensitivity via X11 ClientMessage.
    ///
    /// This sends a `CMD_APP_SENS` command to the daemon window with the
    /// sensitivity value encoded as a float in the message data.
    pub fn set_sensitivity(&self, sens: f32) -> Result<()> {
        let bits = sens.to_bits();
        let lo = (bits & 0xFFFF) as u16;
        let hi = ((bits >> 16) & 0xFFFF) as u16;

        // Format 16 ClientMessage uses [u16; 10]
        let mut data = [0u16; 10];
        data[0] = lo;
        data[1] = hi;
        data[2] = CMD_APP_SENS as u16;

        let event = ClientMessageEvent {
            response_type: 33, // ClientMessage
            format: 16,
            sequence: 0,
            window: self.app_win,
            type_: self.command_atom,
            data: ClientMessageData::from(data),
        };

        self.conn.send_event(
            false,
            self.daemon_win,
            EventMask::NO_EVENT,
            event,
        )?.ignore_error();

        self.conn.flush()?;
        Ok(())
    }

    /// Try to decode an X11 event as a spnav event.
    ///
    /// Returns `Some(SpnavEvent)` if the event is a Magellan motion or button
    /// event, or `None` if it's not a spnav event.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn example<C: x11rb::connection::Connection>(spnav: &spnav_rs::x11::X11Spnav<C>) -> Result<(), Box<dyn std::error::Error>> {
    /// use x11rb::protocol::Event;
    ///
    /// // In your event loop:
    /// // let event = conn.wait_for_event()?;
    /// // if let Some(spnav_event) = spnav.try_event(&event)? {
    /// //     println!("Got: {:?}", spnav_event);
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_event(&self, event: &Event) -> Result<Option<SpnavEvent>> {
        let cev = match event {
            Event::ClientMessage(cev) => cev,
            _ => return Ok(None),
        };

        // Check format 16 (required for Magellan protocol)
        if cev.format != 16 {
            return Ok(None);
        }

        let data = cev.data.as_data16();

        if cev.type_ == self.motion_atom {
            // Motion event: data[2..8] = x, y, z, rx, ry, rz
            // data[8] = period
            Ok(Some(SpnavEvent::Motion(MotionEvent {
                x: data[2] as i16 as i32,
                y: data[3] as i16 as i32,
                z: data[4] as i16 as i32,
                rx: data[5] as i16 as i32,
                ry: data[6] as i16 as i32,
                rz: data[7] as i16 as i32,
                period: data[8] as u32,
            })))
        } else if cev.type_ == self.button_press_atom {
            // Button press: data[2] = button number
            Ok(Some(SpnavEvent::Button(ButtonEvent {
                press: true,
                bnum: data[2] as i16 as i32,
            })))
        } else if cev.type_ == self.button_release_atom {
            // Button release: data[2] = button number
            Ok(Some(SpnavEvent::Button(ButtonEvent {
                press: false,
                bnum: data[2] as i16 as i32,
            })))
        } else {
            Ok(None)
        }
    }

    /// Get a reference to the underlying X11 connection.
    pub fn connection(&self) -> &C {
        &self.conn
    }

    /// Get the daemon window ID.
    pub fn daemon_window(&self) -> Window {
        self.daemon_win
    }

    /// Get the application window ID.
    pub fn app_window(&self) -> Window {
        self.app_win
    }

    /// Register the application window with the daemon.
    fn send_app_window(&self, win: Window) -> Result<()> {
        let mut data = [0u16; 10];
        data[0] = ((win >> 16) & 0xFFFF) as u16;
        data[1] = (win & 0xFFFF) as u16;
        data[2] = CMD_APP_WINDOW as u16;

        let event = ClientMessageEvent {
            response_type: 33,
            format: 16,
            sequence: 0,
            window: win,
            type_: self.command_atom,
            data: ClientMessageData::from(data),
        };

        self.conn.send_event(
            false,
            self.daemon_win,
            EventMask::NO_EVENT,
            event,
        )?.ignore_error();

        self.conn.flush()?;
        Ok(())
    }
}

/// Intern an atom by name, returning an error if it doesn't exist.
fn intern_atom<C: Connection>(conn: &C, name: &[u8]) -> Result<Atom> {
    let reply = conn.intern_atom(false, name)?.reply()?;
    Ok(reply.atom)
}

/// Find the daemon window by reading the CommandEvent property from the root
/// window and verifying the window's WM_NAME is "Magellan Window".
fn find_daemon_window<C: Connection>(
    conn: &C,
    root: Window,
    command_atom: Atom,
) -> Result<Window> {
    // Read the CommandEvent property from root window.
    // The property contains the daemon's Window ID.
    let reply = conn.get_property(
        false,
        root,
        command_atom,
        x11rb::NONE,
        0,
        1,
    )?.reply()?;

    if reply.value_len == 0 || reply.value.is_empty() {
        return Err(crate::error::Error::SocketNotFound);
    }

    // The property value is a Window ID (32-bit)
    let daemon_win = if reply.format == 32 && reply.value.len() >= 4 {
        u32::from_ne_bytes([
            reply.value[0],
            reply.value[1],
            reply.value[2],
            reply.value[3],
        ])
    } else {
        return Err(crate::error::Error::Protocol(format!(
            "unexpected CommandEvent property format: {}",
            reply.format
        )));
    };

    // Verify the window's WM_NAME is "Magellan Window"
    let wm_name_atom = intern_atom(conn, b"WM_NAME")?;
    let name_reply = conn.get_property(
        false,
        daemon_win,
        wm_name_atom,
        x11rb::NONE,
        0,
        64,
    )?.reply()?;

    let name = String::from_utf8_lossy(&name_reply.value);
    if name != "Magellan Window" {
        return Err(crate::error::Error::Protocol(format!(
            "daemon window has unexpected WM_NAME: {:?}",
            name
        )));
    }

    Ok(daemon_win)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_constants() {
        // Verify command constants match C implementation
        assert_eq!(CMD_APP_WINDOW, 27695);
        assert_eq!(CMD_APP_SENS, 27696);
    }
}
