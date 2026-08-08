# Toolchain: /app/src/pipeline.fav を含む — これが toolchain イメージの証明
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY fav/fav-al2023 /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav
COPY infra/e2e-demo/eks/src/pipeline.fav /app/src/pipeline.fav
ENTRYPOINT ["/usr/local/bin/fav"]
