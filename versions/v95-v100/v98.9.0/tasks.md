# Tasks: v98.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v98.8.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v98.8.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98800_tests` が存在することを確認する（v98.8.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,251 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: driver.rs に mod v98900_tests を追加

- [x] `mod v98800_tests` の直後に `mod v98900_tests`（2 テスト）を追加する:
  - `sap_odata_rune_exports_kpi_alert`: `runes/sap-odata/sap_odata.fav` に `KpiAlert` が含まれることを確認
  - `analytics_demo_run_script_exists`: `analytics_demo/run.sh` の存在を確認
- [x] `mod v98900_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T2: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,253 tests, 0 failures であることを確認する

## T3: CHANGELOG.md に v98.9.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.9.0]` エントリを追加する

## T4: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v98.9.0` に更新する
- [x] 最新安定版を `v98.9.0` に更新する（テスト数 4,253）

<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v98.8.0 で対応済み -->

## T-last: CI 事前確認（コードフリーズ確認。T2 の `cargo test` 全 pass 確認後・T3/T4 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
