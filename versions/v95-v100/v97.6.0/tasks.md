# Tasks: v97.6.0 — E2E デモ（発注 → 承認 → SAP 反映）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.5.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97500_tests` が存在することを確認する（v97.5.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,223 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `infra/e2e-demo/sap-odata/workflow_demo/README.md` を新規作成

- [x] デモ概要（発注 → 自動承認ルーティング → SAP 反映、`route_by_approval_result` pipeline）を記載する
- [x] 前提条件（fav CLI インストール済み）を記載する
- [x] 実行手順（`bash run.sh`）を記載する

## T2: `infra/e2e-demo/sap-odata/workflow_demo/run.sh` を新規作成

- [x] `#!/usr/bin/env bash` + `set -euo pipefail` を記述する
- [x] `fav run ../pipeline_workflow.fav` を実行するスクリプトを記述する

## T3: `fav/src/driver.rs` に `mod v97600_tests` を追加

- [x] `mod v97500_tests` の直後に `#[cfg(test)] mod v97600_tests { ... }` を追加する
- [x] `workflow_demo_readme_exists` テストを追加する（README.md の存在確認）
- [x] `workflow_demo_run_sh_has_fav_run` テストを追加する（run.sh に `fav run` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,225 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v97.6.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.6.0]` エントリを追加する

## T6: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v97.6.0` に更新する
- [x] 最新安定版を `v97.6.0` に更新する（テスト数 4,225）

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
