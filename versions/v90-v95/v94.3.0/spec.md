# Spec: v94.3.0 — Lambda SnapStart 対応 Terraform

## Background

AWS Lambda SnapStart は、Lambda 関数の初期化済みスナップショットをキャッシュし、
コールドスタート時間を大幅に削減する機能（主に Java ランタイム向け）。

SAP OData 連携 Lambda（`favnir-sap-sync`）はコールドスタート時に SAP 接続確立・
認証トークン取得・型マッピング初期化が発生し、コールドスタートに数秒かかる。
SnapStart を有効にすることで P50 コールドスタートを ~93% 削減できる。

v94.3.0 では `infra/lambda/sap-sync/` に Terraform 設定を追加する。

## Goals

1. `infra/lambda/sap-sync/main.tf` — Lambda 関数リソース（SnapStart 有効）を定義する
2. `infra/lambda/sap-sync/variables.tf` — 変数定義（SAP 接続情報等）を追加する
3. `infra/lambda/sap-sync/outputs.tf` — 出力値定義（Lambda ARN 等）を追加する

## Syntax/API Examples

```hcl
# infra/lambda/sap-sync/main.tf（抜粋）
resource "aws_lambda_function" "sap_sync" {
  function_name = "favnir-sap-sync"
  runtime       = "java21"
  snap_start {
    apply_on = "PublishedVersions"
  }
  environment {
    variables = {
      SAP_BASE_URL   = var.sap_base_url
      SAP_CLIENT_ID  = var.sap_client_id
    }
  }
}
```

```hcl
# infra/lambda/sap-sync/variables.tf
variable "sap_base_url" {
  description = "SAP S/4HANA OData base URL"
  type        = string
}

variable "sap_client_id" {
  description = "SAP client ID"
  type        = string
  default     = "100"
}

variable "sap_user" {
  description = "SAP username"
  type        = string
  sensitive   = true
}

variable "sap_pass" {
  description = "SAP password"
  type        = string
  sensitive   = true
}

variable "lambda_role_arn" {
  description = "IAM role ARN for Lambda execution"
  type        = string
}
```

```hcl
# infra/lambda/sap-sync/outputs.tf
output "lambda_arn" {
  description = "SAP sync Lambda function ARN"
  value       = aws_lambda_function.sap_sync.arn
}

output "lambda_function_name" {
  description = "SAP sync Lambda function name"
  value       = aws_lambda_function.sap_sync.function_name
}
```

## Success Criteria

- `infra/lambda/sap-sync/` ディレクトリが存在する
- `infra/lambda/sap-sync/main.tf` に `snap_start` が含まれる
- `driver.rs` の `mod v94300_tests` が pass する
  - `lambda_sap_sync_infra_exists`: `../infra/lambda/sap-sync` ディレクトリが存在することを確認
  - `lambda_sap_sync_has_snap_start`: `main.tf` に `snap_start` が含まれることを確認
- `cargo test 2>&1 | grep "test result"` が 4,148 tests, 0 failures を示す（着手前: 4,146）
- `cargo clippy --locked -- -D warnings` が pass する

## Error Codes

なし（Terraform ファイル追加のみ、Rust エラーコードへの影響なし）

## Files to Modify / Create

| ファイル | 操作 | 内容 |
|---|---|---|
| `infra/lambda/sap-sync/main.tf` | **新規作成** | Lambda 関数定義（SnapStart 有効） |
| `infra/lambda/sap-sync/variables.tf` | **新規作成** | 変数定義 |
| `infra/lambda/sap-sync/outputs.tf` | **新規作成** | 出力値定義 |
| `fav/src/driver.rs` | **追加** | `mod v94300_tests`（2 件） |
| `CHANGELOG.md` | **追記** | v94.3.0 エントリ |
