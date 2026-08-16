# v77.4.0 実装計画 — Join 系不変条件

Date: 2026-08-15

---

## Step 1: driver.rs — JoinType / JoinNullPolicy / JoinInvariant 追加

`fav/src/driver.rs` の末尾に `// --- v77.4.0: Join 系不変条件 ---` コメントと型定義を追加する。

```rust
// --- v77.4.0: Join 系不変条件 ---

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinNullPolicy {
    Fail,
    Warn,
    Allow,
}

#[derive(Debug, Clone)]
pub struct JoinInvariant {
    pub join_type:   JoinType,
    pub null_policy: JoinNullPolicy,
}
```

---

## Step 2: driver.rs — check_join_invariant 追加

```rust
pub fn check_join_invariant(
    left_count: usize,
    result_count: usize,
    null_count: usize,
    inv: &JoinInvariant,
) -> Result<(), InvariantViolation> {
    // Step 1: JoinType による行数チェック
    match inv.join_type {
        JoinType::Left | JoinType::Full => {
            if result_count < left_count {
                return Err(InvariantViolation {
                    invariant_name: "join_row_count".to_string(),
                    expected:       format!(">= {} (left_count)", left_count),
                    actual:         result_count.to_string(),
                });
            }
        }
        JoinType::Inner | JoinType::Right => {}
    }
    // Step 2: NullPolicy による NULL チェック
    match inv.null_policy {
        JoinNullPolicy::Fail => {
            if null_count > 0 {
                return Err(InvariantViolation {
                    invariant_name: "join_null_count".to_string(),
                    expected:       "0 nulls (Fail policy)".to_string(),
                    actual:         null_count.to_string(),
                });
            }
        }
        JoinNullPolicy::Warn | JoinNullPolicy::Allow => {}
    }
    Ok(())
}
```

---

## Step 3: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3742 テストが引き続き pass することを確認する（v774000_tests 追加前の状態）。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.4.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: driver.rs — v774000_tests モジュール追加

```rust
#[cfg(test)]
mod v774000_tests {
    use super::*;  // JoinType, JoinNullPolicy, JoinInvariant, check_join_invariant, InvariantViolation を参照するため必須

    #[test]
    fn join_invariant_inner_no_nulls() { ... }

    #[test]
    fn join_invariant_left_preserves_rows() { ... }
}
```

---

## Step 6: Cargo.toml バージョン更新

`77.3.0` → `77.4.0`

また、driver.rs 内に存在する `77.3.0` バージョン文字列アサーションを `77.4.0` へ一括置換する（`replace_all: true`）。

---

## Step 7: versions/current.md 更新

進行中バージョンを v77.4.0 に、次に切る版を v77.5.0 に更新する。

---

## Step 8: 最終確認

`cargo test` が 3744 tests all pass であることを確認する。
