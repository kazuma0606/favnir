# Tasks: v89.5.0 — E2E デモ完成（4 シナリオ全実行 + Lambda デプロイ）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,027 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89400_tests` が存在することを確認する（v89.4.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する（v90.0.0 宣言バージョンまでバンプしない設計のため、89.0.0 が正しい）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` に `outstanding_payables` が存在することを確認する（v89.3.0 完了済みの証拠）
- [x] `infra/e2e-demo/sap-odata/scripts/run.sh` が存在することを確認する（v88.8.0 完了済みの証拠）
- [x] `scripts/start-sap-mock.sh` が存在することを確認する（`run-sap-demo.sh` の作成先階層の確認）

## T1: `scripts/run-sap-demo.sh` を作成

- [x] `scripts/run-sap-demo.sh` を新規作成する（`scripts/start-sap-mock.sh` と同階層）
- [x] `#!/usr/bin/env bash` + `set -euo pipefail` で始める
- [x] `SCRIPT_DIR` / `REPO_ROOT` で絶対パスを取得する
- [x] [1/3] docker compose でモックサーバーを起動する（`docker-compose.yml` 存在確認付き）
- [x] [2/3] `infra/e2e-demo/sap-odata/scripts/run.sh` を呼び出す
- [x] [3/3] `aws s3 ls s3://favnir-sap-demo/ --recursive` で S3 出力を確認する（`AWS_ENDPOINT_URL` 対応）
- [x] `chmod +x scripts/run-sap-demo.sh` で実行権限を付与する

## T2: `mod v89500_tests` を `driver.rs` に追加

- [x] `mod v89400_tests { ... }` の直後に `#[cfg(test)] mod v89500_tests { ... }` を追加する
- [x] `sap_e2e_demo_pipeline_has_journal_entry_scenario` テストを実装する（`pipeline.fav` に `"outstanding_payables"` を確認）
- [x] `sap_e2e_run_script_exists` テストを実装する（`"../scripts/run-sap-demo.sh"` の存在を確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,029 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
