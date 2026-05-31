# ---- Stage 1: Builder ----
# ECS 版 runtime/Dockerfile と同一ビルドステージ
FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fav/ .
# Linux (GCC) 環境では Windows MSVC 用 CXXFLAGS をクリア
ENV CXXFLAGS=""
RUN cargo build --release

# ---- Stage 2: Toolchain ----
# runtime との違い: /app/src/pipeline.fav を含む — これが toolchain イメージの証明
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fav /usr/local/bin/fav
COPY infra/e2e-demo/eks/src/pipeline.fav /app/src/pipeline.fav
ENTRYPOINT ["/usr/local/bin/fav"]
