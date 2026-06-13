FROM public.ecr.aws/lambda/provided:al2023

# jq をインストール（bootstrap の JSON パース用）
RUN dnf install -y jq && dnf clean all

# fav バイナリ（x86_64-unknown-linux-musl でビルド済み）
COPY fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav

# Favnir verifier ソース
COPY verifier.fav /var/task/verifier.fav

# カスタムランタイム bootstrap（Lambda Runtime API ループ）
COPY bootstrap /var/runtime/bootstrap
RUN chmod +x /var/runtime/bootstrap

CMD ["verifier.handler"]
