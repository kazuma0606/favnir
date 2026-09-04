# Tasks: v98.5.0 — KPI 閾値アラート + Slack / メール通知

Status: COMPLETE

## T0: 着手前チェックリスト

> 前提: v98.1.0〜v98.4.0 が完了済みであること（KpiDefinition / BwQuery / SacDataset / pipeline_analytics の実装）

- [x] `versions/current.md` の最新安定版が `v98.4.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98400_tests` が存在することを確認する（v98.4.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,243 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: analytics.fav に KpiAlert 型と format_kpi_alert を追加

- [x] `runes/sap-odata/analytics.fav` の末尾（`bw_query_mock<T>` の後）に `KpiAlert` 型を追記する
- [x] `format_kpi_alert(alert: KpiAlert) -> String` ヘルパーを追記する
  - `match alert.status { Ok -> "OK" / Warning(_) -> "WARNING" / Critical(_) -> "CRITICAL" }` を使う
  - 返却文字列の形式: `"[LEVEL] kpi_name: message"`
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T2: sap_odata.fav に KpiAlert / format_kpi_alert re-export を追加

- [x] `runes/sap-odata/sap_odata.fav` の Analytics re-export ブロック末尾（`bw_query_mock` の直後・`-- $batch` の前）に追記する
- [x] `public type KpiAlert = analytics.KpiAlert` を追加する
- [x] `public fn format_kpi_alert(alert: analytics.KpiAlert) -> String` を追加する

## T3: driver.rs に mod v98500_tests を追加

- [x] `mod v98400_tests` の直後に `mod v98500_tests`（2 テスト）を追加する:
  - `analytics_fav_has_kpi_alert`: `analytics.fav` に `KpiAlert` が含まれることを確認
  - `analytics_fav_has_format_kpi_alert`: `analytics.fav` に `format_kpi_alert` が含まれることを確認
- [x] `mod v98500_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T4: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,245 tests, 0 failures であることを確認する

## T5: CHANGELOG.md に v98.5.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.5.0]` エントリを追加する

## T6: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v98.5.0` に更新する
- [x] 最新安定版を `v98.5.0` に更新する（テスト数 4,245）

<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
