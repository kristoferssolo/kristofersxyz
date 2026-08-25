# Compute recipe
FROM lukemathwalker/cargo-chef:0.1.78-rust-1.98.0-bookworm AS chef
WORKDIR /app
# droast ignore=DF007 reason="build context is constrained by .dockerignore"
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Install tools and build dependencies
FROM rustlang/rust:nightly-bookworm AS cacher
WORKDIR /app
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Install cargo-binstall
RUN curl --fail --location --silent --show-error\
    https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz\
    | tar -xz -C /usr/local/bin

RUN cargo binstall cargo-leptos cargo-chef -y

# Cook dependencies
COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Actual build
FROM rustlang/rust:nightly-bookworm AS builder
WORKDIR /app

# Copy the tools from the cacher stage
COPY --from=cacher /usr/local/rustup /usr/local/rustup
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY rust-toolchain.toml ./
RUN rustup target add wasm32-unknown-unknown

# Bring in the cooked dependencies
COPY --from=cacher /app/target target
# droast ignore=DF007 reason="build context is constrained by .dockerignore"
COPY . .

# Build the Leptos app
RUN cargo leptos build --release -vv

# Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update\
    && apt-get install -y --no-install-recommends \
        ca-certificates=20250419~deb12u1\
        curl=7.88.1-10+deb12u15\
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --no-create-home --home-dir /app \
        --shell /usr/sbin/nologin app \
    && mkdir /app/data \
    && chown 10001:10001 /app/data

# Copy binaries and assets
COPY --chown=10001:10001 --from=builder /app/target/release/kristofersxyz /app/
COPY --chown=10001:10001 --from=builder /app/target/site /app/site

ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000

EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD ["curl", "--fail", "--silent", "--show-error", "--output", "/dev/null", "http://127.0.0.1:3000/"]
USER 10001:10001
CMD ["/app/kristofersxyz"]
