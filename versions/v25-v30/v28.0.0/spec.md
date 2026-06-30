# v28.0.0 Spec — Data Lakehouse マイルストーン宣言

## 概要

v27.1〜v27.9 で実装した Data Lakehouse スタックの完成を宣言するマイルストーンバージョン。
「Delta Lake / Iceberg テーブルの読み書き、dbt モデル参照、主要 DWH 接続、SQLite 組み込み DB」
の全コンポーネントが揃ったことを MILESTONE.md・README・サイトドキュメントで正式記録する。

## 背景

| マイルストーン | バージョン | 内容 |
|---|---|---|
| Practical Self-Hosting | v25.0.0 | compiler.fav が自分自身をコンパイル |
| Streaming Native | v27.0.0 | リアルタイムパイプライン型安全対応 |
| **Data Lakehouse** | **v28.0.0** | 現代データ基盤への完全統合 |

## 完了コンポーネント（v27.1〜v27.9）

| Rune / 機能 | バージョン | 状態 |
|---|---|---|
| delta-lake Rune | v27.1 | COMPLETE |
| iceberg Rune | v27.2 | COMPLETE |
| clickhouse Rune | v27.3 | COMPLETE |
| bigquery Rune | v27.4 | COMPLETE |
| redshift Rune | v27.5 | COMPLETE |
| jsonl Rune | v27.6 | COMPLETE |
| `fav infer --from delta/iceberg` | v27.7 | COMPLETE |
| dbt 連携 Rune | v27.8 | COMPLETE |
| sqlite Rune | v27.9 | COMPLETE |

## 実装内容

### T0 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version` を `"27.9.0"` → `"28.0.0"` に更新。

### T1 — MILESTONE.md 更新
`MILESTONE.md` に "Data Lakehouse" セクションを追加。
- 完了した Rune 一覧（9 コンポーネント）
- 象徴的なデモコード（DeltaLake + Dbt + SQLite の連鎖）
- v28.x 残件（rusqlite 実統合 / delta-rs 実統合）

### T2 — README.md 更新
`README.md` に v28.0 "Data Lakehouse" 参照を追記。

### T3 — サイトドキュメント
`site/content/docs/data-lakehouse.mdx` 新規作成。

### T4 — roadmap 完了マーク
`versions/roadmap/roadmap-v27.1-v28.0.md` に完了マークを追記。

### T5 — CHANGELOG
`CHANGELOG.md` に `[v28.0.0]` セクション追加。

### T6 — ベンチマーク
`benchmarks/v28.0.0.json` 新規作成（test_count: 2226）。

### T7 — driver.rs テスト（6 件）

```
v280000_tests:
  milestone_md_mentions_data_lakehouse
  milestone_md_lists_sqlite_rune
  milestone_md_lists_dbt_rune
  readme_mentions_v28
  site_data_lakehouse_page_exists
  changelog_has_v28_0_0
```

> sqlite（組み込み DB の完成）と dbt（データ変換標準との統合）をマイルストーンの
> 象徴として個別確認。他 6 Rune（delta-lake / iceberg / clickhouse / bigquery /
> redshift / jsonl）は v27.1〜v27.6 の各テストモジュールで検証済みのため重複しない。

## テスト数

- v27.9.0: 2220 tests
- v28.0.0: **2226 tests**（+6）

## 完了条件

- [ ] `Cargo.toml` version = "28.0.0"
- [ ] MILESTONE.md に "Data Lakehouse" セクションあり
- [ ] README.md に `v28.0` または `v28.0.0` の記述あり
- [ ] `site/content/docs/data-lakehouse.mdx` 存在
- [ ] `CHANGELOG.md` に `[v28.0.0]` セクションあり
- [ ] `benchmarks/v28.0.0.json` 存在（test_count: 2226）
- [ ] `cargo test --bin fav v280000` が 6/6 PASS
- [ ] `cargo test --bin fav` 全体が 2226 tests PASS
