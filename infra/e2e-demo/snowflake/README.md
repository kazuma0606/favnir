# Favnir v10.9.0 — Snowflake E2E Demo

## 概要

Favnir の Snowflake 統合（v10.1.0〜v10.8.0）を実際の Snowflake インスタンスに対して証明する E2E デモ。

**パイプライン**: `LoadCsv |> TransformRows |> SnowflakeInsert |> QuerySummary`

```
CSV ファイル (sample.csv)
  └─ LoadCsv        → List<OrderRow>      (!IO)
  └─ TransformRows  → List<OrderRow>      (pure)
  └─ SnowflakeInsert → Int               (!Snowflake)
  └─ QuerySummary   → Unit               (!Snowflake !AWS)
       └─ s3://favnir-e2e-demo/proof/snowflake/summary-<TIMESTAMP>.json
```

---

## 前提条件

- `fav` バイナリ（v10.9.0 以降）が `PATH` に存在すること
- AWS CLI が設定済みで `favnir-e2e-demo` バケットへの PutObject 権限があること
- Snowflake アカウントと RSA キーペアが用意されていること
- Terraform >= 1.5 と `snowflake-labs/snowflake` プロバイダーが使えること

---

## セットアップ

### 1. RSA キーペア生成（未作成の場合）

```bash
openssl genrsa -out snowflake_private_key.pem 2048
openssl rsa -in snowflake_private_key.pem -pubout -out snowflake_public_key.pem
```

Snowflake ユーザーに公開鍵を登録:

```sql
ALTER USER <YOUR_USER> SET RSA_PUBLIC_KEY='<公開鍵の内容（ヘッダー/フッター除く）>';
```

### 2. Terraform で Snowflake リソースを作成

```bash
cd infra/e2e-demo/snowflake/terraform

terraform init
terraform plan \
  -var="snowflake_account=<ACCOUNT_ID>" \
  -var="snowflake_user=<USER>"
terraform apply \
  -var="snowflake_account=<ACCOUNT_ID>" \
  -var="snowflake_user=<USER>"
```

作成されるリソース:
- Snowflake warehouse: `DEMO_WH`（X-Small、auto-suspend 60s）
- Snowflake database: `DEMO_DB`
- Snowflake table: `DEMO_DB.PUBLIC.ORDERS`
- AWS IAM role: `favnir-snowflake-e2e`（S3 proof 書き込み）

### 3. 環境変数の設定

```bash
export SNOWFLAKE_ACCOUNT="<ACCOUNT_ID>"          # 例: xy12345.ap-northeast-1.aws
export SNOWFLAKE_USER="<USER>"
export SNOWFLAKE_ROLE="SYSADMIN"
export SNOWFLAKE_WAREHOUSE="DEMO_WH"
export SNOWFLAKE_DATABASE="DEMO_DB"
export SNOWFLAKE_SCHEMA="PUBLIC"
export SNOWFLAKE_PRIVATE_KEY="$(cat snowflake_private_key.pem)"
```

または `fav.toml` に記載:

```toml
[snowflake]
account   = "${SNOWFLAKE_ACCOUNT}"
user      = "${SNOWFLAKE_USER}"
warehouse = "DEMO_WH"
role      = "SYSADMIN"
database  = "DEMO_DB"
schema    = "PUBLIC"
```

---

## 実行

```bash
cd infra/e2e-demo/snowflake
./scripts/run.sh
```

期待する出力:

```
=== Favnir v10.9.0 Snowflake E2E Demo ===
Timestamp: 20260604-120000

[1/4] fav check src/demo.fav ...
PASS: type check
[2/4] fav run src/demo.fav src/sample.csv ...
PASS: pipeline run
[3/4] verifying S3 proof ...
PASS: S3 summary proof exists
[4/4] uploading run log ...
PASS: run log uploaded

=== RESULT: PASS=4 FAIL=0 ===
```

---

## 証跡確認

```bash
# 集計結果
aws s3 ls s3://favnir-e2e-demo/proof/snowflake/
aws s3 cp s3://favnir-e2e-demo/proof/snowflake/summary-<TIMESTAMP>.json .
cat summary-<TIMESTAMP>.json
```

期待する集計結果（sample.csv 6 行分）:

```json
[
  { "region": "EU-WEST",  "total": 2700.0, "count": 2 },
  { "region": "US-EAST",  "total": 1550.0, "count": 2 },
  { "region": "US-WEST",  "total": 1251.25, "count": 2 }
]
```

---

## クリーンアップ

```bash
cd infra/e2e-demo/snowflake/terraform
terraform destroy \
  -var="snowflake_account=<ACCOUNT_ID>" \
  -var="snowflake_user=<USER>"
```

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `fav check src/demo.fav` 通過 | PASS |
| `fav run` が ORDERS テーブルに 6 行 INSERT | PASS |
| `s3://favnir-e2e-demo/proof/snowflake/summary-*.json` 存在 | PASS |
| 実行ログが S3 に保存済み | PASS |
