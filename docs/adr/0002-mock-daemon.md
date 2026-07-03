# Mock daemon: shared simulator in tests/common/

Tests that need a spacenavd at the other end of a UNIX socket use a single `SpnavDaemonSimulator` in `tests/common/mod.rs` rather than spinning up an ad-hoc `UnixListener` per test, recording/replaying daemon traces, or refactoring `SpnavClient` to take an injectable transport.

## Context

Gap-filling the test suite (client handshake, event-mask request/response, event-loop behavior) required ~15 tests that all talk to a fake daemon. Each test needs the same sequence: bind a temp socket, accept a connection, run the 4-byte handshake advertising v1, then relay frames full-duplex. Without a shared helper, every test would re-implement that sequence.

## Considered options

- **Per-test ad-hoc servers.** Zero upfront cost; high per-test cost. With 15+ tests the duplication dominates.
- **Shared simulator** (chosen). One `SpnavDaemonSimulator` struct in `tests/common/`. Binds a `tempfile` socket, spawns reader+writer tasks, auto-does the handshake. Tests interact via `sim.send(frame)` / `sim.recv().await`.
- **Recorded-trace replay.** Needs a real `spacenavd` to capture from first, and traces go stale when the protocol evolves. Not viable in CI.
- **Injectable transport trait.** Cleanest abstraction but widens the pub API solely for tests — rejected per ADR-0001's principle.

The simulator itself is untested crate code, but it's built entirely on the `ReqResp` / `to_bytes` / `from_bytes` primitives that `tests/protocol.rs` already covers, so a bug in the simulator would surface as protocol-test failures rather than silent misbehavior.

Frame encoders (`encode_motion_frame`, `encode_button_frame`, etc.) also live in `tests/common/` and are shared with `tests/protocol.rs`, which used to have its own private copies.
