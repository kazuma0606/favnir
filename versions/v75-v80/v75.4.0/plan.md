# v75.4.0 実装計画 — Temporal join（時点結合）

Date: 2026-08-14
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs — 型定義追加

`fav/src/driver.rs` の末尾（v75.3.0 ブロックの後）に以下を追加する。

```rust
// --- v75.4.0: Temporal join（時点結合） ---

/// 時点結合の設定。
#[derive(Debug, Clone)]
pub struct TemporalJoinConfig {
    pub left_key:    String,
    pub right_key:   String,
    pub as_of_field: String,
}
```

### Step 2: driver.rs — `validate_temporal_join_config` 関数追加

```rust
/// TemporalJoinConfig のフィールド名を検証する。
/// 空文字列・英数字アンダースコア以外の文字は SQL クエリとして無効なため拒否する。
pub fn validate_temporal_join_config(config: &TemporalJoinConfig) -> Result<(), String> {
    fn check_field(value: &str, name: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err(format!("{name} must not be empty"));
        }
        if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(format!("{name} contains invalid characters"));
        }
        Ok(())
    }
    check_field(&config.left_key,    "left_key")?;
    check_field(&config.right_key,   "right_key")?;
    check_field(&config.as_of_field, "as_of_field")?;
    Ok(())
}
```

### Step 3: driver.rs — `format_temporal_join_sql` 関数追加

```rust
/// 時点結合の SQL フラグメントを生成する（Snowflake SCD Type 2 向け）。
///
/// 例:
/// ```sql
/// JOIN prices ON orders.product_id = prices.product_id
///   AND prices.valid_from <= orders.order_date
///   AND (prices.valid_to IS NULL OR prices.valid_to > orders.order_date)
/// ```
pub fn format_temporal_join_sql(
    left_table: &str,
    right_table: &str,
    config: &TemporalJoinConfig,
) -> String {
    format!(
        "JOIN {right} ON {left}.{lk} = {right}.{rk}\n  AND {right}.valid_from <= {left}.{asof}\n  AND ({right}.valid_to IS NULL OR {right}.valid_to > {left}.{asof})",
        left  = left_table,
        right = right_table,
        lk    = config.left_key,
        rk    = config.right_key,
        asof  = config.as_of_field,
    )
}
```

**設計注意点:**
- `valid_from <= as_of_field` と `valid_to IS NULL OR valid_to > as_of_field` の組み合わせで
  SCD Type 2 の「時点有効レコード」を正確に特定する
- テーブル名の SQL インジェクション検証は呼び出し側の責任（`format_as_of_query` と同方針）

### Step 4: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v75.4.0 エントリを追加する。

### Step 5: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v754000_tests {
    use super::*;

    #[test]
    fn temporal_join_sql_generated() {
        let config = TemporalJoinConfig {
            left_key:    "product_id".to_string(),
            right_key:   "product_id".to_string(),
            as_of_field: "order_date".to_string(),
        };
        let sql = format_temporal_join_sql("orders", "prices", &config);
        assert!(sql.contains("JOIN prices ON orders.product_id = prices.product_id"),
            "join condition must match");
        assert!(sql.contains("prices.valid_from <= orders.order_date"),
            "valid_from condition required");
        assert!(sql.contains("prices.valid_to IS NULL OR prices.valid_to > orders.order_date"),
            "valid_to condition required");
    }

    #[test]
    fn temporal_join_invalid_config_rejected() {
        // 各フィールドが空文字列の場合は Err
        let bad_left = TemporalJoinConfig {
            left_key: "".to_string(), right_key: "id".to_string(), as_of_field: "ts".to_string(),
        };
        assert!(validate_temporal_join_config(&bad_left).is_err(),
            "empty left_key must be rejected");

        let bad_right = TemporalJoinConfig {
            left_key: "id".to_string(), right_key: "".to_string(), as_of_field: "ts".to_string(),
        };
        assert!(validate_temporal_join_config(&bad_right).is_err(),
            "empty right_key must be rejected");

        let bad_asof = TemporalJoinConfig {
            left_key: "id".to_string(), right_key: "id".to_string(), as_of_field: "".to_string(),
        };
        assert!(validate_temporal_join_config(&bad_asof).is_err(),
            "empty as_of_field must be rejected");

        // 全フィールド正常 → Ok
        let good = TemporalJoinConfig {
            left_key: "product_id".to_string(),
            right_key: "product_id".to_string(),
            as_of_field: "order_date".to_string(),
        };
        assert!(validate_temporal_join_config(&good).is_ok(), "valid config must pass");
    }
}
```

### Step 6: Cargo.toml・driver.rs バージョン更新

- `Cargo.toml`: `"75.3.0"` → `"75.4.0"`
- `driver.rs` 内の `version = \"75.3.0\"` を `replace_all` で `version = \"75.4.0\"` に更新

### Step 7: versions/current.md 更新

- 「進行中バージョン」を v75.4.0 に更新
- 「次に切る版」を v75.5.0 に更新

### Step 8: 最終確認

- `cargo test` 全件 pass（3700 tests）
- `cargo test v754000` 2 件 pass

---

## 依存関係

```
Step 1 (TemporalJoinConfig 型定義)
  └→ Step 2 (validate_temporal_join_config)
  └→ Step 3 (format_temporal_join_sql)
       └→ Step 5 (テスト)
Step 4 (CHANGELOG) — Step 5 より先に実施
Step 6 (バージョン更新) — Step 5 完了後
Step 7 (current.md) — Step 6 完了後
Step 8 (最終確認) — Step 6, 7 完了後
```

---

## リスク

- `format_temporal_join_sql` のフォーマット文字列中でテーブル名がそのまま埋め込まれるため、
  テーブル名の SQL インジェクション対策が呼び出し側に委ねられる点を doc コメントに明記すること
- `valid_from <= as_of` と `valid_to > as_of` の境界値ロジックが SCD Type 2 の標準定義と
  一致しているか（open-ended = `valid_to IS NULL`、閉端 = `valid_to` が指定された閉区間）
