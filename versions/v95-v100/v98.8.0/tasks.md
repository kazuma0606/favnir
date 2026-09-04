# Tasks: v98.8.0 — サイトドキュメント（Analytics / KPI パターンガイド）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v98.7.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v98.7.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98700_tests` が存在することを確認する（v98.7.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,249 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: sap-analytics.mdx を新規作成

- [x] `site/content/docs/guides/sap-analytics.mdx` を新規作成する
- [x] フロントマターに `title: "SAP Analytics Guide"`、`order: 12`、`category: "Guide"` が含まれることを確認する
- [x] 概要セクション（KPI 監視 pipeline フロー図）が含まれることを確認する
- [x] KPI 定義パターンセクション（`KpiDefinition` / `KpiThreshold` / `KpiSnapshot` コード例）が含まれることを確認する
- [x] BW/4HANA クエリセクションが含まれることを確認する
- [x] SAC データプッシュセクション（`SacDataset` / `sac_push_mock`）が含まれることを確認する
- [x] `fav report --sap` コマンドリファレンスセクションが含まれることを確認する
- [x] コード例が `bind` 構文・`--` コメント・`|>` stage を使っていることを確認する

## T2: driver.rs に mod v98800_tests を追加

- [x] `mod v98700_tests` の直後に `mod v98800_tests`（2 テスト）を追加する:
  - `sap_analytics_guide_exists`: `site/content/docs/guides/sap-analytics.mdx` の存在を確認
  - `sap_analytics_guide_has_kpi_definition`: `KpiDefinition` が含まれることを確認
- [x] `mod v98800_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する

## T3: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,251 tests, 0 failures であることを確認する

## T4: CHANGELOG.md に v98.8.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.8.0]` エントリを追加する

## T5: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v98.8.0` に更新する
- [x] 最新安定版を `v98.8.0` に更新する（テスト数 4,251）

<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
