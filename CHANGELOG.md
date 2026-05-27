# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-XX-XX

### Added

- Initial release of spnav-rs
- Async `SpnavClient` for UNIX domain socket communication with spacenavd
- Protocol v0 and v1 support with automatic negotiation
- Event types: Motion, Button, Device, Config, RawAxis, RawButton
- Configuration API: sensitivity, deadzone, axis mapping, button mapping, LED control, grab mode
- `EventMask` for filtering event types
- `PositionRot` utility for accumulating motion events into position/quaternion
- Model and view matrix generation (`to_model_matrix()`, `to_view_matrix()`)
- `Stream` interface via `subscribe()` method
- `x11` feature: X11 Magellan protocol support via `x11rb`
  - `X11Spnav` client for X11-based applications
  - Compatible with both spacenavd and 3dxsrv
- Examples: `basic_viewer`, `device_info`, `config_tool`
- Comprehensive test suite (24 unit tests)
- `justfile` for common development tasks
- CI workflow with fmt, clippy, test, and doc checks

### Supported Devices

All devices supported by spacenavd, including:
- SpaceMouse (Classic, Pro, Pro Wireless, Enterprise, Compact, Module, Wireless)
- SpaceNavigator (USB, Notebooks)
- SpacePilot, SpacePilot Pro
- SpaceExplorer
- SpaceTraveller
- SpaceBall (2003, 3003, 4000, 5000)
- CadMan
- NuLOOQ

[0.1.0]: https://github.com/libspnav/spnav-rs/releases/tag/v0.1.0
