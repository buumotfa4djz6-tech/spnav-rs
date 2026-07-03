# Test organization: integration in tests/, unit tests in owning module

Pub-API tests live in `tests/` as integration tests. Unit tests for private state (e.g. helpers that aren't exposed outside their module) live as `#[cfg(test)] mod tests` inside the owning `src/<module>.rs`. We do not add private-reach `pub(crate)` shims or trait abstractions solely to make a unit testable from `tests/`.

## Context

The crate has two obvious places for tests. Early drafts had a mix: some tests in `tests/`, some duplicated in `src/lib.rs`'s inline `mod tests`, and some inline in per-module files. When we cleaned the suite up we had to pick a convention for where future tests go, especially for internal state like `EventLoopState` in `src/client.rs`.

## Considered options

- **Integration tests only.** Everything in `tests/`. Forces test-reachable APIs to be pub, which widens the surface for testability's sake.
- **Idiomatic Rust split** (chosen). Pub-API tests in `tests/`; unit tests for private internals in the owning module. The split follows visibility: if a test can exercise the behavior through the pub API, put it in `tests/`. If it needs private access, put it in the module.
- **All inline.** Every test in `src/`. Loses the separation between "tests any consumer of the crate could write" and "tests that need to know the internals."

We picked the split because `EventLoopState` (the client's internal select-loop) is observable through `SpnavClient`'s pub API in every case we've encountered so far, which means `tests/` handles it without any visibility changes. The carve-out for inline unit tests is there for the day we find a case that genuinely needs private access — we'd rather not design the pub API around test needs.
