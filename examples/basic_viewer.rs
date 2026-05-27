//! Basic event viewer - prints all spacenav events as they arrive.
//!
//! Run with: `cargo run --example basic_viewer`

use spnav_rs::SpnavClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SpnavClient::open().await?;
    println!("Connected to spacenavd (protocol v{})", client.protocol_version());
    println!("Waiting for events... (Ctrl+C to exit)");

    loop {
        let event = client.wait_event().await?;
        match &event {
            spnav_rs::event::SpnavEvent::Motion(m) => {
                println!(
                    "Motion: x={:+4} y={:+4} z={:+4}  rx={:+4} ry={:+4} rz={:+4}  period={}ms",
                    m.x, m.y, m.z, m.rx, m.ry, m.rz, m.period
                );
            }
            spnav_rs::event::SpnavEvent::Button(b) => {
                println!("Button {} {}", b.bnum, if b.press { "pressed" } else { "released" });
            }
            spnav_rs::event::SpnavEvent::Device(d) => {
                println!(
                    "Device: {:?} (id={}, usb={:04x}:{:04x})",
                    d.op, d.id, d.usb_vendor, d.usb_product
                );
            }
            spnav_rs::event::SpnavEvent::Config(c) => {
                println!("Config: cfg={} data={:?}", c.cfg, c.data);
            }
            spnav_rs::event::SpnavEvent::RawAxis(a) => {
                println!("RawAxis: idx={} value={}", a.idx, a.value);
            }
            spnav_rs::event::SpnavEvent::RawButton(b) => {
                println!("RawButton: btn={} {}", b.bnum, if b.press { "pressed" } else { "released" });
            }
        }
    }
}
