# v75.8.0 仕様書 — `fav time-travel` コマンド

Date: 2026-08-15
Status: 計画中

---

## Background

v75.2.0 で実装した `AsOfQuery` / `format_as_of_query` は単一のクエリを生成するが、Snowflake / Delta / 汎用クエリの形式切り替えができなかった。v75.8.0 では CLI 向けのタイムトラベルクエリ生成 API を追加し、複数の SQL 方言に対応した SQL 文字列を型安全に生成する基盤を提供する。

---

## Goals

1. `TimeTravelFormat` enum（Snowflake, Delta, Generic）を追加する
2. `TimeTravelQuery` 構造体（table: String, as_of_ts: i64, format: TimeTravelFormat）を追加する
3. `cmd_time_travel(query: &TimeTravelQuery) -> String` — SQL 文字列生成を追加する
4. `parse_time_travel_timestamp(s: &str) -> Result<i64, String>` — RFC3339 UTC パースを追加する
5. Rust テスト 2 件を追加し 3708 tests に到達する

---

## 型・関数仕様

### `TimeTravelFormat` enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TimeTravelFormat {
    Snowflake,  // AS OF TIMESTAMP 'YYYY-MM-DD HH:MM:SS'
    Delta,      // VERSION AS OF <epoch_secs>
    Generic,    // WHERE _timestamp = <epoch_secs>
}
```

---

### `TimeTravelQuery` 構造体

```rust
#[derive(Debug, Clone)]
pub struct TimeTravelQuery {
    pub table:    String,           // テーブル名（SQL インジェクション検証は呼び出し側の責任）
    pub as_of_ts: i64,              // 基準 Unix epoch 秒
    pub format:   TimeTravelFormat, // SQL 方言
}
```

---

### `cmd_time_travel`

```rust
pub fn cmd_time_travel(query: &TimeTravelQuery) -> String
```

**出力フォーマット:**
- `Snowflake`: `"SELECT * FROM {table} AS OF TIMESTAMP 'YYYY-MM-DD HH:MM:SS'"`
  — `as_of_ts` を UTC 日時に変換（既存の `unix_secs_to_utc` を再利用）
- `Delta`: `"SELECT * FROM {table} VERSION AS OF {as_of_ts}"`
- `Generic`: `"SELECT * FROM {table} WHERE _timestamp = {as_of_ts}"`

---

### `parse_time_travel_timestamp`

```rust
pub fn parse_time_travel_timestamp(s: &str) -> Result<i64, String>
```

**サポートフォーマット:** `"YYYY-MM-DDTHH:MM:SSZ"`（UTC のみ、タイムゾーンオフセット不可）

**変換ロジック:**
1. 文字列の長さ・区切り文字（`-`, `T`, `:`, `Z`）を検証
2. 年・月・日・時・分・秒を `str::parse` で取得
3. 月（1〜12）・日（1〜28/29/30/31）の範囲を検証（範囲外は `Err`）
4. 1970-01-01 からの日数を計算（`is_leap` を再利用）
5. `days * 86400 + hour * 3600 + min * 60 + sec` を返す

**制限:**
- `year >= 1970` のみサポート（それ以前はエラー）
- テーブル名の SQL インジェクション検証は `cmd_time_travel` の呼び出し側の責任

---

## CLI イメージ（将来統合）

```bash
$ fav time-travel --table orders --at "2025-01-01T00:00:00Z"
SELECT * FROM orders AS OF TIMESTAMP '2025-01-01 00:00:00'

$ fav time-travel --table orders --at "2025-01-01T00:00:00Z" --format delta
SELECT * FROM orders VERSION AS OF 1735689600
```

---

## Success Criteria

- `TimeTravelFormat` enum が定義されている（Snowflake / Delta / Generic）
- `TimeTravelQuery` 構造体が定義されている
- `cmd_time_travel` が各フォーマットで正しい SQL を生成する
- `parse_time_travel_timestamp("2025-01-01T00:00:00Z")` が `Ok(1735689600)` を返す
- 無効な文字列が `Err` を返す
- `cargo test` が 3708 tests all pass
- `CHANGELOG.md` の先頭に v75.8.0 エントリが存在する

---

## テスト仕様

### `time_travel_snowflake_format`

- `query = TimeTravelQuery { table: "orders", as_of_ts: 1735689600, format: TimeTravelFormat::Snowflake }`
- `cmd_time_travel(&query)` が `"AS OF TIMESTAMP '2025-01-01 00:00:00'"` を含む
- `cmd_time_travel(&query)` が `"orders"` を含む
- `parse_time_travel_timestamp("2025-01-01T00:00:00Z") == Ok(1735689600)`
- `parse_time_travel_timestamp("invalid")` が `Err` を返す

### `time_travel_delta_format`

- `query = TimeTravelQuery { table: "orders", as_of_ts: 1735689600, format: TimeTravelFormat::Delta }`
- `cmd_time_travel(&query)` が `"VERSION AS OF 1735689600"` を含む
- `query2 = TimeTravelQuery { table: "orders", as_of_ts: 1735689600, format: TimeTravelFormat::Generic }`
- `cmd_time_travel(&query2)` が `"WHERE _timestamp = 1735689600"` を含む

---

## 変更ファイル

- `fav/src/driver.rs` — `TimeTravelFormat`, `TimeTravelQuery`, `cmd_time_travel`, `parse_time_travel_timestamp`, `v758000_tests` を追加
- `CHANGELOG.md` — v75.8.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.7.0` → `75.8.0` に更新

---

## 依存（既実装）

- `unix_secs_to_utc(epoch_secs: i64) -> (i32, u32, u32, u32, u32, u32)` — v75.2.0 で実装済み（Snowflake フォーマット生成に使用）。**可視性: `pub(crate)`** — `cmd_time_travel` は同一ファイル（`driver.rs`）内に実装するため問題ない。
- `is_leap(year: i32) -> bool` — v75.2.0 で実装済み（`parse_time_travel_timestamp` の日数計算に使用）

---

## 対象外

- タイムゾーンオフセット（`+09:00` 等）のパース（UTC `Z` のみ）
- テーブル名の SQL インジェクション検証（`validate_table_name` の呼び出しは CLI 側の責任）。`format_as_of_query`（v75.2.0）は内部で `validate_table_name` を呼ぶが、`cmd_time_travel` は呼ばない（CLI 統合層の責任とする設計上の意図的相違）。
- 実際の `fav time-travel` CLI サブコマンド統合（将来バージョン。統合時に `validate_table_name` の呼び出しを CLI 側で追加すること）
- site/ MDX 追加（CLI 統合バージョンで行う）
