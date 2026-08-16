# v75.2.0 — `TemporalRange` / `AsOfQuery` 型

Date: 2026-08-14
Status: 計画中

ロードマップ: [roadmap-v75.1-v76.0.md](../../roadmap/roadmap-v75.1-v76.0.md)

---

## Background

v75.1.0 で鮮度チェック（`FreshnessPolicy`）の基盤を構築した。
本バージョンでは時点クエリと期間フィルターを型安全に表現する `TemporalRange` / `AsOfQuery` を追加する。

Snowflake / Delta Lake の `AS OF` 構文はタイムトラベルクエリとして広く使われるが、
タイムスタンプの渡し間違いや範囲外の指定はランタイムエラーになりやすい。
`AsOfQuery` 型でタイムスタンプを明示的に持ち、`format_as_of_query` で SQL を自動生成することで、
クエリ構築ミスをコード上で検出できるようにする。

---

## Goals

1. `TemporalRange` 構造体を追加（from_ts / to_ts の期間を表す）
2. `AsOfQuery` 構造体を追加（table + as_of_ts のタイムトラベルクエリ）
3. `format_as_of_query(q: &AsOfQuery) -> String` — Snowflake 形式の SQL を生成
4. `is_in_range(ts: i64, range: &TemporalRange) -> bool` — タイムスタンプが期間内か判定
5. Rust テスト 2 件を driver.rs に追加（3694 → 3696）

---

## 型・API 仕様

### `TemporalRange` 構造体

```rust
pub struct TemporalRange {
    pub from_ts: i64,  // 期間開始タイムスタンプ（UNIX 秒、含む）
    pub to_ts: i64,    // 期間終了タイムスタンプ（UNIX 秒、含む）
}
```

### `AsOfQuery` 構造体

```rust
pub struct AsOfQuery {
    pub table: String,   // クエリ対象テーブル名
    pub as_of_ts: i64,   // タイムトラベル先タイムスタンプ（UNIX 秒）
}
```

### `format_as_of_query` 関数

```rust
pub fn format_as_of_query(q: &AsOfQuery) -> String
```

Snowflake の `AS OF TIMESTAMP` 構文を生成する。

出力例（`as_of_ts = 1735689600`、表 = `orders`）:
```sql
SELECT * FROM orders AS OF TIMESTAMP '2026-01-01 00:00:00'
```

タイムスタンプは UTC の `YYYY-MM-DD HH:MM:SS` 形式でフォーマットする。
`as_of_ts` が負値（1970-01-01 00:00:00 以前）の場合は 0 にクランプして `1970-01-01 00:00:00` として扱う。

### `is_in_range` 関数

```rust
pub fn is_in_range(ts: i64, range: &TemporalRange) -> bool
```

- `range.from_ts <= ts <= range.to_ts` であれば `true`
- 両端を含む閉区間

### Favnir コード例

```favnir
// タイムトラベルクエリ（Snowflake AS OF）
bind snapshot <- AsOfQuery { table: "orders", as_of_ts: run_date }

// 期間フィルター
bind range    <- TemporalRange { from_ts: start, to_ts: end }
bind filtered <- orders |> filter_in_range(range)
```

---

## Success Criteria

- `TemporalRange` / `AsOfQuery` が Rust にコンパイルされる
- `is_in_range` が閉区間で正しく判定する（両端を含む）
- `format_as_of_query` が Snowflake `AS OF TIMESTAMP 'YYYY-MM-DD HH:MM:SS'` 形式を出力する
- 以下の Rust テスト 2 件が pass する:
  - `temporal_range_filters_correctly` — `is_in_range` の境界値を含む判定を検証
  - `as_of_query_generates_sql` — `format_as_of_query` の SQL 出力形式を検証
- テスト総数: 3696（3694 + 2）

---

## Error Codes

新規エラーコード追加なし。

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `TemporalRange` / `AsOfQuery` 構造体 / `format_as_of_query` / `is_in_range` 関数を追加。`format_as_of_query` の内部ヘルパー（`unix_secs_to_utc` / `days_to_ymd` / `is_leap`）も同ファイルに追加。`v752000_tests` モジュールを追加 |
| `fav/Cargo.toml` | version `"75.1.0"` → `"75.2.0"` |
| `CHANGELOG.md` | v75.2.0 エントリを追加 |
| `versions/current.md` | 進行中バージョンを v75.2.0 に更新 |
