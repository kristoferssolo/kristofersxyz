export RUSTC_WRAPPER :=  env("RUSTC_WRAPPER", "sccache")
export RUST_LOG := env("RUST_LOG", "kristofersxyz=debug")

set shell := ["bash", "-euo", "pipefail", "-c"]

# List available recipes
default:
    @just --list

alias b := build
alias c := check
alias d := docs
alias f := fmt
alias r := run
alias t := test

[group("build")]
build:
    cargo leptos build --release

# Run all checks (fmt, feature matrix, clippy, docs, test)
[group("dev")]
check: fmt features clippy sleek docs test

# Check the library configuration used by editors and plain Cargo commands
[group("dev")]
features:
    cargo check --no-default-features

# Run the development server
[group("run")]
run:
    cargo leptos watch

# Format code
[group("dev")]
fmt:
    cargo fmt --all

# Run clippy
[group("dev")]
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format sql
[group("dev")]
sleek:
    sleek migrations/*.sql

# Build documentation
[group("dev")]
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Run tests with nextest
[group("dev")]
test:
    cargo nextest run --no-default-features --features ssr
    RUST_TEST_THREADS=1 cargo leptos test

# Check dependency advisories, licenses, sources, and bans against deny.toml.
# Kept out of `just check` because it fetches the RustSec advisory database.
[group("dev")]
security:
    cargo deny check

# Measure the current Argon2 policy on production-equivalent hardware
[group("dev")]
benchmark-auth:
    cargo bench --no-default-features --features ssr,benchmarks --bench authentication

# Clean build artifacts
[group("dev")]
clean:
    cargo clean

[group("dev")]
setup:
    cargo install cargo-nextest sccache cargo-deny

# Serve the release build
serve: build
    ./target/release/kristofersxyz

# Run end-to-end tests
end2end:
    cd end2end && npx playwright test

# CI pipeline
ci:
    just check
    just end2end

# Add a new migration
[group("db")]
migrate-create NAME:
    sqlx migrate add {{NAME}}

# Run database migrations
[group("db")]
migrate:
    sqlx migrate run

# Revert the last database migration
[group("db")]
migrate-revert:
    sqlx migrate revert

# Reset the database
[group("db")]
db-reset:
    sqlx database drop -y
    sqlx database create
    just migrate

# Load the versioned portfolio content into the local database
[group("db")]
seed:
    sqlx database create
    just migrate
    sqlite3 data/portfolio.db < seeds/portfolio.sql

# Copy the running database to DESTINATION, which must not already exist
[group("db")]
db-backup DESTINATION:
    #!/usr/bin/env bash
    set -euo pipefail
    destination="{{ DESTINATION }}"
    mkdir --parents "$(dirname -- "$destination")"
    cargo run --quiet --no-default-features --features ssr -- backup "$destination"

# Restore BACKUP into a throwaway copy and check that the copy is usable
[group("db")]
db-verify-restore BACKUP:
    #!/usr/bin/env bash
    set -euo pipefail
    backup="{{ BACKUP }}"
    test -f "$backup" || { echo "no backup file at '$backup'" >&2; exit 1; }
    workspace="$(mktemp --directory)"
    trap 'rm --recursive --force "$workspace"' EXIT
    # A backup is a static file, so this copy cannot tear a page the way a copy
    # of the running database can.
    cp -- "$backup" "$workspace/restored.db"
    cargo run --quiet --no-default-features --features ssr -- verify-restore "$workspace/restored.db"
