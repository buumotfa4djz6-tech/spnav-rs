# Context

Rust binding for spacenavd — the daemon that reads 3Dconnexion 6DOF devices (SpaceMouse, SpaceNavigator, etc.) and exposes events over UNIX sockets or X11.

Not tied to any GUI framework. Provides raw event streams; consumers decide what to do with them.

## Glossary

| Term | Meaning |
|------|---------|
| Mapping | The transformation spacenavd applies to raw axis/button values before emitting mapped events. Not any consumer-side transformation. |
| EventMask | Bitmask telling the daemon which event types to deliver. Set per-connection, not a one-time subscription. |
| PositionRot | Software accumulator integrating motion deltas into absolute position/orientation. The daemon only reports deltas; the consumer owns absolute state. |
| Protocol version | spacenavd wire protocol: v0 = basic events only; v1 = adds per-client sensitivity, event masks, device queries, config API. |
| Socket discovery | How the socket path is found: `$SPNAV_SOCKET` env → `/etc/spnavrc` config → default `/var/run/spnav.sock`. |
