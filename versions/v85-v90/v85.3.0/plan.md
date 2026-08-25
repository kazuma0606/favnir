# Plan: v85.3.0 — Docker Compose — SAP OData モックサーバー構築

## Step 1: 前提確認

- `cargo test` を実行し、3,935 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85200_tests` が存在することを確認する（v85.2.0 完了済みの証拠）
- `infra/e2e-demo/` ディレクトリが存在することを確認する

## Step 2: `infra/e2e-demo/sap-odata/` ディレクトリ構成を作成

```bash
mkdir -p infra/e2e-demo/sap-odata/mock
```

## Step 3: `docker-compose.yml` を作成

`infra/e2e-demo/sap-odata/docker-compose.yml` に以下を作成する。

```yaml
version: "3.9"
services:
  sap-mock:
    image: node:20-alpine
    working_dir: /app
    command: >
      sh -c "npm install -g @sap-ux/mockserver-main &&
             mockserver --config /app/config.json"
    ports:
      - "4004:4004"
    volumes:
      - ./mock:/app/data
    environment:
      - PORT=4004

  favnir-runner:
    image: ghcr.io/favnir/fav:latest
    depends_on:
      - sap-mock
    environment:
      - SAP_BASE_URL=http://sap-mock:4004
      - SAP_CLIENT=100
      - SAP_USER=demo
      - SAP_PASS=demo
      - SAP_AUTH=basic
```

## Step 4: モックデータ JSON を作成

### `infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json`

OData v4 レスポンス形式（`value` 配列）、10 件のサンプルデータを作成。

### `infra/e2e-demo/sap-odata/mock/SalesOrderCollection.json`

OData v4 レスポンス形式（`value` 配列）、10 件のサンプルデータを作成。

## Step 5: `infra/e2e-demo/sap-odata/README.md` を作成

起動手順・前提条件（Docker が必要）を記述する。

## Step 6: `scripts/start-sap-mock.sh` を作成

```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../infra/e2e-demo/sap-odata"
docker compose up -d
echo "SAP OData mock server started at http://localhost:4004"
```

実行権限を付与: `chmod +x scripts/start-sap-mock.sh`

## Step 7: `fav/src/driver.rs` に `mod v85300_tests` を追加

`mod v85200_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85300_tests {
    #[test]
    fn sap_mock_docker_compose_exists() {
        assert!(
            std::path::Path::new("../infra/e2e-demo/sap-odata/docker-compose.yml").exists(),
            "infra/e2e-demo/sap-odata/docker-compose.yml should exist"
        );
    }

    #[test]
    fn sap_mock_data_business_partner_exists() {
        assert!(
            std::path::Path::new("../infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json").exists(),
            "infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json should exist"
        );
    }
}
```

## Step 8: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3937 tests, 0 failures
```

## Step 9: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.3.0 エントリを追加する。

## Step 10: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
