# Favnir v10.1.0 Tasks

Date: 2026-06-04
Theme: Snowflake インフラ構築（Terraform）— `infra/snowflake/` + AWS SSM 接続情報管理

---

## Phase A: Terraform ファイル作成

`infra/snowflake/` ディレクトリに Terraform 構成を追加する。
Rust/Favnir コードの変更なし。

- [x] A-1: `infra/snowflake/providers.tf` を作成
  - `terraform { required_version >= 1.5 }`
  - `aws ~> 5.0` + `Snowflake-Labs/snowflake ~> 0.87`
  - S3 backend: bucket=`favnir-terraform-state`, key=`snowflake/terraform.tfstate`, region=`ap-northeast-1`
  - `provider "snowflake"` に account / user / role を設定
- [x] A-2: `infra/snowflake/variables.tf` を作成
  - `aws_region`（default: `ap-northeast-1`）
  - `environment`（default: `prod`）
  - `snowflake_account` / `snowflake_user` / `snowflake_admin_role`（default: `SYSADMIN`）
  - `snowflake_warehouse_size`（default: `X-SMALL`）
  - `snowflake_database`（default: `FAVNIR`）/ `snowflake_schema`（default: `PUBLIC`）
- [x] A-3: `infra/snowflake/main.tf` を作成
  - `snowflake_warehouse.favnir`: `FAVNIR_WH`、auto_suspend=60、auto_resume=true、initially_suspended=true
  - `snowflake_database.favnir`: `FAVNIR`
  - `snowflake_schema.public`: `PUBLIC`
  - `snowflake_role.favnir_app`: `FAVNIR_APP`
  - 権限付与 3 件: WAREHOUSE USAGE / DATABASE USAGE / SCHEMA USAGE+CREATE TABLE+CREATE VIEW
  - タグ: `Project=favnir / Environment=var.environment / ManagedBy=terraform`
- [x] A-4: `infra/snowflake/ssm.tf` を作成
  - `aws_ssm_parameter.snowflake_account`（SecureString、`lifecycle { ignore_changes = [value] }`）
  - `aws_ssm_parameter.snowflake_warehouse`（String、warehouse 名を自動設定）
  - `aws_ssm_parameter.snowflake_database`（String）
  - `aws_ssm_parameter.snowflake_schema`（String）
  - `/favnir/snowflake/private_key` / `/favnir/snowflake/user` / `/favnir/snowflake/role` は手動格納（Terraform 管理外）
- [x] A-5: `infra/snowflake/outputs.tf` を作成
  - `snowflake_warehouse_name` / `snowflake_database_name` / `snowflake_app_role` / `ssm_prefix`

---

## Phase B: 接続情報の手動格納

Terraform apply 後に手動で実施する作業。

- [ ] B-1: RSA 鍵ペアを生成する
  ```bash
  openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out snowflake_rsa_key.p8
  openssl rsa -in snowflake_rsa_key.p8 -pubout -out snowflake_rsa_key.pub
  ```
- [ ] B-2: Snowflake ユーザーに RSA 公開鍵を登録する
  ```sql
  ALTER USER FAVNIR_USER SET RSA_PUBLIC_KEY='<snowflake_rsa_key.pub の本文>';
  ```
- [ ] B-3: AWS SSM に秘密鍵・ユーザー名・ロールを格納する
  ```bash
  aws ssm put-parameter --name "/favnir/snowflake/private_key" --type "SecureString" --value "$(cat snowflake_rsa_key.p8)" --overwrite
  aws ssm put-parameter --name "/favnir/snowflake/user"        --type "SecureString" --value "FAVNIR_USER"  --overwrite
  aws ssm put-parameter --name "/favnir/snowflake/role"        --type "String"       --value "FAVNIR_APP"   --overwrite
  ```
- [ ] B-4: `infra/snowflake/terraform.tfvars` を作成（.gitignore 対象）
  ```hcl
  snowflake_account = "xy12345.ap-northeast-1.aws"
  snowflake_user    = "FAVNIR_TF_USER"
  ```
- [x] B-5: `.gitignore` に `terraform.tfvars` / `*.p8` / `*.pub` / `.terraform/` / `.terraform.lock.hcl` を追加

