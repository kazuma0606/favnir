# SAP OData v4 Mock Server — ローカル開発環境

SAP 公式 OSS モックサーバー（`@sap-ux/mockserver-main`）を Docker Compose で起動し、
本番 SAP ライセンスなしで Favnir SAP OData Rune の開発・テストを行うための環境です。

## 前提条件

- Docker Desktop（または Docker Engine + Compose Plugin）がインストール済みであること
- ポート `4004` が空いていること

## 起動方法

```bash
# リポジトリルートから実行
./scripts/start-sap-mock.sh

# または直接 Docker Compose で実行
cd infra/e2e-demo/sap-odata
docker compose up -d
```

## 停止方法

```bash
cd infra/e2e-demo/sap-odata
docker compose down
```

## エンドポイント

| エンドポイント | 説明 |
|---|---|
| `http://localhost:4004/BusinessPartnerCollection` | BusinessPartner 一覧（10 件） |
| `http://localhost:4004/SalesOrderCollection` | SalesOrder 一覧（10 件） |

## モックデータ

`mock/` ディレクトリに OData v4 形式の JSON ファイルが格納されています。

| ファイル | 件数 | 説明 |
|---|---|---|
| `BusinessPartnerCollection.json` | 10 件 | 取引先マスタサンプル |
| `SalesOrderCollection.json` | 10 件 | 受注サンプル |

## Favnir との接続

`fav.toml` に以下を設定してください（ローカル開発時）:

```toml
[sap]
base_url = "http://localhost:4004"
client   = "100"
username = "demo"
password = "demo"
auth     = "basic"
```

## 関連バージョン

- v85.1.0: `SapTomlConfig` + `inject_sap_config()` Rust 基盤
- v85.2.0: `SapConfig` Favnir 型 + `sap_config_from_env()`
- v85.3.0: 本 Docker Compose 環境
- v85.4.0: `runes/sap-odata/` 骨格 + `rune.toml`
- v85.5.0: OData v4 HTTP クライアント基盤
