# Gearbox Relay — Task runner

# Run the fast test suite (excludes slow/ignored tests)
test:
    cargo test --workspace --exclude relay-sync-server

# Run the slow test suite (ignored tests only)
slow-test:
    cargo test --workspace --exclude relay-sync-server -- --ignored

# Lint check (format + clippy)
lint:
    cargo fmt -- --check
    cargo clippy --all-targets --workspace --exclude relay-sync-server -- -D warnings

# Fix fmt and clippy issues automatically
fix:
    cargo fmt --all
    cargo clippy --all-targets --workspace --exclude relay-sync-server --fix --allow-dirty
