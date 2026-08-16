# v75.8.0 タスクリスト — `fav time-travel` コマンド

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `75.7.0` であることを確認
- [x] `cargo test` が全 pass（3706 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v75.8.0: fav time-travel コマンド ---` コメントを追加する
- [x] `TimeTravelFormat` enum を追加する（Snowflake / Delta / Generic）
- [x] `TimeTravelQuery` 構造体を追加する（table: String, as_of_ts: i64, format: TimeTravelFormat）
- [x] `cmd_time_travel(query: &TimeTravelQuery) -> String` を追加する
  - `Snowflake`: `unix_secs_to_utc` を再利用して `AS OF TIMESTAMP 'YYYY-MM-DD HH:MM:SS'` を生成
  - `Delta`: `VERSION AS OF {as_of_ts}`
  - `Generic`: `WHERE _timestamp = {as_of_ts}`
- [x] `parse_time_travel_timestamp(s: &str) -> Result<i64, String>` を追加する
  - 長さ 20 / 区切り文字検証
  - year >= 1970 のみサポート
  - `is_leap` を再利用して 1970 からの日数を計算
  - `parse_time_travel_timestamp("2025-01-01T00:00:00Z") == Ok(1735689600)`
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v75.8.0 エントリを追加する
- [x] Added セクション（enum 1 件・struct 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v758000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `time_travel_snowflake_format` テストを実装する
  - `cmd_time_travel` が `"AS OF TIMESTAMP '2025-01-01 00:00:00'"` を含む
  - `cmd_time_travel` が `"orders"` を含む
  - `parse_time_travel_timestamp("2025-01-01T00:00:00Z") == Ok(1735689600)`
  - `parse_time_travel_timestamp("invalid").is_err()`
- [x] `time_travel_delta_format` テストを実装する
  - Delta: `"VERSION AS OF 1735689600"` を含む
  - Generic: `"WHERE _timestamp = 1735689600"` を含む
- [x] `cargo test v758000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"75.7.0"` → `"75.8.0"` に変更する
- [x] `driver.rs` 内の `75.7.0` バージョン文字列アサーションを `75.8.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v75.8.0 に更新する
- [x] 「次に切る版」を v75.9.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3708 tests）
- [x] `cargo test v758000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `75.8.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v75.8.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `time_travel_snowflake_format` が pass
- [x] `time_travel_delta_format` が pass
- [x] テスト総数: 3708（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（CLI 統合は将来バージョン）

---

## コードレビュー指摘と対応（code-reviewer）

| 優先度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | SQL インジェクション: `cmd_time_travel` が `validate_table_name` を呼ばない | spec 通り設計上の意図的相違（呼び出し側の責任）— 対応不要 |
| [HIGH] | `hour`/`min`/`sec` の上限チェックが欠落 | `hour > 23`、`min > 59`、`sec > 59` ガードを追加 |
| [MED] | 年ループと月累積ループで実装パターンが異なる | 正しく動作するため現状維持（コメント追記は低優先） |
| [LOW] | テーブル名インジェクションのテストが欠落 | 設計上 `cmd_time_travel` は Result を返さないため追加不可 |
| [LOW] | `time_travel_snowflake_format` が SQL 生成と timestamp パースを混在 | 許容範囲（必須変更ではない）— 現状維持 |
