FROM rust:bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake libssl-dev pkg-config libclang-dev clang \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY . .
# Windows 専用 MSVC フラグ（/EHsc /utf-8）をクリア
ENV CXXFLAGS=""
RUN cargo build --release --bin fav
