# justfile — spnav-rs quick commands
# https://github.com/casey/just

# List all available commands
default:
    @just --list

# ── Build ──────────────────────────────────────────────

# Build the library (default features)
build:
    cargo build

# Build with all features enabled
build-all:
    cargo build --all-features

# Build in release mode
release:
    cargo build --release

# Build release with all features
release-all:
    cargo build --release --all-features

# ── Quality ────────────────────────────────────────────

# Run cargo check
check:
    cargo check --all-features

# Run tests
test:
    cargo test --all-features

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --all-features -- -D warnings

# Generate and open documentation
doc:
    cargo doc --all-features --open

# ── Examples ───────────────────────────────────────────

# Run basic_viewer example
viewer:
    cargo run --example basic_viewer --all-features

# Run device_info example
info:
    cargo run --example device_info --all-features

# Run config_tool example
config:
    cargo run --example config_tool --all-features

# Run x11_viewer example (requires x11 feature)
x11:
    cargo run --example x11_viewer --all-features

# Run x11_cube example (requires x11 feature)
cube:
    cargo run --example x11_cube --all-features

# Run a named example: just example <name>
example name:
    cargo run --example {{name}} --all-features

# ── Maintenance ────────────────────────────────────────

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree

# ── CI ─────────────────────────────────────────────────

# Run all CI checks (fmt + clippy + test)
ci: fmt-check lint test
    @echo "✅ All CI checks passed"

# Dry-run publish
publish-dry:
    cargo publish --dry-run --all-features
