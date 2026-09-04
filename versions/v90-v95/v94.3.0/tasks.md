# Tasks: v94.3.0 — Lambda SnapStart 対応 Terraform

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,146 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94200_tests` が存在することを確認する（v94.2.0 完了済みの証拠）
- [x] `runes/sap-odata/batch.fav` に `ChangeSet` が含まれることを確認する（v94.2.0 完了済みの証拠）

## T1: `infra/lambda/sap-sync/` ディレクトリを作成する

- [x] `infra/lambda/sap-sync/` ディレクトリを作成する

## T2: `infra/lambda/sap-sync/main.tf` を作成する

- [x] `main.tf` を新規作成する（`aws_lambda_function.sap_sync` リソース）
- [x] `snap_start { apply_on = "PublishedVersions" }` が含まれていることを確認する
- [x] 環境変数（SAP_BASE_URL / SAP_CLIENT_ID / SAP_USER / SAP_PASS）が定義されていることを確認する

## T3: `infra/lambda/sap-sync/variables.tf` を作成する

- [x] `variables.tf` を新規作成する（sap_base_url / sap_client_id / sap_user / sap_pass / lambda_role_arn）
- [x] sensitive = true の変数（sap_user / sap_pass）が正しく設定されていることを確認する

## T4: `infra/lambda/sap-sync/outputs.tf` を作成する

- [x] `outputs.tf` を新規作成する（lambda_arn / lambda_function_name）

## T5: `driver.rs` に `mod v94300_tests` を追加する

- [x] `mod v94200_tests { ... }` の直後に `#[cfg(test)] mod v94300_tests { ... }` を追加する（2 テスト）
- [x] `lambda_sap_sync_infra_exists`: `../infra/lambda/sap-sync` ディレクトリが存在することを確認する
- [x] `lambda_sap_sync_has_snap_start`: `main.tf` に `snap_start` が含まれることを確認する

## T6: `CHANGELOG.md` に v94.3.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.3.0 エントリを追加する

## T7: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T8: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,148 tests, 0 failures であることを確認する

## T-last: CI 事前確認（T8 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T9: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
