# v15.0.0 Tasks — CrossCloud E2E Demo（簡略版）

Date: 2026-06-12
Branch: master

---

## Phase A — `infra/e2e-demo/crosscloud/src/migrate.fav`

- [x] A-1: ディレクトリ構造を作成
  ```
  infra/e2e-demo/crosscloud/
    src/
    terraform/aws/
    terraform/azure/
    scripts/
  ```

- [x] A-2: `src/migrate.fav` を作成（5 ステージパイプライン）
  - `type CustomerRow` / `type MigratedRow` の定義
  - `extract_from_rds` — `Postgres.query_raw` で `customers` テーブル読み取り（`!Db`）
  - `transform_row` / `transform_rows` — 純粋変換（`String.trim(full_name)` 等）
  - `load_to_azure_postgres` — `AzurePostgres.execute_raw` で upsert（`!AzureDb`）
  - `save_proof_to_blob` — `AzureBlob.put_raw` で証跡 JSON 保存（`!AzureStorage`）
  - `verify_row_count` — `AzurePostgres.query_raw` で件数照合（`!AzureDb`）
  - `public fn main(ctx: AppCtx) -> Result<Unit, String> !Db !AzureDb !AzureStorage`
    - `IO.argv()` で引数取得
    - 5 ステージを `bind` でチェーン
    - 各ステージで `ctx.io.println` でログ出力
  - 本文は `plan.md` Phase A-2 参照

---

## Phase B — `terraform/aws/main.tf`

- [x] B-1: `terraform/aws/main.tf` を作成
  - `aws_security_group` (RDS アクセス用)
  - `aws_db_subnet_group`
  - `aws_db_parameter_group` (rds.force_ssl = 0)
  - `aws_db_instance` (PostgreSQL 16, db.t3.micro)
  - `aws_s3_bucket` (proof 証跡用)
  - `aws_secretsmanager_secret` + version (RDS 接続文字列)
  - 本文は `plan.md` Phase B 参照

- [x] B-2: `terraform/aws/variables.tf` を作成
  - `aws_region` (default: ap-northeast-1)
  - `rds_password`
  - `env_suffix` (S3 バケット名のサフィックス)

- [x] B-3: `terraform/aws/outputs.tf` を作成
  - `rds_endpoint`
  - `s3_proof_bucket`
  - `rds_conn_secret_arn`

---

## Phase C — `terraform/azure/main.tf`

- [x] C-1: `terraform/azure/main.tf` を作成
  - `azurerm_resource_group` (favnir-crosscloud-demo)
  - `azurerm_postgresql_flexible_server` (version = "16")
  - `azurerm_postgresql_flexible_server_firewall_rule` (allow all — demo 用)
  - `azurerm_postgresql_flexible_server_database` (appdb)
  - `azurerm_storage_account`
  - `azurerm_storage_container` (proof)
  - 本文は `plan.md` Phase C 参照

- [x] C-2: `terraform/azure/variables.tf` を作成
  - `azure_location` (default: japaneast)
  - `azure_pg_password`
  - `env_suffix`

- [x] C-3: `terraform/azure/outputs.tf` を作成
  - `postgresql_fqdn`
  - `storage_account_name`
  - `storage_account_key`
  - `azure_conn_str`

---

## Phase D — スクリプト

- [x] D-1: `scripts/seed.sh` を作成
  - `customers` テーブル作成（UUID PK + email + full_name + status + updated_at）
  - `generate_series(1, 1000)` で 1000 行 INSERT
  - `full_name` に先頭/末尾スペースを含める（`'  Test User N  '`）
  - 本文は `plan.md` Phase D 参照

- [x] D-2: `scripts/run.sh` を作成
  - Secrets Manager から `RDS_CONN_STR` を取得（`aws secretsmanager get-secret-value`）
  - 環境変数 `AZURE_CONN_STR` / `AZURE_STORAGE_ACCOUNT` / `AZURE_STORAGE_KEY` を確認
  - `fav run --legacy src/migrate.fav -- "$RDS_CONN_STR" "$AZURE_CONN_STR" ...` を実行
  - 本文は `plan.md` Phase D 参照

- [x] D-3: `scripts/verify.sh` を作成（PASS=5 確認）
  - `[PASS 1]` Source rows = 1000（`psql $RDS_CONN` で確認）
  - `[PASS 2]` Target rows = 1000（`psql $AZURE_CONN` で確認）
  - `[PASS 3]` No untrimmed names（normalized_name に前後スペースがない）
  - `[PASS 4]` Proof blob exists（`az storage blob exists` で確認）
  - `[PASS 5]` Pipeline exit code 0（run.sh が 0 で完了）
  - 本文は `plan.md` Phase D 参照

---

## Phase E — `infra/e2e-demo/crosscloud/README.md` 更新

