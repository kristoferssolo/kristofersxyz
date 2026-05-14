# Compute recipe
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Install tools and build dependencies
FROM rustlang/rust:nightly-bookworm AS cacher
WORKDIR /app

# Install cargo-binstall
RUN curl -L https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz | tar -xz -C /usr/local/bin

RUN cargo binstall cargo-leptos cargo-chef -y
RUN rustup target add wasm32-unknown-unknown

# Cook dependencies
COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Actual build
FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /app

# Copy the tools from the cacher stage
COPY --from=cacher /usr/local/rustup /usr/local/rustup
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Bring in the cooked dependencies
COPY --from=cacher /app/target target
COPY . .

# Build the Leptos app
RUN cargo leptos build --release -vv

# Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy binaries and assets
COPY --from=builder /app/target/release/server /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/config /app/config

ENV APP_ENVIRONMENT=production
ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000

EXPOSE 3000
CMD ["/app/server"]
