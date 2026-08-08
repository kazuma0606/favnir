# .fav ファイルは一切含まない — これが証明の核心
# pre-built binary (AL2023 / glibc 2.34) を使用（bookworm / glibc 2.36 互換）
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY fav/fav-al2023 /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav
