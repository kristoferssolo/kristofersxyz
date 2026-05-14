export RUSTC_WRAPPER :=  env("RUSTC_WRAPPER", "sccache")
export RUST_LOG := env("RUST_LOG", "warn")

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

# Run all checks (fmt, clippy, docs, test)
[group("dev")]
check: fmt clippy docs test

# Run the development server
[group("run")]
run:
    cargo leptos watch | bunyan

# Format code
[group("dev")]
fmt:
    cargo fmt --all

# Run clippy
[group("dev")]
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
[group("dev")]
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Run tests with nextest
[group("dev")]
test:
    cargo nextest run --all-features
    cargo leptos test

# Clean build artifacts
[group("dev")]
clean:
    cargo clean

[group("dev")]
setup:
    cargo install cargo-nextest sccache

# Add a new migration
[group("db")]
migrate-add NAME:
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

# Serve the release build
serve: build
    ./target/server/release/server

# Run end-to-end tests
end2end:
    cd end2end && npx playwright test

# CI pipeline
ci:
    just check
    just end2end