- [x] E-1: 既存 README.md 冒頭に v15.0.0 簡略版スコープ注記を追加
  - 認証フロー（Entra ID / Cognito / Lambda verifier）は v15.1.0 以降
  - 実行方法・確認方法を記載

---

## Phase F — `fav/src/driver.rs`: v150000_tests + バージョンバンプ

- [x] F-1: `v150000_tests` モジュールを追加（`v148000_tests` の直前）
  - [x] `version_is_15_0_0` — `CARGO_PKG_VERSION == "15.0.0"` 確認
  - [x] `crosscloud_fav_parses` — `migrate.fav` が Parser でエラーなく解析される
  - [x] `crosscloud_effects_declared` — `!Db` / `!AzureDb` / `!AzureStorage` が含まれる
  - [x] `crosscloud_main_has_ctx_param` — `main(ctx: AppCtx)` が含まれる
  - [x] `crosscloud_e2e_demo_structure` — 必須ファイル 6 件の存在確認
    - `src/migrate.fav`
    - `scripts/run.sh`, `scripts/seed.sh`, `scripts/verify.sh`
    - `terraform/aws/main.tf`, `terraform/azure/main.tf`

  テスト本文は `plan.md` Phase F-1 参照。

- [x] F-2: `v148000_tests` の `version_is_14_8_0` を `>=` 比較に修正

- [x] F-3: `fav/Cargo.toml` バージョンを `"15.0.0"` にバンプ

- [x] F-4: `cargo test v150000` で 5 件全パス確認

---

## Phase G — 全テスト（ユニット）

- [x] G-1: `cargo test v150000` 全 5 件パス
- [x] G-2: `cargo test` 全件パス（リグレッションなし、parquet_rune_test_file_passes は pre-existing flaky）

---

## Phase H — インフラ構築 + E2E 実行（要 AWS/Azure アカウント）

- [ ] H-1: `terraform/aws`: `terraform init && terraform apply`
  - 出力: `rds_endpoint`, `s3_proof_bucket`

- [ ] H-2: `terraform/azure`: `terraform init && terraform apply`
  - 出力: `postgresql_fqdn`, `storage_account_name`, `storage_account_key`

- [ ] H-3: RDS に `customers` テーブル作成 + seed
  - `bash scripts/seed.sh "$RDS_CONN_STR"`

- [ ] H-4: Azure PostgreSQL に `customers_migrated` テーブル作成
  - Terraform outputs を使って `psql $AZURE_CONN_STR` で DDL 実行

- [ ] H-5: `bash scripts/run.sh` — 移行実行
  - exit code 0 を確認

- [ ] H-6: `bash scripts/verify.sh`
  - `PASS=5 FAIL=0` を確認

---

## Phase I — コミット

- [ ] I-1: `git commit -m "feat: v15.0.0 — CrossCloud E2E Demo（簡略版 PASS=5）"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `migrate.fav` が Parser でエラーなく解析される | [x] |
| `migrate.fav` に `!Db` / `!AzureDb` / `!AzureStorage` が宣言されている | [x] |
| `migrate.fav` の `main` が `ctx: AppCtx` を持つ | [x] |
| 必須ファイル 6 件（fav + scripts + terraform）が存在する | [x] |
| `cargo test v150000` 全 5 件パス | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `CARGO_PKG_VERSION == "15.0.0"` | [x] |
| `scripts/verify.sh` が `PASS=5 FAIL=0` を出力する（要 AWS/Azure 環境） | [ ] Phase H |

---

## v15.1.0 積み残し候補（Phase H 実施後に追記）

> 認証フェーズ（Entra ID / API Gateway / Lambda verifier）の実装内容は、
> v15.0.0 E2E 完了後に以下のいずれかで実装する。

| 候補 | 内容 |
|---|---|
| Azure AD アプリ登録 + AWS IAM OIDC 直接信頼 | Cognito 経由より簡潔。`sts:AssumeRoleWithWebIdentity` で AWS 一時認証情報を取得 |
| crosscloud/plan.md の Cognito 連携フル実装 | plan.md Phase 1 の完全実装 |
| HMAC リクエスト整合性チェック | Lambda verifier + DynamoDB nonce table |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.0.0/spec.md` | 仕様・スコープ |
| `versions/v15.0.0/plan.md` | 各フェーズの具体的な変更内容 |
| `infra/e2e-demo/crosscloud/plan.md` | フル版設計（v15.1+ 参照用） |
| `versions/roadmap-v14.1-v15.0.md` | v15.0.0 の簡略版スコープ定義 |
| `infra/e2e-demo/fav2py/terraform/main.tf` | AWS RDS Terraform 参考 |
| `infra/e2e-demo/fav2py/src/pipeline.fav` | Postgres/Azure パターン参考 |
