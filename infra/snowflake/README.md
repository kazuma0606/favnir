# Favnir — Snowflake インフラ（Terraform）

Snowflake on AWS の Terraform 構成。
`infra/registry` / `infra/site` と同じ S3 backend / タグ規則を使用する。

---

## 前提条件

- Terraform >= 1.5
- AWS CLI 認証済み（`aws sts get-caller-identity` が通ること）
- Snowflake アカウント保有（trial / enterprise 問わず）
- Snowflake の SYSADMIN 相当のユーザーが Terraform 操作に使えること

---

## 初回セットアップ

### 1. RSA 鍵ペアを生成する

```bash
openssl genrsa 2048 | openssl pkcs8 -topk8 -nocrypt -out snowflake_rsa_key.p8
openssl rsa -in snowflake_rsa_key.p8 -pubout -out snowflake_rsa_key.pub
```

> `snowflake_rsa_key.p8` と `snowflake_rsa_key.pub` は `.gitignore` 対象。リポジトリに含めないこと。

### 2. Snowflake ユーザーに公開鍵を登録する

Snowflake の Web コンソール（Worksheets）または SnowSQL で実行：

```sql
-- Terraform 操作用ユーザーに公開鍵を登録
ALTER USER FAVNIR_TF_USER SET RSA_PUBLIC_KEY='<snowflake_rsa_key.pub の MIIBIjAN... 部分>';

-- アプリ用ユーザー（FAVNIR_USER）も同様に登録
ALTER USER FAVNIR_USER SET RSA_PUBLIC_KEY='<公開鍵>';
```

### 3. terraform.tfvars を作成する

```bash
cat > infra/snowflake/terraform.tfvars <<EOF
snowflake_account = "xy12345.ap-northeast-1.aws"
snowflake_user    = "FAVNIR_TF_USER"
EOF
```

> `terraform.tfvars` は `.gitignore` 対象。

### 4. terraform init / apply を実行する

```bash
cd infra/snowflake
terraform init
terraform plan -var-file=terraform.tfvars
terraform apply -var-file=terraform.tfvars
```

### 5. SSM に秘密鍵・アプリユーザー情報を格納する

Terraform が管理しない秘密情報を手動で SSM に格納する：

```bash
# RSA 秘密鍵（PEM 全文）
aws ssm put-parameter \
  --name "/favnir/snowflake/private_key" \
  --type "SecureString" \
  --value "$(cat snowflake_rsa_key.p8)" \
  --overwrite

# アプリ用ユーザー名
aws ssm put-parameter \
  --name "/favnir/snowflake/user" \
  --type "SecureString" \
  --value "FAVNIR_USER" \
  --overwrite

# アプリ用ロール
aws ssm put-parameter \
  --name "/favnir/snowflake/role" \
  --type "String" \
  --value "FAVNIR_APP" \
  --overwrite
```

### 6. 格納内容を確認する

```bash
aws ssm get-parameters-by-path \
  --path /favnir/snowflake/ \
  --with-decryption \
  --query "Parameters[*].{Name:Name,Value:Value}" \
  --output table
```

7 件（account / warehouse / database / schema / private_key / user / role）が表示されれば完了。

---

## 環境変数一覧

v10.2.0 以降の VM Primitive（`Snowflake.execute_raw` / `Snowflake.query_raw`）が読み込む環境変数：

| 環境変数 | 内容 | SSM パス |
|---|---|---|
| `SNOWFLAKE_ACCOUNT` | アカウント ID（例: `xy12345.ap-northeast-1.aws`）| `/favnir/snowflake/account` |
| `SNOWFLAKE_PRIVATE_KEY` | RSA 秘密鍵（PEM 全文、改行含む）| `/favnir/snowflake/private_key` |
| `SNOWFLAKE_USER` | Snowflake ユーザー名 | `/favnir/snowflake/user` |
| `SNOWFLAKE_ROLE` | Snowflake ロール名（`FAVNIR_APP`）| `/favnir/snowflake/role` |

Lambda / ECS での実行時は SSM から取得して環境変数に注入する。

ローカル実行時：

```bash
export SNOWFLAKE_ACCOUNT="xy12345.ap-northeast-1.aws"
export SNOWFLAKE_USER="FAVNIR_USER"
export SNOWFLAKE_ROLE="FAVNIR_APP"
export SNOWFLAKE_PRIVATE_KEY="$(aws ssm get-parameter \
  --name /favnir/snowflake/private_key --with-decryption \
  --query Parameter.Value --output text)"
```

---

## LocalStack での開発

SSM パラメータのみ LocalStack でシミュレーション可能。
Snowflake リソース（warehouse / database 等）は実接続が必須。

```bash
# LocalStack 起動
docker run -d -p 4566:4566 localstack/localstack

# SSM をローカルで操作
AWS_ENDPOINT_URL=http://localhost:4566 aws ssm put-parameter \
  --name "/favnir/snowflake/account" \
  --type "SecureString" \
  --value "test-account" \
  --overwrite
```

---

## コスト管理

| 設定 | 値 | 説明 |
|---|---|---|
| `warehouse_size` | `X-SMALL` | 最小サイズ（デモ用）|
| `auto_suspend` | 60 秒 | 60 秒間クエリがなければ停止 |
| `auto_resume` | true | Favnir Rune からのクエリで自動再開 |
| `initially_suspended` | true | `terraform apply` 直後はサスペンド状態 |

本番環境では `auto_suspend = 600`（10 分）程度に設定することを推奨。

---

## Terraform 操作

```bash
cd infra/snowflake

# 初期化
terraform init

# 差分確認
terraform plan -var-file=terraform.tfvars

# 適用
terraform apply -var-file=terraform.tfvars

# 出力確認
terraform output
```

---

## 関連バージョン

| バージョン | 内容 |
|---|---|
| v10.1.0 | このディレクトリ（Terraform 基盤）|
| v10.2.0 | `vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` 追加 |
| v10.3.0 | `!Snowflake` エフェクト型を言語に追加 |
| v10.6.0 | `runes/snowflake/` — Snowflake Rune 実装 |
| v10.9.0 | E2E テスト（実 Snowflake 接続）|
