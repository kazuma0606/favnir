# Tasks: v98.1.0 — `KpiDefinition<T>` / `KpiSnapshot<T>` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v98.0.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98000_tests` が存在することを確認する（v98.0.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,235 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: `runes/sap-odata/analytics.fav` を新規作成

- [x] `KpiThreshold` レコード型を定義する（`warning: Float` / `critical: Float`）
- [x] `KpiDefinition<T>` ジェネリックレコード型を定義する
- [x] `KpiStatus` バリアント型を定義する（`Ok` / `Warning(Float)` / `Critical(Float)`）
- [x] `KpiSnapshot<T>` ジェネリックレコード型を定義する
- [x] `measure_kpi_status(kpi: KpiDefinition<T>, value: Float) -> KpiStatus` ヘルパー関数を実装する
- [x] `make_kpi_snapshot(kpi: KpiDefinition<T>, value: Float, measured_at: String) -> KpiSnapshot<T>` ヘルパー関数を実装する

## T2: `fav/src/driver.rs` に `mod v98100_tests` を追加

- [x] `mod v98000_tests` の直後に `mod v98100_tests`（2 テスト）を追加する:
  - `analytics_fav_exists`: `../runes/sap-odata/analytics.fav` の存在を確認
  - `analytics_fav_has_kpi_definition`: `KpiDefinition` が含まれることを確認

## T2b: `versions/roadmap/roadmap-v98.1-v99.0.md` テスト数修正

- [x] `## 前提` の baseline を `4,230` → `4,235` に修正する（実績値）
- [x] バージョン一覧表の全テスト数を +5 オフセットで修正する（v98.1.0: 4237〜v99.0.0: 4257）
- [x] スプリント終了時チェックリストの `4,252` → `4,257` に修正する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,237 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v98.1.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.1.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v98.1.0` に更新する
- [x] 最新安定版を `v98.1.0` に更新する（テスト数 4,237）

<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->
<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
