# Favnir v10.1.0 実装計画 — Snowflake インフラ構築（Terraform）

作成日: 2026-06-04

---

## 全体構成

変更が必要なファイルと依存関係：

```
Phase A: Terraform ファイル作成
  infra/snowflake/providers.tf  ← 新規
  infra/snowflake/variables.tf  ← 新規
  infra/snowflake/main.tf       ← 新規（warehouse / database / schema / role）
  infra/snowflake/ssm.tf        ← 新規（SSM Parameter Store）
  infra/snowflake/outputs.tf    ← 新規

Phase B: 接続情報の手動格納
  AWS SSM Parameter Store  ← aws ssm put-parameter で手動実行
  Snowflake ユーザー設定   ← ALTER USER で RSA 公開鍵登録

Phase C: ドキュメント
  infra/snowflake/README.md  ← 新規

Phase D: 検証 + commit
  terraform init / plan 実行確認
  既存 infra への影響確認
```

Rust/Favnir コードの変更なし。`cargo test` テスト件数 1261 件維持。

---

## Phase A — Terraform ファイル作成

### A-1: providers.tf

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

### A-2: variables.tf

```hcl
variable "aws_region" {
  type    = string
  default = "ap-northeast-1"
}

variable "environment" {
  type    = string
  default = "prod"
}

variable "snowflake_account" {
  type        = string
  description = "Snowflake account identifier (e.g. xy12345.ap-northeast-1.aws)"
}

variable "snowflake_user" {
  type        = string
  description = "Snowflake user for Terraform operations"
}

variable "snowflake_admin_role" {
  type    = string
  default = "SYSADMIN"
}

variable "snowflake_warehouse_size" {
  type    = string
  default = "X-SMALL"
}

variable "snowflake_database" {
  type    = string
  default = "FAVNIR"
}

variable "snowflake_schema" {
  type    = string
  default = "PUBLIC"
}
```

### A-3: main.tf

spec.md の「4. main.tf — Snowflake リソース定義」の内容をそのまま実装する。

4 つのリソースブロック：
1. `snowflake_warehouse.favnir`（auto_suspend=60、X-SMALL、initially_suspended=true）
2. `snowflake_database.favnir`
3. `snowflake_schema.public`
4. `snowflake_role.favnir_app` + 権限付与 3 件（WAREHOUSE USAGE / DATABASE USAGE / SCHEMA CREATE）

### A-4: ssm.tf

spec.md の「5. ssm.tf — 接続情報の SSM 格納」の内容を実装する。

- `aws_ssm_parameter.snowflake_account`（SecureString、lifecycle.ignore_changes）
- `aws_ssm_parameter.snowflake_warehouse`（String、warehouse 名を自動設定）
- `aws_ssm_parameter.snowflake_database`（String）
- `aws_ssm_parameter.snowflake_schema`（String）
- `/favnir/snowflake/private_key` と `/favnir/snowflake/user` は手動格納のみ（Terraform 管理外）

### A-5: outputs.tf

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

## Phase B — 接続情報の手動格納

### B-1: RSA 鍵ペア生成

```bash
openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out snowflake_rsa_key.p8
openssl rsa -in snowflake_rsa_key.p8 -pubout -out snowflake_rsa_key.pub
```

### B-2: Snowflake ユーザーに公開鍵を登録

Snowflake の Web コンソールまたは CLI で実行：

```sql
ALTER USER FAVNIR_USER SET RSA_PUBLIC_KEY='<snowflake_rsa_key.pub の本文>';
```

### B-3: SSM に秘密鍵・ユーザー名を格納

```bash
# private_key
aws ssm put-parameter \
  --name "/favnir/snowflake/private_key" \
  --type "SecureString" \
  --value "$(cat snowflake_rsa_key.p8)" \
  --overwrite

# user
aws ssm put-parameter \
  --name "/favnir/snowflake/user" \
  --type "SecureString" \
  --value "FAVNIR_USER" \
  --overwrite

# role
aws ssm put-parameter \
  --name "/favnir/snowflake/role" \
  --type "String" \
  --value "FAVNIR_APP" \
  --overwrite
```

