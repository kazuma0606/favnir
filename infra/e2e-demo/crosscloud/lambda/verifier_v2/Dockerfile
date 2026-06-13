FROM public.ecr.aws/lambda/provided:al2023

# jq をインストール（bootstrap の JSON パース用）
RUN dnf install -y jq && dnf clean all

# fav バイナリ（x86_64-glibc でビルド済み: Dockerfile.builder 使用）
COPY fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav

# Favnir verifier_v2 ソース（KMS ECDSA P-256 版）
COPY verifier_v2.fav /var/task/verifier_v2.fav

# カスタムランタイム bootstrap（Lambda Runtime API ループ）
COPY bootstrap /var/runtime/bootstrap
RUN chmod +x /var/runtime/bootstrap

CMD ["verifier_v2.handler"]