---

## Phase C: ドキュメント

- [x] C-1: `infra/snowflake/README.md` を作成
  - **前提条件**: Terraform >= 1.5、AWS CLI 認証済み、Snowflake アカウント保有
  - **初回セットアップ**: RSA 鍵生成 → Snowflake ユーザー登録 → SSM 格納 → `terraform init` → `terraform apply`
  - **環境変数一覧**（v10.2.0 VM Primitive 用）:
    - `SNOWFLAKE_ACCOUNT` — アカウント ID
    - `SNOWFLAKE_PRIVATE_KEY` — RSA 秘密鍵（PEM 全文）
    - `SNOWFLAKE_USER` — ユーザー名
    - `SNOWFLAKE_ROLE` — ロール名（`FAVNIR_APP`）
  - **LocalStack 開発**: `AWS_ENDPOINT_URL=http://localhost:4566` で SSM のみローカル確認可。Snowflake は実接続必須
  - **コスト管理**: auto_suspend=60 / auto_resume=true の説明。本番では 600 秒推奨

---

## Phase D: 検証 + commit

- [ ] D-1: `terraform init` が通ること
  ```bash
  cd infra/snowflake && terraform init
  ```
  - Snowflake-Labs/snowflake プロバイダーが取得できること
  - S3 backend に接続できること
- [ ] D-2: `terraform plan` が通ること（エラーなし）
  ```bash
  terraform plan -var-file=terraform.tfvars
  ```
  - `Plan: N to add, 0 to change, 0 to destroy.` が出力される
- [ ] D-3: 既存 infra への影響がないこと
  ```bash
  cd infra/registry && terraform plan  # No changes
  cd infra/site    && terraform plan  # No changes
  ```
- [ ] D-4: SSM パラメータが 7 件格納されていること
  ```bash
  aws ssm get-parameters-by-path --path /favnir/snowflake/ --with-decryption
  ```
  - account / warehouse / database / schema / user / role / private_key
- [x] D-5: `cargo test` — 1261 件全通過（Rust コード変更なし）
- [x] D-6: 本ファイル完了チェック
- [x] D-7: `memory/MEMORY.md` に v10.1.0 完了を記録
- [x] D-8: commit

---

## 完了条件

| 条件 | 確認方法 |
|---|---|
| `infra/snowflake/` に 5 つの .tf ファイルが存在する | `ls infra/snowflake/*.tf` |
| `terraform init` が通る | 実行確認 |
| `terraform plan` がエラーなし | `Plan: N to add` が出力される |
| SSM に `/favnir/snowflake/*` が 7 件格納されている | `aws ssm get-parameters-by-path` |
| `infra/snowflake/README.md` にセットアップ手順がある | 目視確認 |
| 既存 infra（registry / site）に `terraform plan` で変更なし | `No changes` |
| `cargo test` 1261 件全通過 | `test result: ok. 1261 passed` |

---

## 実装メモ

### Snowflake プロバイダー認証方式

`Snowflake-Labs/snowflake ~> 0.87` は JWT 認証（RSA キーペア）を推奨。
`provider "snowflake"` ブロックに `private_key_path` または `private_key` を追加することも可能だが、
Terraform state にキーが入らないよう SSM 経由での実行時注入を標準とする。

### `initially_suspended = true` の意味

Terraform apply 直後にウェアハウスがアクティブにならない設定。
課金を最小化するため、Favnir の Rune が呼ばれたときのみ auto_resume で起動させる。

### terraform.tfvars の取り扱い

接続情報（account / user）は `.gitignore` 対象の `terraform.tfvars` に格納。
CI/CD から `terraform apply` する場合は環境変数 `TF_VAR_snowflake_account` 等を使用する。

### 次のバージョン（v10.2.0）への接続

v10.2.0 では `vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` を追加する。
これらは以下の環境変数を読み込む：
- `SNOWFLAKE_ACCOUNT`
- `SNOWFLAKE_PRIVATE_KEY`（PEM 全文、改行含む）
- `SNOWFLAKE_USER`
- `SNOWFLAKE_ROLE`

本バージョンで SSM に格納した値を Lambda 実行環境に注入する方式（v10.2.0 で設計）。
