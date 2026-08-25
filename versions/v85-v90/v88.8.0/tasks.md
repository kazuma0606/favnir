# Tasks: v88.8.0 — E2E デモ Lambda 基盤

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,011 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88700_tests` が存在することを確認する（v88.7.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（v89.0.0 宣言バージョンまでバンプしない設計のため、88.0.0 が正しい）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` が存在することを確認する（デモ基盤の前提ファイル）

## T1: Terraform ファイル作成

- [x] `infra/e2e-demo/sap-odata/terraform/variables.tf` を新規作成する
- [x] `infra/e2e-demo/sap-odata/terraform/ssm.tf` を新規作成する
- [x] `infra/e2e-demo/sap-odata/terraform/main.tf` を新規作成する（Lambda + IAM + S3、`favnir-sap-e2e-demo` 含む）
- [x] `infra/e2e-demo/sap-odata/lambda/bootstrap.zip.placeholder` を新規作成する（bootstrap.zip のプレースホルダ。本実装は v88.9.0 安定化スプリントで整備）

## T2: 実行スクリプト作成

- [x] `infra/e2e-demo/sap-odata/scripts/run.sh` を新規作成する（`lambda` を含む）
- [x] `chmod +x infra/e2e-demo/sap-odata/scripts/run.sh` を実行する

## T3: `driver.rs` に `mod v88800_tests` を追加

- [x] `mod v88700_tests { ... }` の直後に `#[cfg(test)] mod v88800_tests { ... }` を追加する
- [x] `sap_e2e_demo_terraform_exists` テストを実装する（`main.tf` に `"favnir-sap-e2e-demo"` を確認）
- [x] `sap_e2e_demo_run_script_exists` テストを実装する（`run.sh` に `"lambda"` を確認（小文字変換後））

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,013 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 指摘対応

- [HIGH] SAP_PASSWORD を Lambda 環境変数にプレーンテキスト渡し → SSM パラメータパス（`_SSM_PATH` サフィックス）を渡す方式に変更。ssm.tf の `with_decryption` も全て `false` に変更（値不要）。
- [MED] IAM ロール名・ポリシー名・Lambda 関数名に `${var.environment}` を追加して環境間重複を防止
- [MED] 実在しない `bootstrap.zip` で apply 失敗 → Lambda リソースに `count = 0` を追加（v88.9.0 で 1 に変更）
- [LOW] SSM ARN のワイルドカードアカウント ID → `data.aws_caller_identity.current.account_id` に変更
- [LOW] `with_decryption = false` を全 SSM パラメータに明示
- [LOW] run.sh に `--cli-binary-format raw-in-base64-out` を追加（AWS CLI v2 互換）
- [LOW] run.sh の `AWS_ENDPOINT_URL` を `ENDPOINT_ARGS` 配列で明示的に `--endpoint-url` へ渡す

## spec-reviewer 指摘対応

- [HIGH] T0 Cargo.toml バージョン確認の意図を明記（v89.0.0 前は 88.0.0 が正しい旨を追記）
- [MED] `lambda/bootstrap.zip.placeholder` を T1・spec に追加（本実装は v88.9.0 と明記）
- [MED] spec.md に LocalStack 方針（`AWS_ENDPOINT_URL` 切り替え）を追記
- [LOW] tasks.md T2 に `chmod +x` ステップを追加
