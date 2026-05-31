# ---- Stage 1: Builder ----
# rust:slim-bookworm は debian-based (glibc) のため
# duckdb (bundled) / wasmtime (cranelift) の C++ native 依存と互換性あり
# Alpine (musl) は glibc 依存ライブラリとの相性問題があるため使用しない
FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fav/ .
# .cargo/config.toml に Windows MSVC 用の CXXFLAGS=/EHsc /utf-8 が設定されているため
# Linux (GCC) 環境では明示的にクリアする（force=false なので ENV で上書き可能）
ENV CXXFLAGS=""
RUN cargo build --release

# ---- Stage 2: Runtime ----
# .fav ファイルは一切含まない — これが証明の核心
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fav /usr/local/bin/fav
ENTRYPOINT ["/usr/local/bin/fav"]
