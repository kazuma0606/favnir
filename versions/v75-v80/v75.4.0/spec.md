# v75.4.0 仕様書 — Temporal join（時点結合）

Date: 2026-08-14
Status: 計画中

---

## Background

データウェアハウスでは「注文時点の商品価格」のような **Point-in-Time Join**（時点結合）が頻出する。
通常の JOIN では最新価格が使われてしまうが、時点結合では「その注文が発生した時点で有効だった価格」を
正確に取り出す必要がある。

v75.3.0 で SCD Type 2 の行管理（`ScdRow`）を実装した。v75.4.0 ではその SCD 履歴テーブルに対して
時点結合を行う SQL フラグメントを型安全に生成する基盤を追加する。

---

## Goals

1. `TemporalJoinConfig` 構造体（left_key, right_key, as_of_field: String）を追加する
2. `format_temporal_join_sql` 関数（Snowflake 向け時点結合 SQL フラグメント生成）を追加する
3. `validate_temporal_join_config` 関数（フィールド名検証）を追加する
4. Rust テスト 2 件を追加し 3700 tests に到達する

---

## 型・関数仕様

### `TemporalJoinConfig` 構造体

```rust
#[derive(Debug, Clone)]
pub struct TemporalJoinConfig {
    pub left_key:    String,   // 左テーブルの結合キー列名
    pub right_key:   String,   // 右テーブルの結合キー列名
    pub as_of_field: String,   // 左テーブルの「時点」を示す列名
}
```

---

### `validate_temporal_join_config`

```rust
pub fn validate_temporal_join_config(config: &TemporalJoinConfig) -> Result<(), String>
```

**検証ルール:**
- `left_key`, `right_key`, `as_of_field` のいずれかが空文字列の場合 `Err` を返す
- いずれかのフィールド名が英数字・アンダースコア以外の文字を含む場合も `Err` を返す
  （SQL インジェクション対策として `validate_table_name` と同様の方針を踏襲）
- すべてのフィールド名が空でなく英数字・アンダースコアのみで構成される場合 `Ok(())` を返す

**責任境界:**
- フィールド名（列名）の検証: `validate_temporal_join_config` の責任
- テーブル名の検証: 呼び出し側の責任（`format_as_of_query` と同方針）

---

### `format_temporal_join_sql`

```rust
pub fn format_temporal_join_sql(
    left_table: &str,
    right_table: &str,
    config: &TemporalJoinConfig,
) -> String
```

生成する SQL フラグメント（Snowflake SCD Type 2 向け）:

```sql
JOIN prices ON orders.product_id = prices.product_id
  AND prices.valid_from <= orders.order_date
  AND (prices.valid_to IS NULL OR prices.valid_to > orders.order_date)
```

**設計注意点:**
- `valid_from <= as_of_field` かつ `valid_to IS NULL OR valid_to > as_of_field` で
  「as_of_field 時点で有効なレコード」を正確に特定する（SCD Type 2 の標準パターン）
- `format_as_of_query`（v75.2.0）と同様に、テーブル名検証は呼び出し側の責任とする
  （この関数自体は SQL フラグメント生成のみを担う）

---

## Favnir コード例

```favnir
// 注文日時点の商品価格で結合
bind result <- orders |> temporal_join(
    prices,
    key: "product_id",
    as_of_field: "order_date"
)
// → 各注文の order_date 時点で有効な price レコードと結合
```

---

## Success Criteria

- `TemporalJoinConfig` 構造体が定義されている
- `validate_temporal_join_config` が空フィールドを正しく拒否する
- `format_temporal_join_sql` が正しい Snowflake SCD Type 2 JOIN フラグメントを生成する
- `cargo test` が 3700 tests all pass

---

## テスト仕様

### `temporal_join_sql_generated`

- `left_key="product_id"`, `right_key="product_id"`, `as_of_field="order_date"` で設定
- `format_temporal_join_sql("orders", "prices", &config)` の結果が以下を含む:
  - `"JOIN prices ON orders.product_id = prices.product_id"`
  - `"prices.valid_from <= orders.order_date"`
  - `"prices.valid_to IS NULL OR prices.valid_to > orders.order_date"`

### `temporal_join_invalid_config_rejected`

- `left_key=""` → `validate_temporal_join_config` が `Err` を返す
- `right_key=""` → `Err` を返す
- `as_of_field=""` → `Err` を返す
- 全フィールドが正常値 → `Ok(())` を返す

---

## 変更ファイル

- `fav/src/driver.rs` — `TemporalJoinConfig`, `validate_temporal_join_config`, `format_temporal_join_sql`, `v754000_tests` を追加
- `CHANGELOG.md` — v75.4.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.3.0` → `75.4.0` に更新

---

## 対象外

- Favnir 言語レベルでの `temporal_join` 組み込み演算子（将来バージョン予定）
- SCD Type 1 の時点結合（履歴がないため不要）
- 左テーブル・右テーブル名の SQL インジェクション検証（呼び出し側の責任）
