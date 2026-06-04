# Favnir v10.1.0 Tasks

Date: 2026-06-04
Theme: Snowflake インフラ構築（Terraform）— `infra/snowflake/` + AWS SSM 接続情報管理

---

## Phase A: Terraform ファイル作成

`infra/snowflake/` ディレクトリに Terraform 構成を追加する。
Rust/Favnir コードの変更なし。

- [x] A-1: `infra/snowflake/providers.tf` を作成
  - `terraform { required_version >= 1.5 }`
  - `aws ~> 5.0` + `Snowflake-Labs/snowflake ~> 0.87`（後に v0.100 へ修正）
  - S3 backend: bucket=`favnir-terraform-state`, key=`snowflake/terraform.tfstate`, region=`ap-northeast-1`
  - `provider "snowflake"` に account / user / role を設定
  - **追加（apply 時修正）**: version `~> 0.100`、`authenticator = "JWT"`、`private_key = file(...)` に変更
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
  - **追加（apply 時修正）**: `snowflake_role` → `snowflake_account_role`、`snowflake_grant_privileges_to_role` → `snowflake_grant_privileges_to_account_role`、`local.snowflake_account` を locals に追加
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

- [x] B-1: RSA 鍵ペアを生成する
  ```bash
  openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out snowflake_rsa_key.p8
  openssl rsa -in snowflake_rsa_key.p8 -pubout -out snowflake_rsa_key.pub
  ```
  - `infra/snowflake/snowflake_rsa_key.p8` / `.pub` 生成済み
  - 公開鍵フィンガープリント: `SHA256:esJnLXZIP/bOd4Bbbyqc8F274i5z25zpNfTCPI4yg+Y=`
- [x] B-2: Snowflake ユーザーに RSA 公開鍵を登録する
  ```sql
  ALTER USER YOSHIMURAHISANORI SET RSA_PUBLIC_KEY='MIIBIjAN...';
  ```
  - Snowflake Worksheets（Projects → SQL Worksheet）で実行済み
  - `DESC USER YOSHIMURAHISANORI` で `RSA_PUBLIC_KEY_FP` の一致を確認済み
  - `snow connection test -c favnir` → JWT 認証成功確認済み（`snowflake_rsa_key.p8` でログイン可）
- [ ] B-3: AWS SSM に秘密鍵・ユーザー名・ロールを格納する（手動、未実施）
  ```bash
  aws ssm put-parameter --name "/favnir/snowflake/private_key" --type "SecureString" --value "$(cat snowflake_rsa_key.p8)" --overwrite
  aws ssm put-parameter --name "/favnir/snowflake/user"        --type "SecureString" --value "YOSHIMURAHISANORI" --overwrite
  aws ssm put-parameter --name "/favnir/snowflake/role"        --type "String"       --value "FAVNIR_APP"        --overwrite
  ```
- [x] B-4: `infra/snowflake/terraform.tfvars` を作成（.gitignore 対象）
  ```hcl
  snowflake_organization    = "rtqjkbw"
  snowflake_account_name    = "ix11747"
  snowflake_user            = "YOSHIMURAHISANORI"
  snowflake_private_key_path = "./snowflake_rsa_key.p8"
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

- [x] D-1: `terraform init` が通ること
  - provider v0.100.0 で初期化済み（`.terraform.lock.hcl` 存在確認）
- [x] D-2: `terraform plan` / `terraform apply` が通ること
  - providers.tf・main.tf・ssm.tf・outputs.tf を provider v0.100 向けに修正後に通過
  - `terraform apply` → **11 resources created**
    - `FAVNIR_WH` / `FAVNIR` DB / `PUBLIC` schema / `FAVNIR_APP` ロール + grants 3 件 / SSM 4 件
- [ ] D-3: 既存 infra への影響がないこと（未確認）
  ```bash
  cd infra/registry && terraform plan  # No changes
  cd infra/site    && terraform plan  # No changes
  ```
- [ ] D-4: SSM パラメータが 7 件格納されていること（4/7 完了）
  ```bash
  aws ssm get-parameters-by-path --path /favnir/snowflake/ --with-decryption
  ```
  - [x] account / warehouse / database / schema（terraform apply で自動作成）
  - [ ] private_key / user / role（B-3 の手動格納が必要）
- [x] D-5: `cargo test` — 1261 件全通過（Rust コード変更なし）
- [x] D-6: 本ファイル完了チェック
- [x] D-7: `memory/MEMORY.md` に v10.1.0 完了を記録
- [x] D-8: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `infra/snowflake/` に 5 つの .tf ファイルが存在する | ✅ |
| `terraform init` が通る | ✅ |
| `terraform apply` で 11 リソース作成 | ✅ |
| SSM に `/favnir/snowflake/*` が 7 件格納されている | ⚠️ 4/7（B-3 残）|
| `infra/snowflake/README.md` にセットアップ手順がある | ✅ |
| 既存 infra（registry / site）に `terraform plan` で変更なし | ⬜ 未確認 |
| `cargo test` 1261 件全通過 | ✅ |
| snow CLI JWT 認証成功 | ✅ |

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

### provider v0.100 への対応（apply 時に発覚）

当初 `~> 0.87` を指定したが `.terraform.lock.hcl` に `0.100.0` が固定されており、
以下の互換性修正が必要だった：
- `snowflake_role` → `snowflake_account_role`
- `snowflake_grant_privileges_to_role` → `snowflake_grant_privileges_to_account_role`
- `private_key_path`（deprecated）→ `private_key = file(...)` + `authenticator = "JWT"`
- `var.snowflake_account`（未宣言）→ `local.snowflake_account`（`"${org}-${account_name}"` で導出）
- `outputs.tf` の role 参照を `snowflake_account_role` に修正

### snow CLI 接続設定

`~/.snowflake/config.toml` に `[connections.favnir]` を追加:
- `account = "rtqjkbw-ix11747"` / `authenticator = "SNOWFLAKE_JWT"`
- `private_key_file = "C:/Users/yoshi/favnir/infra/snowflake/snowflake_rsa_key.p8"`

### 次のバージョン（v10.2.0）への接続

v10.2.0 では `vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` を追加する。
これらは以下の環境変数を読み込む：
- `SNOWFLAKE_ACCOUNT`
- `SNOWFLAKE_PRIVATE_KEY`（PEM 全文、改行含む）
- `SNOWFLAKE_USER`
- `SNOWFLAKE_ROLE`

本バージョンで SSM に格納した値を Lambda 実行環境に注入する方式（v10.2.0 で設計）。
