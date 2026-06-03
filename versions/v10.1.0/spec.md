# Favnir v10.1.0 仕様書 — Snowflake インフラ構築（Terraform）

作成日: 2026-06-04

---

## 概要

Snowflake on AWS の Terraform 基盤を整備する。
v10.2.0 以降の VM Primitive・エフェクト型・Rune 実装の前提となる接続情報管理を確立する。

Rust/Favnir コードの変更はなし。テスト件数は 1261 件のまま維持。

---

## 1. ディレクトリ構成

```
infra/snowflake/
  providers.tf      # Terraform / AWS / Snowflake プロバイダー設定
  variables.tf      # 入力変数定義
  main.tf           # Snowflake リソース（warehouse / database / schema / role）
  ssm.tf            # AWS SSM Parameter Store への接続情報格納
  outputs.tf        # 出力値（account ID / warehouse 名等）
  README.md         # セットアップ手順
```

---

## 2. providers.tf

```hcl
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    snowflake = {
      source  = "Snowflake-Labs/snowflake"
      version = "~> 0.87"
    }
  }
  backend "s3" {
    bucket = "favnir-terraform-state"
    key    = "snowflake/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

provider "aws" {
  region = var.aws_region
}

provider "snowflake" {
  account = var.snowflake_account
  user    = var.snowflake_user
  role    = var.snowflake_admin_role
}
```

---

## 3. variables.tf

| 変数名 | 型 | 説明 |
|---|---|---|
| `aws_region` | string | AWS リージョン（default: `ap-northeast-1`）|
| `environment` | string | 環境名（default: `prod`）|
| `snowflake_account` | string | Snowflake アカウント ID（例: `xy12345.ap-northeast-1.aws`）|
| `snowflake_user` | string | Terraform 用 Snowflake ユーザー名 |
| `snowflake_admin_role` | string | Terraform 用ロール（default: `SYSADMIN`）|
| `snowflake_warehouse_size` | string | ウェアハウスサイズ（default: `X-SMALL`）|
| `snowflake_database` | string | データベース名（default: `FAVNIR`）|
| `snowflake_schema` | string | スキーマ名（default: `PUBLIC`）|

---

## 4. main.tf — Snowflake リソース定義

### ウェアハウス

```hcl
resource "snowflake_warehouse" "favnir" {
  name           = "FAVNIR_WH"
  warehouse_size = var.snowflake_warehouse_size
  auto_suspend   = 60   # 60秒でサスペンド
  auto_resume    = true
  initially_suspended = true
}
```

### データベース

```hcl
resource "snowflake_database" "favnir" {
  name = var.snowflake_database
}
```

### スキーマ

```hcl
resource "snowflake_schema" "public" {
  database = snowflake_database.favnir.name
  name     = var.snowflake_schema
}
```

### アプリケーションロール

```hcl
resource "snowflake_role" "favnir_app" {
  name = "FAVNIR_APP"
}

resource "snowflake_grant_privileges_to_role" "warehouse_usage" {
  role_name  = snowflake_role.favnir_app.name
  privileges = ["USAGE"]
  on_account_object {
    object_type = "WAREHOUSE"
    object_name = snowflake_warehouse.favnir.name
  }
}

resource "snowflake_grant_privileges_to_role" "database_usage" {
  role_name  = snowflake_role.favnir_app.name
  privileges = ["USAGE"]
  on_account_object {
    object_type = "DATABASE"
    object_name = snowflake_database.favnir.name
  }
}

resource "snowflake_grant_privileges_to_role" "schema_privileges" {
  role_name  = snowflake_role.favnir_app.name
  privileges = ["USAGE", "CREATE TABLE", "CREATE VIEW"]
  on_schema {
    schema_name = "\"${snowflake_database.favnir.name}\".\"${snowflake_schema.public.name}\""
  }
}
```

---

## 5. ssm.tf — 接続情報の SSM 格納

接続情報はすべて AWS SSM Parameter Store の SecureString として格納する。
Terraform は SSM パラメータの参照先（パス）を管理し、値は手動で `aws ssm put-parameter` で格納する（秘密情報を Terraform state に含めない）。

### 格納するパラメータ

