# v75.5.0 実装計画 — `RetentionPolicy` 型

Date: 2026-08-15
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs — `RetentionAction` enum 追加

`fav/src/driver.rs` の末尾（v75.4.0 ブロックの後）に追加する。

```rust
// --- v75.5.0: RetentionPolicy 型 ---

/// 保持期限超過時に適用するアクション。
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionAction {
    Delete,
    Archive,
    Anonymize,
}
```

### Step 2: driver.rs — `RetentionResult` enum 追加

```rust
/// `apply_retention_check` の判定結果。
/// `Keep` は保持期限内、その他は期限超過に対応するアクション。
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionResult {
    Keep,
    Delete,
    Archive,
    Anonymize,
}
```

### Step 3: driver.rs — `RetentionPolicy` 構造体追加

```rust
/// データ保持ポリシー。
///
/// # フィールド
/// - `max_age_days`: 保持最大日数（0 = 全行が即時対象）
/// - `action`: 期限超過時のアクション
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub max_age_days: u64,
    pub action:       RetentionAction,
}
```

### Step 4: driver.rs — `apply_retention_check` 関数追加

```rust
/// 行のタイムスタンプと現在時刻を保持ポリシーと照合する。
///
/// # 判定ロジック
/// `now - row_ts > max_age_days * 86400` のとき期限超過。
/// 境界値（ちょうど max_age_days 日）は Keep（開区間、v75.4.0 の valid_to と同方針）。
/// `now < row_ts`（未来タイムスタンプ）も Keep として扱う。
pub fn apply_retention_check(
    row_ts: i64,
    now:    i64,
    policy: &RetentionPolicy,
) -> RetentionResult {
    let age_secs = now - row_ts;
    let max_secs = policy.max_age_days as i64 * 86_400;
    if age_secs > max_secs {
        match policy.action {
            RetentionAction::Delete    => RetentionResult::Delete,
            RetentionAction::Archive   => RetentionResult::Archive,
            RetentionAction::Anonymize => RetentionResult::Anonymize,
        }
    } else {
        RetentionResult::Keep
    }
}
```

### Step 4.5: cargo check 確認

`cargo check` でコンパイルエラーがないことを確認する。

### Step 5: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v75.5.0 エントリを追加する。

### Step 6: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v755000_tests {
    use super::*;

    #[test]
    fn retention_delete_old_rows() {
        let policy = RetentionPolicy { max_age_days: 365, action: RetentionAction::Delete };
        // 366日後 → Delete
        assert_eq!(
            apply_retention_check(0, 366 * 86_400, &policy),
            RetentionResult::Delete,
            "row older than 365 days must be deleted"
        );
        // ちょうど365日 → Keep（boundary exclusive）
        assert_eq!(
            apply_retention_check(0, 365 * 86_400, &policy),
            RetentionResult::Keep,
            "row exactly at boundary must be kept"
        );
        // 100日後 → Keep
        assert_eq!(
            apply_retention_check(0, 100 * 86_400, &policy),
            RetentionResult::Keep,
            "row within retention must be kept"
        );
    }

    #[test]
    fn retention_anonymize_action() {
        let policy = RetentionPolicy { max_age_days: 90, action: RetentionAction::Anonymize };
        let base: i64 = 1_000_000;
        // 91日後 → Anonymize
        assert_eq!(
            apply_retention_check(base, base + 91 * 86_400, &policy),
            RetentionResult::Anonymize,
            "row older than 90 days must be anonymized"
        );
        // ちょうど90日 → Keep
        assert_eq!(
            apply_retention_check(base, base + 90 * 86_400, &policy),
            RetentionResult::Keep,
            "row exactly at boundary must be kept"
        );
        // 未来の now → Keep
        assert_eq!(
            apply_retention_check(base, base - 1, &policy),
            RetentionResult::Keep,
            "future row_ts (now < row_ts) must be kept"
        );
    }
}
```

### Step 7: Cargo.toml・driver.rs バージョン更新

- `Cargo.toml`: `"75.4.0"` → `"75.5.0"`
- `driver.rs` 内の `version = \"75.4.0\"` を `replace_all` で `version = \"75.5.0\"` に更新

### Step 8: versions/current.md 更新

- 「進行中バージョン」を v75.5.0 に更新
- 「次に切る版」を v75.6.0 に更新

### Step 9: 最終確認

- `cargo test` 全件 pass（3702 tests）
- `cargo test v755000` 2 件 pass

---

## 依存関係

```
Step 1 (RetentionAction)
  └→ Step 3 (RetentionPolicy — action フィールドの型)
Step 2 (RetentionResult)
  └→ Step 4 (apply_retention_check の戻り型)
Step 3 (RetentionPolicy)
  └→ Step 4 (apply_retention_check の引数型)
Step 4 (apply_retention_check)
  └→ Step 6 (テスト)
Step 5 (CHANGELOG) — Step 6 より先に実施
Step 7 (バージョン更新) — Step 6 完了後
Step 8 (current.md) — Step 7 完了後
Step 9 (最終確認) — Step 7, 8 完了後
```

---

## リスク

- `max_age_days: u64` を `i64` にキャストする際、`u64::MAX` 付近の値でオーバーフローが起きる可能性がある。
  実用上は数千日以下の値のみ使用されるため、`as i64` キャストは許容される。
  必要であれば `try_into` で `Err` を返す設計に変更すること。
- `Archive` アクションは結果として `RetentionResult::Archive` を返すのみで、実際のアーカイブ処理は呼び出し側の責任。
  doc コメントに明記すること。
