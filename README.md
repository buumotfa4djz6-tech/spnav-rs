# spnav-rs

[![CI](https://github.com/FreeSpacenav/libspnav/actions/workflows/ci.yml/badge.svg)](https://github.com/FreeSpacenav/libspnav/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/spnav-rs.svg)](https://crates.io/crates/spnav-rs)
[![docs.rs](https://docs.rs/spnav-rs/badge.svg)](https://docs.rs/spnav-rs)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue.svg)](#msrv)

Async Rust client library for 6DOF input devices (SpaceMouse, SpaceNavigator, etc.)
communicating with [spacenavd](http://spacenav.sourceforge.net/) via UNIX domain sockets
or the X11 Magellan protocol.

## Features

- **Async-first API** — Built on [tokio](https://tokio.rs/) for non-blocking event handling
- **UNIX socket protocol** — Direct communication with spacenavd daemon
- **X11 Magellan protocol** — Compatible with both spacenavd and 3Dconnexion's 3dxsrv (`x11` feature)
- **Full event support** — Motion, button, device, config, raw axis, and raw button events
- **Configuration API** — Sensitivity, deadzone, axis mapping, button mapping, LED control
- **Stream interface** — `futures::Stream` support for reactive programming
- **Position/rotation utilities** — Built-in `PositionRot` accumulator with model/view matrix generation

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
spnav-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic usage (UNIX socket)

```rust
use spnav_rs::{SpnavClient, SpnavEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SpnavClient::open().await?;
    println!("Connected to spacenavd (protocol v{})", client.protocol_version());

    loop {
        let event = client.wait_event().await?;
        match event {
            SpnavEvent::Motion(m) => {
                println!("Motion: x={} y={} z={}", m.x, m.y, m.z);
            }
            SpnavEvent::Button(b) => {
                println!("Button {} {}", b.bnum, if b.press { "pressed" } else { "released" });
            }
            _ => {}
        }
    }
}
```

### X11 Magellan protocol

Enable the `x11` feature for X11 compatibility:

```toml
[dependencies]
spnav-rs = { version = "0.1", features = ["x11"] }
```

```rust,no_run
use x11rb::rust_connection::RustConnection;
use spnav_rs::x11::X11Spnav;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    // Create your X11 window here...
    let win = conn.generate_id()?;

    let spnav = X11Spnav::open(&conn, win)?;

    // In your X11 event loop, decode spnav events:
    // let event = conn.wait_for_event()?;
    // if let Some(spnav_event) = spnav.try_event(&event)? {
    //     println!("Spnav: {:?}", spnav_event);
    // }

    Ok(())
}
```

## Examples

```bash
# Print all events as they arrive
cargo run --example basic_viewer

# Query device information
cargo run --example device_info

# Demonstrate configuration API
cargo run --example config_tool

# X11 integration demo
cargo run --example x11_cube --features x11

# X11 viewer demo
cargo run --example x11_viewer --features x11
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `x11`   | No      | Enable X11 Magellan protocol support via `x11rb` |

## API Overview

### SpnavClient (UNIX socket)

| Method | Description |
|--------|-------------|
| `open()` | Connect to spacenavd via UNIX socket |
| `open_path(path)` | Connect to a specific socket path |
| `wait_event()` | Async wait for next event |
| `poll_event()` | Non-blocking event poll |
| `subscribe()` | Get a `Stream` of events |
| `set_sensitivity(sens)` | Set per-client sensitivity |
| `set_client_name(name)` | Set client name |
| `set_event_mask(mask)` | Filter event types |
| `dev_name()` | Get device name |
| `dev_type()` | Get device type |
| `cfg_get_sens()` | Get global sensitivity |
| `cfg_set_sens(s)` | Set global sensitivity |
| `cfg_save()` | Save configuration |

### X11Spnav (X11 protocol)

| Method | Description |
|--------|-------------|
| `open(conn, win)` | Open X11 connection to daemon |
| `set_window(win)` | Change registered window |
| `set_sensitivity(sens)` | Set sensitivity via X11 |
| `try_event(event)` | Decode X11 event as spnav event |

### Event Types

| Event | Description |
|-------|-------------|
| `SpnavEvent::Motion` | 6DOF motion data (x, y, z, rx, ry, rz, period) |
| `SpnavEvent::Button` | Button press/release with button number |
| `SpnavEvent::Device` | Device add/remove notifications |
| `SpnavEvent::Config` | Configuration change events |
| `SpnavEvent::RawAxis` | Raw axis data |
| `SpnavEvent::RawButton` | Raw button data |

### PositionRot Utility

```rust
use spnav_rs::PositionRot;

let mut pr = PositionRot::new();
// In your motion event handler:
// pr.move_obj(&motion_event);  // Object-space movement
// pr.move_view(&motion_event); // View-space movement

// Get transformation matrix for rendering
let model_matrix = pr.to_model_matrix();
let view_matrix = pr.to_view_matrix();
```

## Requirements

- **spacenavd** — The daemon must be running for socket-based connections
  ```bash
  # Install on Debian/Ubuntu
  sudo apt install spacenavd

  # Install on Arch Linux
  sudo pacman -S spacenavd

  # Start the daemon
  sudo systemctl start spacenavd
  ```

- **X11 development libraries** — Required for the `x11` feature
  ```bash
  # Debian/Ubuntu
  sudo apt install libx11-dev libxext-dev

  # Arch Linux
  sudo pacman -S libx11 libxext
  ```

## Platform Support

| Platform | Status |
|----------|--------|
| Linux    | ✅ Primary support |
| macOS    | ⚠️ Experimental (requires spacenavd build) |
| FreeBSD  | ⚠️ Experimental |

## MSRV (Minimum Supported Rust Version)

Rust **1.75** or later. The MSRV may be bumped in minor releases as needed.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run the test suite (`cargo test --all-features`)
4. Ensure formatting (`cargo fmt --all -- --check`)
5. Run clippy (`cargo clippy --all-features -- -D warnings`)
6. Commit your changes (`git commit -m 'feat: add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

BSD 3-Clause License. See [LICENSE](LICENSE) for details.

```
Copyright (C) 2007-2024 John Tsiombikas <nuclear@member.fsf.org>
Copyright (C) 2024 spnav-rs contributors
```

## Acknowledgments

This library is a Rust port of [libspnav](http://spacenav.sourceforge.net/) by
John Tsiombikas. The original C library is the reference implementation for
communicating with spacenavd.

## Related Projects

- [libspnav](http://spacenav.sourceforge.net/) — Original C library
- [spacenavd](http://spacenav.sourceforge.net/) — The spacenav daemon
- [Spacenav SDK](https://github.com/FreeSpacenav/libspnav) — Official SDK repository
