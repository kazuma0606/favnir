FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

# fav バイナリ（scripts/build-and-push.sh が x86_64-unknown-linux-musl でビルドして配置）
COPY fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav

# migrate.fav パイプライン（v15.0.0 から変更なし）
COPY migrate.fav /app/migrate.fav

WORKDIR /app

# 接続情報は ECS タスク定義の secrets / environment から注入される
ENV DATABASE_URL=""
ENV AZURE_CONN_STR=""
ENV AZURE_STORAGE_ACCOUNT=""
ENV AZURE_STORAGE_KEY=""
ENV AZURE_CONTAINER="proof"

CMD ["fav", "run", "--legacy", "/app/migrate.fav"]
