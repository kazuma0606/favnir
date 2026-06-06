FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV HOME=/root

RUN apt-get update && apt-get install -y \
    curl ca-certificates \
    python3 python3-pip \
    libpq-dev gcc \
    && rm -rf /var/lib/apt/lists/*

# uv
RUN pip3 install --no-cache-dir uv

# psycopg2-binary（uv でも利用可能にしておく）
RUN pip3 install --no-cache-dir psycopg2-binary

# fav binary（ビルド済みをコピー）
COPY fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav

# app sources
WORKDIR /app
COPY src/ /app/

CMD ["fav", "run", "/app/pipeline.fav"]
