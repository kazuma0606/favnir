# Tasks: v85.8.0 — SSM Parameter Store 設定（`infra/sap/`）

Status: COMPLETE

> MILESTONE.md / README.md / `site/content/docs/` の更新は v86.0.0 宣言バージョンで実施する。
> 本バージョンは `infra/sap/` Terraform ファイルの作成と Rust テスト追加のみ。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,945 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v85700_tests` が存在することを確認する（v85.7.0 完了済みの証拠）
- [x] `infra/snowflake/ssm.tf` が存在することを確認する（パターン参照用）

## T1: `infra/sap/` Terraform ファイルを作成

- [x] `infra/sap/providers.tf` を作成する
  - AWS provider（`~> 5.0`）+ S3 backend（`sap/terraform.tfstate`）
- [x] `infra/sap/variables.tf` を作成する
  - `aws_region` / `sap_base_url` / `sap_client` / `sap_auth` / `sap_username` / `sap_password`
  - `sap_username` / `sap_password` は `sensitive = true`
- [x] `infra/sap/ssm.tf` を作成する
  - `sap_base_url`（String）/ `sap_client`（String）/ `sap_auth`（String）
  - `sap_username`（SecureString + `lifecycle { ignore_changes = [value] }`）
  - `sap_password`（SecureString + `lifecycle { ignore_changes = [value] }`）
  - タグ: `Project = "favnir"` / `ManagedBy = "terraform"`
- [x] `infra/sap/outputs.tf` を作成する
  - `ssm_prefix`（`"/favnir/sap/"`）
  - `sap_base_url_ssm_name`
- [x] `infra/sap/README.md` を作成する
  - 前提条件・変数設定手順・`terraform init / plan / apply` コマンドを記述

## T2: `mod v85800_tests` を追加

- [x] `mod v85700_tests { ... }` の直後に `#[cfg(test)] mod v85800_tests { ... }` を追加する
- [x] `sap_infra_ssm_tf_exists` テストを実装する
  - `Path::new("../infra/sap/ssm.tf").exists()` が `true` であることを確認
- [x] `sap_infra_readme_exists` テストを実装する
  - `Path::new("../infra/sap/README.md").exists()` が `true` であることを確認

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,947 tests, 0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v85.8.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [SECURITY][MED] `ssm.tf` に String/SecureString 採用根拠コメントを追加（Snowflake パターンと同じ方針）
- [SECURITY][LOW] README の `"changeme"` を `"<actual-password>"` に変更、`TF_VAR_*` 環境変数経由の代替手順を追記
- [BUG][MED] `lifecycle` 省略が意図的設計であることをコメントで明示（String 型には付けない）
- [STYLE][LOW] `outputs.tf` に `sap_client` / `sap_auth` / `sap_username` / `sap_password` の SSM 名出力を追加
- [STYLE][LOW] `providers.tf` の backend ハードコード理由コメントを追加（Terraform の制約）
