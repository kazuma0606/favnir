# Spec: v85.3.0 — Docker Compose — SAP OData モックサーバー構築

## Background

v85.2.0 で `SapConfig` Favnir 型と `sap_config_from_env()` を定義した。
本バージョンでは SAP 公式 OSS モックサーバーを Docker Compose で起動できる環境を整備する。
本番 SAP ライセンスなしでローカル開発・テストが可能になり、
後続バージョン（v85.5.0〜）の OData HTTP クライアント実装を手元で検証できるようになる。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.3.0 セクション）

## Goals

- `infra/e2e-demo/sap-odata/` に Docker Compose 構成を作成する
- `infra/e2e-demo/sap-odata/mock/` にモックデータ JSON を配置する
- `infra/e2e-demo/sap-odata/README.md` に起動手順を記述する
- `scripts/start-sap-mock.sh` 起動スクリプトを作成する
- Rust テスト 2 件を追加して **3,937 tests** を達成する

## Files to Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `infra/e2e-demo/sap-odata/docker-compose.yml` | 新規作成 | `sap-mock` + `favnir-runner` サービス定義 |
| `infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json` | 新規作成 | サンプル BusinessPartner 10 件 |
| `infra/e2e-demo/sap-odata/mock/SalesOrderCollection.json` | 新規作成 | サンプル SalesOrder 10 件 |
| `infra/e2e-demo/sap-odata/README.md` | 新規作成 | 起動手順・前提条件 |
| `scripts/start-sap-mock.sh` | 新規作成 | Docker Compose 起動スクリプト |
| `fav/src/driver.rs` | 追記 | `mod v85300_tests`（テスト 2 件） |

## docker-compose.yml 設計

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

## モックデータ形式

### BusinessPartnerCollection.json

OData v4 レスポンス形式（`value` 配列）。10 件のサンプルデータ。

```json
{
  "@odata.context": "$metadata#BusinessPartnerCollection",
  "value": [
    {
      "BusinessPartner": "1000001",
      "BusinessPartnerName": "SAP SE",
      "BusinessPartnerCategory": "2",
      "Country": "DE"
    }
  ]
}
```

### SalesOrderCollection.json

OData v4 レスポンス形式（`value` 配列）。10 件のサンプルデータ。

```json
{
  "@odata.context": "$metadata#SalesOrderCollection",
  "value": [
    {
      "SalesOrder": "0000000001",
      "SalesOrderType": "OR",
      "SoldToParty": "1000001",
      "TotalNetAmount": "10000.00",
      "TransactionCurrency": "EUR"
    }
  ]
}
```

## start-sap-mock.sh 設計

```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../infra/e2e-demo/sap-odata"
docker compose up -d
echo "SAP OData mock server started at http://localhost:4004"
```

## Success Criteria

- `cargo test` が **3,937 tests**, 0 failures
- `sap_mock_docker_compose_exists`:
  - `infra/e2e-demo/sap-odata/docker-compose.yml` が存在する（ファイル存在チェック）
- `sap_mock_data_business_partner_exists`:
  - `infra/e2e-demo/sap-odata/mock/BusinessPartnerCollection.json` が存在する（ファイル存在チェック）

## Error Codes

新規エラーコードなし。

## 注記

- `@sap-ux/mockserver-main` は SAP が提供する OSS の OData v4 モックサーバー（Apache 2.0）
- MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する
- テストはファイルの存在確認のみ（Docker 起動は必要としない）
- テストのファイルパス: `std::path::Path::new("../infra/e2e-demo/sap-odata/docker-compose.yml").exists()`（`cargo test` は `fav/` をカレントとして実行するため `../` 1 段で `favnir/` に到達し、さらに `infra/` へ）
