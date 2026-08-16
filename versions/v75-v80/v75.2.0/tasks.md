# v75.2.0 タスクリスト — `TemporalRange` / `AsOfQuery` 型

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.1.0` であることを確認
- [x] `cargo test` が全 pass（3694 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型定義・関数追加

- [x] `fav/src/driver.rs` に `TemporalRange` 構造体を追加する（from_ts: i64, to_ts: i64）
- [x] `AsOfQuery` 構造体を追加する（table: String, as_of_ts: i64）
- [x] `is_in_range(ts: i64, range: &TemporalRange) -> bool` を追加する（閉区間判定）
- [x] `format_as_of_query(q: &AsOfQuery) -> String` を追加する（Snowflake SQL 生成）
- [x] `unix_secs_to_utc` / `days_to_ymd` / `is_leap` ヘルパー関数を追加する
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.2.0 エントリを追加する
- [x] Added セクション（4つの型・関数）と Tests セクション（2件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v752000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `temporal_range_filters_correctly` テストを実装する
  - 下限・中間・上限のそれぞれで `is_in_range` が正しい結果を返すことを assert
  - 範囲外（below / above）も assert
- [x] `as_of_query_generates_sql` テストを実装する
  - `as_of_ts = 1_735_689_600`（2026-01-01 00:00:00 UTC）で SQL を生成
  - `SELECT * FROM orders`・`AS OF TIMESTAMP`・`2026-01-01`・`00:00:00` を assert
- [x] `cargo test v752000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.1.0"` → `"75.2.0"` に変更する
- [x] `driver.rs` 内の `75.1.0` バージョン文字列アサーションを `75.2.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.2.0 に更新する
- [x] 「次に切る版」を v75.3.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3696 tests）
- [x] `cargo test v752000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.2.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.2.0]` であることを確認する
- [x] site/ MDX 追加: 本バージョンは Rust 内部型基盤のみのため不要

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `temporal_range_filters_correctly` が pass
- [x] `as_of_query_generates_sql` が pass
- [x] テスト総数: 3696（+2）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | ID | 概要 | 対応 |
|---|---|---|---|
| [HIGH] | BUG-1 | 負の epoch で秒・分・時が誤った値になる | `div_euclid`/`rem_euclid` に修正済み |
| [HIGH] | SECURITY-1 | テーブル名無検証による SQL インジェクション | `validate_table_name` 追加、`format_as_of_query` を `Result<String, String>` に変更済み |
| [MED] | BUG-2 | `y as i32` の silent truncation | コメントで想定年範囲を明記済み |
| [MED] | STYLE-1 | `unix_secs_to_utc` の visibility | `pub(crate)` に変更済み |
| [LOW] | TEST-1 | 負の epoch テストケース欠如 | `as_of_query_generates_sql` に負の epoch テスト追加済み |
| [LOW] | STYLE-2 | `driver.rs` への配置 | 将来の課題として記録（現状 [LOW] のため未対応） |