| SSM パス | 内容 |
|---|---|
| `/favnir/snowflake/account` | Snowflake アカウント ID |
| `/favnir/snowflake/user` | アプリ用ユーザー名 |
| `/favnir/snowflake/role` | アプリ用ロール（`FAVNIR_APP`）|
| `/favnir/snowflake/warehouse` | ウェアハウス名（`FAVNIR_WH`）|
| `/favnir/snowflake/database` | データベース名（`FAVNIR`）|
| `/favnir/snowflake/schema` | スキーマ名（`PUBLIC`）|
| `/favnir/snowflake/private_key` | JWT 認証用 RSA 秘密鍵（PEM）|

```hcl
# パラメータの参照先定義（値は手動で格納）
resource "aws_ssm_parameter" "snowflake_account" {
  name  = "/favnir/snowflake/account"
  type  = "SecureString"
  value = var.snowflake_account
  lifecycle { ignore_changes = [value] }
}

resource "aws_ssm_parameter" "snowflake_warehouse" {
  name  = "/favnir/snowflake/warehouse"
  type  = "String"
  value = snowflake_warehouse.favnir.name
}

resource "aws_ssm_parameter" "snowflake_database" {
  name  = "/favnir/snowflake/database"
  type  = "String"
  value = snowflake_database.favnir.name
}

resource "aws_ssm_parameter" "snowflake_schema" {
  name  = "/favnir/snowflake/schema"
  type  = "String"
  value = snowflake_schema.public.name
}

# private_key と user は手動格納のみ（Terraform で値を管理しない）
```

### RSA 鍵ペア生成（手動手順）

```bash
# 秘密鍵生成
openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out snowflake_rsa_key.p8

# 公開鍵生成
openssl rsa -in snowflake_rsa_key.p8 -pubout -out snowflake_rsa_key.pub

# Snowflake ユーザーに公開鍵を登録
# ALTER USER FAVNIR_USER SET RSA_PUBLIC_KEY='<公開鍵本文>';

# 秘密鍵を SSM に格納
aws ssm put-parameter \
  --name "/favnir/snowflake/private_key" \
  --type "SecureString" \
  --value "$(cat snowflake_rsa_key.p8)" \
  --overwrite
```

---

## 6. outputs.tf

```hcl
output "snowflake_warehouse_name" {
  value = snowflake_warehouse.favnir.name
}

output "snowflake_database_name" {
  value = snowflake_database.favnir.name
}

output "snowflake_app_role" {
  value = snowflake_role.favnir_app.name
}

output "ssm_prefix" {
  value = "/favnir/snowflake/"
}
```

---

## 7. README.md 内容

以下のセクションを含む：

1. **前提条件** — Terraform >= 1.5、AWS CLI 認証、Snowflake アカウント
2. **初回セットアップ** — RSA 鍵ペア生成 → Snowflake ユーザー登録 → SSM 格納 → `terraform init` → `terraform apply`
3. **環境変数** — `SNOWFLAKE_ACCOUNT` / `SNOWFLAKE_PRIVATE_KEY` / `SNOWFLAKE_USER` / `SNOWFLAKE_ROLE`（v10.2.0 VM Primitive 用）
4. **LocalStack 開発** — `AWS_ENDPOINT_URL=http://localhost:4566` でローカル動作確認（Snowflake 接続は実接続必須）
5. **運用** — Snowflake ウェアハウスのコスト管理（auto_suspend / auto_resume 設定）

---

## 8. 既存インフラとの整合

| 項目 | 既存 | v10.1.0 追加 |
|---|---|---|
| Terraform state bucket | `favnir-terraform-state` | 同じバケット / key=`snowflake/terraform.tfstate` |
| AWS リージョン | `ap-northeast-1` | 同じ |
| タグ規則 | `Project=favnir / ManagedBy=terraform` | 同じ |
| SSM パス規則 | `/favnir/<service>/...` | `/favnir/snowflake/...` |

---

## 完了条件

| 条件 | 確認方法 |
|---|---|
| `terraform init` が通る | `cd infra/snowflake && terraform init` |
| `terraform plan` が通る（エラーなし） | `terraform plan` |
| SSM パラメータ 4 件が格納されている | `aws ssm get-parameters-by-path --path /favnir/snowflake/` |
| `infra/snowflake/README.md` にセットアップ手順がある | 目視確認 |
| 既存 infra（registry / site）に影響なし | `cd infra/registry && terraform plan` |

---

## スコープ外（v10.2.0 以降）

- VM Primitive（`Snowflake.execute_raw` / `Snowflake.query_raw`）
- `!Snowflake` エフェクト型の Rust 実装
- Snowflake Rune（`runes/snowflake/`）
- E2E テスト（実 Snowflake 接続）
