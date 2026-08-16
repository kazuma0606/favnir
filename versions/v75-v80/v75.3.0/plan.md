# v75.3.0 実装計画 — SCD Type 1 / Type 2 ネイティブ型

Date: 2026-08-14
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs — 型定義追加

`fav/src/driver.rs` の末尾（v75.2.0 ブロックの後）に以下を追加する。

```rust
// --- v75.3.0: SCD Type 1 / Type 2 ネイティブ型 ---

#[derive(Debug, Clone, PartialEq)]
pub enum ScdType { Type1, Type2 }

#[derive(Debug, Clone)]
pub struct ScdRow {
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub is_current: bool,
    pub data: String,
}
```

### Step 2: driver.rs — `apply_scd2_update` 関数追加

```rust
pub fn apply_scd2_update(existing: &[ScdRow], new_data: &str, new_ts: i64) -> Vec<ScdRow> {
    let mut result: Vec<ScdRow> = Vec::new();
    let mut changed = false;
    for row in existing {
        if row.is_current && row.data != new_data {
            result.push(ScdRow {
                valid_from: row.valid_from,
                valid_to: Some(new_ts - 1),
                is_current: false,
                data: row.data.clone(),
            });
            changed = true;
        } else {
            result.push(row.clone());
        }
    }
    if changed || existing.is_empty() {
        result.push(ScdRow {
            valid_from: new_ts,
            valid_to: None,
            is_current: true,
            data: new_data.to_string(),
        });
    }
    result
}
```

**設計注意点:**
- `existing` が空の場合も新レコードを追加する（初回 upsert）
- data が同一の場合（no-op）は新レコードを追加しない
- `is_current=true` かつ `data != new_data` のレコードが複数ある場合はすべて閉じる（不整合データへの耐性）

### Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v75.3.0 エントリを追加する。

### Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v753000_tests {
    use super::*;

    #[test]
    fn scd2_creates_history_row() {
        // existing に 1 件（is_current=true, data="旧"）
        let existing = vec![ScdRow {
            valid_from: 1_000, valid_to: None, is_current: true,
            data: r#"{"city":"Tokyo"}"#.to_string(),
        }];
        let result = apply_scd2_update(&existing, r#"{"city":"Osaka"}"#, 2_000);
        assert_eq!(result.len(), 2);
        let new_row = result.iter().find(|r| r.is_current).unwrap();
        assert_eq!(new_row.data, r#"{"city":"Osaka"}"#);
        assert_eq!(new_row.valid_from, 2_000);
        assert!(new_row.valid_to.is_none());
    }

    #[test]
    fn scd2_marks_previous_expired() {
        let existing = vec![ScdRow {
            valid_from: 500, valid_to: None, is_current: true,
            data: r#"{"status":"active"}"#.to_string(),
        }];
        let result = apply_scd2_update(&existing, r#"{"status":"inactive"}"#, 1_500);
        // 旧レコードが valid_to = 1499 で閉じられていること
        let old_row = result.iter().find(|r| !r.is_current).unwrap();
        assert_eq!(old_row.valid_to, Some(1_499));
        assert_eq!(old_row.valid_from, 500);
        // no-op ケース：data が同一のため新レコードが追加されないこと
        let result2 = apply_scd2_update(&result, r#"{"status":"inactive"}"#, 2_000);
        assert_eq!(result2.len(), 2, "no-op must not add a new row");
    }
}
```

`cargo test v753000` で 2 件 PASS を確認する。

### Step 5: Cargo.toml・driver.rs バージョン更新

- `Cargo.toml`: `"75.2.0"` → `"75.3.0"`
- `driver.rs` 内の `version = \"75.2.0\"` を `replace_all` で `version = \"75.3.0\"` に更新

### Step 6: versions/current.md 更新

- 「進行中バージョン」を v75.3.0 に更新
- 「次に切る版」を v75.4.0 に更新

### Step 7: 最終確認

- `cargo test` 全件 pass（3698 tests）
- `cargo test v753000` 2 件 pass

---

## 依存関係

```
Step 1 (型定義)
  └→ Step 2 (apply_scd2_update)
       └→ Step 4 (テスト)
Step 3 (CHANGELOG) — Step 4 より先に実施
Step 5 (バージョン更新) — Step 4 完了後
Step 6 (current.md) — Step 5 完了後
Step 7 (最終確認) — Step 5, 6 完了後
```

---

## リスク

- `apply_scd2_update` の no-op 判定（data が同一の場合）のロジックが微妙：`is_current=false` の古いレコードを誤って変更しないよう注意
- 複数の `is_current=true` レコードが存在する不整合データ（通常は起きないが）への耐性を確認
