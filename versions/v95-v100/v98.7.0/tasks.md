# Tasks: v98.7.0 — E2E デモ（日次 KPI → SAC → Slack）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v98.6.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98600_tests` が存在することを確認する（v98.6.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,247 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: pipeline_kpi_monitor.fav を新規作成

- [x] `infra/e2e-demo/sap-odata/analytics_demo/` ディレクトリを作成する
- [x] `pipeline_kpi_monitor.fav` を新規作成する
- [x] `kpi_monitor` pipeline（`!SapOData !SapAnalytics`）が 4 ステージ（Fetch / Evaluate / Push / Alert）で実装されていることを確認する
- [x] `KpiAlert` が使用されていることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: run.sh を新規作成

- [x] `infra/e2e-demo/sap-odata/analytics_demo/run.sh` を新規作成する
- [x] `#!/usr/bin/env bash` + `set -euo pipefail` で始まることを確認する
- [x] `fav run "${SCRIPT_DIR}/pipeline_kpi_monitor.fav"` を実行する内容になっていることを確認する

## T3: README.md を新規作成

- [x] `infra/e2e-demo/sap-odata/analytics_demo/README.md` を新規作成する
- [x] デモ概要・前提条件・実行手順（`bash run.sh`）・pipeline フロー図を含む

## T4: driver.rs に mod v98700_tests を追加

- [x] `mod v98600_tests` の直後に `mod v98700_tests`（2 テスト）を追加する:
  - `analytics_demo_pipeline_exists`: `analytics_demo/pipeline_kpi_monitor.fav` の存在を確認
  - `pipeline_kpi_monitor_has_kpi_alert`: `KpiAlert` が含まれることを確認
- [x] `mod v98700_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T5: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,249 tests, 0 failures であることを確認する

## T6: CHANGELOG.md に v98.7.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.7.0]` エントリを追加する

## T7: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v98.7.0` に更新する
- [x] 最新安定版を `v98.7.0` に更新する（テスト数 4,249）

<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後・T6/T7 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
