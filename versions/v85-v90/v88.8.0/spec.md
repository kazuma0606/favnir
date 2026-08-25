# Spec: v88.8.0 — E2E デモ Lambda 基盤

## Background

v88.1.0〜v88.7.0 で SAP OData Rune の型定義・関数スタブが揃った。
本バージョンでは SAP パイプラインを AWS Lambda で実行するデモ基盤を整備する。
既存の `infra/e2e-demo/sap-odata/pipeline.fav` を Lambda 上で動作させることを想定した
Terraform 構成と実行スクリプトを追加する。

## Goals

1. `infra/e2e-demo/sap-odata/terraform/main.tf` — Lambda + IAM + S3 出力バケット
2. `infra/e2e-demo/sap-odata/terraform/ssm.tf` — SSM パラメータ参照
3. `infra/e2e-demo/sap-odata/terraform/variables.tf` — 変数定義
4. `infra/e2e-demo/sap-odata/scripts/run.sh` — デモ実行スクリプト
5. Rust テスト 2 件で上記ファイルの存在を担保する

## Infrastructure Design

### Lambda 構成（main.tf）

- **Lambda 関数**: `favnir-sap-e2e-demo`（ランタイム: provided.al2）
- **IAM ロール**: `favnir-sap-e2e-demo-role`（LambdaBasicExecution + SSM + S3 権限）
- **S3 バケット**: `favnir-sap-e2e-demo-output`（パイプライン結果出力先）
- **LocalStack 対応**: `AWS_ENDPOINT_URL` 環境変数で LocalStack / 本番を切り替える（`provider "aws"` の `endpoints` ブロックは使用しない）
- **`lambda/bootstrap.zip`**: 本バージョンではプレースホルダ（`lambda/bootstrap.zip.placeholder`）のみ作成。実バイナリの整備は v88.9.0 安定化スプリントで行う。

### SSM パラメータ参照（ssm.tf）

- `/favnir/sap/base_url`
- `/favnir/sap/username`
- `/favnir/sap/password`
- `/favnir/sap/client`

### 変数（variables.tf）

- `aws_region`（default: `ap-northeast-1`）
- `environment`（default: `dev`）

### 実行スクリプト（scripts/run.sh）

Lambda を invoke し、出力を S3 から取得して表示する簡易スクリプト。

## Success Criteria（Rust テストで担保）

- `sap_e2e_demo_terraform_exists`:
  `infra/e2e-demo/sap-odata/terraform/main.tf` が存在し、`"favnir-sap-e2e-demo"` を含む
- `sap_e2e_demo_run_script_exists`:
  `infra/e2e-demo/sap-odata/scripts/run.sh` が存在し、`"lambda"` を含む（大文字小文字問わず）
- `cargo test` で 4,013 tests, 0 failures（4,011 + 2）

## Files to Modify / Create

| ファイル | 変更種別 |
|---|---|
| `infra/e2e-demo/sap-odata/terraform/main.tf` | 新規作成 |
| `infra/e2e-demo/sap-odata/terraform/ssm.tf` | 新規作成 |
| `infra/e2e-demo/sap-odata/terraform/variables.tf` | 新規作成 |
| `infra/e2e-demo/sap-odata/lambda/bootstrap.zip.placeholder` | 新規作成（プレースホルダ） |
| `infra/e2e-demo/sap-odata/scripts/run.sh` | 新規作成 |
| `fav/src/driver.rs` | `mod v88800_tests` 追加 |

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
