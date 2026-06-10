# Agent skills

Configuration consumed by the Matt Pocock engineering skills (`triage`, `to-issues`, `to-prd`, `qa`, `improve-codebase-architecture`, `diagnose`, `tdd`, etc.).

## Issue tracker

GitHub Issues (via `gh` CLI) at `buumotfa4djz6-tech/spnav-rs`. See `docs/agents/issue-tracker.md`.

## Triage labels

Default canonical roles (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

## Domain docs

Single-context layout: `CONTEXT.md` + `docs/adr/` at repo root. See `docs/agents/domain.md`.

## Test coverage — known blind spots

The test suite has ~130 passing tests, all of which exercise crate logic (tautological tests and lib.rs duplicates were removed; gap-fill tests were added for the areas below).

Areas now covered by integration tests:

- `SpnavClient::open()` handshake — `tests/client_handshake.rs`
- `find_socket()` path discovery (env var override, fallback) — `tests/socket_discovery.rs`
- Event-mask request encoding and daemon-failure propagation — `tests/event_mask_enforcement.rs`
- Client event-loop behavior (event forwarding, request/response interleaving, connection close, poll/wait) — `tests/client_event_loop.rs`

A shared `SpnavDaemonSimulator` in `tests/common/mod.rs` stands up a fake spacenavd on a temp UNIX socket for any test that needs one. See `docs/adr/0002-mock-daemon.md`.

Areas still with no real coverage:

- `X11Spnav::try_event` — the Magellan decode. Requires either a real X server (`Xvfb`) or a small API refactor to extract the decode as a pure function. Deferred; the UNIX-side decode is well-covered and the Magellan protocol is parallel.
- `PositionRot` view-vs-model under combined rotation + translation — marginal; existing per-axis tests give enough confidence.
- `/etc/spnavrc` branch of `find_socket()` — requires a filesystem fixture or root; left to manual verification.

Before relying on the suite as a regression net for a non-trivial change in one of the uncovered areas, add at least one real test there first.
