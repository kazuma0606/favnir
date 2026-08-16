# v77.2.0 実装計画 — フィルター系不変条件

Date: 2026-08-15

---

## Step 1: driver.rs — FilterInvariant 構造体追加

`fav/src/driver.rs` の末尾に `// --- v77.2.0: フィルター系不変条件 ---` コメントと型定義を追加する。

```rust
// --- v77.2.0: フィルター系不変条件 ---

#[derive(Debug, Clone)]
pub struct FilterInvariant {
    pub expected_ratio_min: f64,
    pub expected_ratio_max: f64,
}
```

---

## Step 2: driver.rs — check_filter_invariant 追加

```rust
pub fn check_filter_invariant(
    input_count: usize,
    output_count: usize,
    inv: &FilterInvariant,
) -> Result<(), InvariantViolation> {
    if input_count == 0 {
        return Ok(());
    }
    let ratio = output_count as f64 / input_count as f64;
    if ratio >= inv.expected_ratio_min && ratio <= inv.expected_ratio_max {
        Ok(())
    } else {
        Err(InvariantViolation {
            invariant_name: "filter_ratio".to_string(),
            expected:       format!("[{:.4}, {:.4}]", inv.expected_ratio_min, inv.expected_ratio_max),
            actual:         format!("{:.4}", ratio),
        })
    }
}
```

---

## Step 3: driver.rs — format_filter_invariant_report 追加

```rust
pub fn format_filter_invariant_report(
    inv: &FilterInvariant,
    result: &Result<(), InvariantViolation>,
) -> String {
    match result {
        Ok(()) => format!(
            "filter_ratio OK: ratio in [{:.4}, {:.4}]",
            inv.expected_ratio_min, inv.expected_ratio_max
        ),
        Err(v) => format!(
            "filter_ratio VIOLATED: expected {}, actual {}",
            v.expected, v.actual
        ),
    }
}
```

---

## Step 4: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3738 テストが引き続き pass することを確認する（v772000_tests 追加前の状態）。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.2.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 6: driver.rs — v772000_tests モジュール追加

```rust
#[cfg(test)]
mod v772000_tests {
    use super::*;

    #[test]
    fn filter_invariant_ratio_valid() { ... }

    #[test]
    fn filter_invariant_ratio_violated() { ... }
}
```

---

## Step 7: Cargo.toml バージョン更新

`77.1.0` → `77.2.0`

また、driver.rs 内に存在する `77.1.0` バージョン文字列アサーションを `77.2.0` へ一括置換する（`replace_all: true`）。

---

## Step 8: versions/current.md 更新

進行中バージョンを v77.2.0 に、次に切る版を v77.3.0 に更新する。

---

## Step 9: 最終確認

`cargo test` が 3740 tests all pass であることを確認する。
