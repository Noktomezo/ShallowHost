default: check

# Run dev server with auto-reload on file changes
dev:
    watchexec -r -e rs -- cargo run

# Preview an available update without downloading, installing, or restarting
dev-update:
    SHALLOWHOST_MOCK_UPDATE=1 watchexec -r -e rs -- cargo run

# Build optimized release binary and compress with UPX (--best --lzma) via xtask
build:
    cargo run --package xtask --release -- build

# Run cargo check across all targets
check:
    cargo check --all-targets

# Run unit and integration tests
test:
    cargo test --all-targets

# Run clippy for strict lint checks
clippy:
    cargo clippy --all-targets -- -D warnings

# Format check
fmt:
    cargo fmt --all --check

# Full strict verification (check + test + clippy + fmt)
strict: check test clippy fmt

# Clean build artifacts
clean:
    cargo clean
