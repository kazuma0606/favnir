# Stage 1: Build the fav binary
FROM rust:1.88-slim AS builder
# Build dependencies for bundled duckdb, rusqlite, and TLS
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY fav/ ./fav/
WORKDIR /build/fav
# Clear Windows-specific compiler flags
ENV CXXFLAGS=""
RUN cargo build --release --bin fav

# Stage 2: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies: libssl, curl (runtime API), jq (event parsing)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Copy fav binary
COPY --from=builder /build/fav/target/release/fav /usr/local/bin/fav

# Copy Favnir source
WORKDIR /app
COPY rune-registry/src/ ./src/
COPY rune-registry/fav.toml ./

# Copy Lambda custom runtime bootstrap
COPY rune-registry/bootstrap /var/runtime/bootstrap
RUN chmod +x /var/runtime/bootstrap

ENTRYPOINT ["/var/runtime/bootstrap"]
