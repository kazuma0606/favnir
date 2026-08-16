# v77.3.0 実装計画 — 集約系不変条件

Date: 2026-08-15

---

## Step 1: driver.rs — AggregateProperty / AggregateInvariant 追加

`fav/src/driver.rs` の末尾に `// --- v77.3.0: 集約系不変条件 ---` コメントと型定義を追加する。

```rust
// --- v77.3.0: 集約系不変条件 ---

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateProperty {
    NonNegative,
    NonPositive,
    Bounded { min: f64, max: f64 },
    NonNull,
}

#[derive(Debug, Clone)]
pub struct AggregateInvariant {
    pub column:   String,
    pub property: AggregateProperty,
}
```

---

## Step 2: driver.rs — check_aggregate_invariant 追加

```rust
pub fn check_aggregate_invariant(
    values: &[f64],
    inv: &AggregateInvariant,
) -> Result<(), InvariantViolation> {
    match &inv.property {
        AggregateProperty::NonNegative => {
            if let Some(&v) = values.iter().find(|&&v| v < 0.0) {
                Err(InvariantViolation {
                    invariant_name: inv.column.clone(),
                    expected:       "NonNegative (>= 0.0)".to_string(),
                    actual:         format!("{:.4}", v),
                })
            } else {
                Ok(())
            }
        }
        AggregateProperty::NonPositive => {
            if let Some(&v) = values.iter().find(|&&v| v > 0.0) {
                Err(InvariantViolation {
                    invariant_name: inv.column.clone(),
                    expected:       "NonPositive (<= 0.0)".to_string(),
                    actual:         format!("{:.4}", v),
                })
            } else {
                Ok(())
            }
        }
        AggregateProperty::Bounded { min, max } => {
            if let Some(&v) = values.iter().find(|&&v| v < *min || v > *max) {
                Err(InvariantViolation {
                    invariant_name: inv.column.clone(),
                    expected:       format!("[{:.4}, {:.4}]", min, max),
                    actual:         format!("{:.4}", v),
                })
            } else {
                Ok(())
            }
        }
        AggregateProperty::NonNull => {
            if values.is_empty() {
                Err(InvariantViolation {
                    invariant_name: inv.column.clone(),
                    expected:       "NonNull (non-empty)".to_string(),
                    actual:         "empty".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }
}
```

---

## Step 3: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3740 テストが引き続き pass することを確認する（v773000_tests 追加前の状態）。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.3.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: driver.rs — v773000_tests モジュール追加

```rust
#[cfg(test)]
mod v773000_tests {
    use super::*;  // AggregateProperty, AggregateInvariant, check_aggregate_invariant, InvariantViolation を参照するため必須

    #[test]
    fn aggregate_invariant_non_negative_passes() { ... }

    #[test]
    fn aggregate_invariant_bounded_violated() { ... }
}
```

---

## Step 6: Cargo.toml バージョン更新

`77.2.0` → `77.3.0`

また、driver.rs 内に存在する `77.2.0` バージョン文字列アサーションを `77.3.0` へ一括置換する（`replace_all: true`）。

---

## Step 7: versions/current.md 更新

進行中バージョンを v77.3.0 に、次に切る版を v77.4.0 に更新する。

---

## Step 8: 最終確認

`cargo test` が 3742 tests all pass であることを確認する。
