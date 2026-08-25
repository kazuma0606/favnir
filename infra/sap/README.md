# infra/sap — SAP S/4HANA SSM Parameter Store

SAP S/4HANA OData v4 接続情報を AWS SSM Parameter Store で管理する Terraform モジュール。

## 前提条件

- Terraform >= 1.5
- AWS CLI 設定済み（`aws configure`）
- S3 バケット `favnir-terraform-state` が存在する（他モジュールで作成済み）

## 変数

| 変数名 | 型 | 説明 | デフォルト |
|---|---|---|---|
| `aws_region` | string | AWS リージョン | `ap-northeast-1` |
| `sap_base_url` | string | SAP OData v4 ベース URL | 必須 |
| `sap_client` | string | SAP クライアント番号 | `100` |
| `sap_auth` | string | 認証方式（`basic` \| `oauth2`） | `basic` |
| `sap_username` | string | SAP ユーザー名（機密） | 必須 |
| `sap_password` | string | SAP パスワード（機密） | 必須 |

## セットアップ手順

機密値（`sap_username` / `sap_password`）は環境変数経由で渡すことを推奨する。
`terraform.tfvars` に平文で書くと git にコミットするリスクがあるため注意。

```bash
cd infra/sap

# 1. 初期化
terraform init

# 2-A. 環境変数経由（推奨 — 機密値を tfvars に書かない）
export TF_VAR_sap_base_url="https://my-s4hana.example.com/sap/opu/odata/sap"
export TF_VAR_sap_username="FAVNIR_USER"
export TF_VAR_sap_password="<actual-password>"

# 2-B. 変数ファイル経由（git に含めないこと — .gitignore に terraform.tfvars を追加）
cat > terraform.tfvars <<EOF
sap_base_url = "https://my-s4hana.example.com/sap/opu/odata/sap"
sap_username = "FAVNIR_USER"
sap_password = "<actual-password>"
EOF

# 3. プラン確認
terraform plan

# 4. 適用
terraform apply
```

## 作成される SSM パラメータ

| SSM パス | 型 | 説明 |
|---|---|---|
| `/favnir/sap/base_url` | String | SAP OData v4 ベース URL |
| `/favnir/sap/client` | String | SAP クライアント番号 |
| `/favnir/sap/auth` | String | 認証方式 |
| `/favnir/sap/username` | SecureString | SAP ユーザー名 |
| `/favnir/sap/password` | SecureString | SAP パスワード |

## Favnir からの利用方法

`fav.toml` の `[sap]` セクションで設定するか、環境変数で直接指定する。

```toml
[sap]
base_url = "${SAP_BASE_URL}"
client   = "100"
username = "${SAP_USER}"
password = "${SAP_PASS}"
auth     = "basic"
```

SSM から環境変数にエクスポートする場合:

```bash
export SAP_BASE_URL=$(aws ssm get-parameter --name /favnir/sap/base_url --query Parameter.Value --output text)
export SAP_USER=$(aws ssm get-parameter --name /favnir/sap/username --with-decryption --query Parameter.Value --output text)
export SAP_PASS=$(aws ssm get-parameter --name /favnir/sap/password --with-decryption --query Parameter.Value --output text)
```
