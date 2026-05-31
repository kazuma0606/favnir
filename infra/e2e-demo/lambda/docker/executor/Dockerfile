FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fav/ .
ENV CXXFLAGS=""
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fav /usr/local/bin/fav
COPY infra/e2e-demo/lambda/docker/executor/handler.sh /handler.sh
COPY infra/e2e-demo/lambda/docker/executor/bootstrap /bootstrap
RUN chmod +x /handler.sh /bootstrap
# .fav ファイルはコピーしない（runtime イメージ）
ENTRYPOINT ["/bootstrap"]
