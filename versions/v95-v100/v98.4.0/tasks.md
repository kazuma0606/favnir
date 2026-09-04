# Tasks: v98.4.0 — レポート自動生成 pipeline

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v98.3.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98300_tests` が存在することを確認する（v98.3.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,241 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T0b: ロードマップ注記の整理（実装前に確認）

- [x] `roadmap-v98.1-v99.0.md` の `## v98.4.0` 末尾の「Effect::SapAnalytics を Effect enum に追加」注記が、
  Effect enum 削除済み（v35.4.0）という現実と乖離していることを認識した上で、
  本バージョンでは `effect_catalog.rs` への文字列定数追加をもって代替実装とすることを確認する

## T1: effect_catalog.rs に SAP_ANALYTICS 定数を追加

- [x] `fav/src/effect_catalog.rs` に `pub const SAP_ANALYTICS: &str = "SapAnalytics";` を追記する
- [x] ドキュメントコメント（`///`）が付いていることを確認する

## T2: checker.fav の ns_to_effect に "Sac" ブランチを追加

- [x] `fav/self/checker.fav` の `ns_to_effect` 関数内、`if ns == "Grafana"` ブロック直前に `if ns == "Sac" { "SapAnalytics" }` を追加する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T3: sac.fav に report_to_sac_rows を追加

- [x] `runes/sap-odata/sac.fav` に `report_to_sac_rows(report: sales_order.SalesReport) -> Result<List<String>, String>` を追記する
- [x] フィールド参照が `report.report_date`（`date` ではなく）/ `report.by_currency`（`totals` ではなく）であることを確認する
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）
- [x] `runes/sap-odata/sap_odata.fav` に `report_to_sac_rows` の re-export を追記する
  - `-- SAC re-export（v98.3.0〜）` ブロック内（`SacDataset` / `sac_push_mock` の直後）

## T4: pipeline_analytics.fav を新規作成

- [x] `infra/e2e-demo/sap-odata/pipeline_analytics.fav` を新規作成する
- [x] `daily_sales_report` pipeline（`!SapOData !SapAnalytics`）が含まれることを確認する
- [x] ステージが Fetch / Aggregate / Push の 3 段構成であることを確認する
- [x] `build_sales_report` の第 1 引数（`date: String`）に文字列リテラルを渡していることを確認する（`today()` 関数は存在しないため使わない）
- [x] コメントが `--` スタイルであることを確認する（`//` 不可）

## T5: driver.rs に mod v98400_tests を追加

- [x] `mod v98300_tests` の直後に `mod v98400_tests`（2 テスト）を追加する:
  - `pipeline_analytics_fav_exists`: `../infra/e2e-demo/sap-odata/pipeline_analytics.fav` の存在を確認
  - `pipeline_analytics_has_daily_sales_report`: `daily_sales_report` が含まれることを確認
- [x] `mod v98400_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T6: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,243 tests, 0 failures であることを確認する

## T7: CHANGELOG.md に v98.4.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.4.0]` エントリを追加する

## T8: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v98.4.0` に更新する
- [x] 最新安定版を `v98.4.0` に更新する（テスト数 4,243）

<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T6 の `cargo test` 全 pass 確認後・T7/T8 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