### B-4: terraform.tfvars の作成（gitignore 対象）

```hcl
snowflake_account = "xy12345.ap-northeast-1.aws"
snowflake_user    = "FAVNIR_TF_USER"
```

`infra/snowflake/terraform.tfvars` は `.gitignore` に追加する。

---

## Phase C — ドキュメント

### C-1: README.md

spec.md の「7. README.md 内容」に従い作成。

セクション：
1. 前提条件
2. 初回セットアップ（RSA 鍵 → Snowflake 登録 → SSM 格納 → terraform init → apply）
3. 環境変数一覧（v10.2.0 VM Primitive 用：`SNOWFLAKE_ACCOUNT` / `SNOWFLAKE_PRIVATE_KEY` / `SNOWFLAKE_USER` / `SNOWFLAKE_ROLE`）
4. LocalStack 開発（SSM のみ LocalStack 経由・Snowflake は実接続）
5. コスト管理（auto_suspend / auto_resume の説明）

---

## Phase D — 検証 + commit

### D-1: terraform init

```bash
cd infra/snowflake
terraform init
```

- Snowflake provider（`Snowflake-Labs/snowflake ~> 0.87`）が取得できること
- S3 backend に接続できること

### D-2: terraform plan

```bash
terraform plan -var-file=terraform.tfvars
```

- エラーなし（リソース作成計画が表示される）
- `Plan: N to add, 0 to change, 0 to destroy.` 形式の出力

### D-3: 既存 infra への影響確認

```bash
cd infra/registry && terraform plan
cd infra/site && terraform plan
```

- 両者とも `No changes` であること

### D-4: SSM パラメータ確認

```bash
aws ssm get-parameters-by-path --path /favnir/snowflake/ --with-decryption
```

- account / warehouse / database / schema / user / role / private_key の 7 件が存在すること

### D-5: .gitignore 更新

`infra/snowflake/.gitignore`（または最上位の `.gitignore`）に追加：

```
terraform.tfvars
*.p8
*.pub
.terraform/
.terraform.lock.hcl
```

### D-6: commit

---

## 実装順序と依存関係

```
A-1（providers.tf） ← 最初（他に依存なし）
A-2（variables.tf） ← A-1 と並行可
A-3（main.tf）      ← A-1 + A-2 完了後
A-4（ssm.tf）       ← A-3 完了後（リソース参照あり）
A-5（outputs.tf）   ← A-3 + A-4 完了後

B-1〜B-4           ← A-1〜A-5 完了後（手動作業）

C-1（README.md）    ← B 完了後（手順確定後に執筆）

D-1〜D-6           ← A〜C 完了後
```

---

## 注意点

### Snowflake プロバイダーのバージョン

`Snowflake-Labs/snowflake` はバージョンによって resource / argument 名が変わりやすい。
`~> 0.87` を固定し、アップグレード時はロックファイルを更新すること。

### auto_suspend の設定

`auto_suspend = 60`（60秒）はデモ用の最小設定。
本番では `auto_suspend = 600` 程度が現実的。spec では 60 秒を明示しているのでその通りに実装する。

### Snowflake backend の非対応

Snowflake は Terraform backend として使えない（S3 backend を継続使用する）。

### private_key の改行処理

SSM に格納された PEM 形式の秘密鍵は、v10.2.0 の `vm.rs` 実装時に改行を保持したまま読み込む必要がある。
`aws ssm get-parameter --with-decryption` で正しく復元できることを D-4 で確認する。

### LocalStack との整合

SSM パラメータは LocalStack（`AWS_ENDPOINT_URL=http://localhost:4566`）でテスト可能。
Snowflake リソース（warehouse / database 等）は実接続必須であり、LocalStack では代替できない。
