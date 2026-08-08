FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli curl \
    && rm -rf /var/lib/apt/lists/*
COPY fav/fav-al2023 /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav
COPY infra/e2e-demo/lambda/src/pipeline.fav /app/src/pipeline.fav
COPY infra/e2e-demo/lambda/docker/compiler/handler.sh /handler.sh
COPY infra/e2e-demo/lambda/docker/compiler/bootstrap /bootstrap
RUN chmod +x /handler.sh /bootstrap
ENTRYPOINT ["/bootstrap"]
