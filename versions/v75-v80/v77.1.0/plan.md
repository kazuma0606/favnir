# v77.1.0 実装計画 — `PipelineInvariant` 型基盤

Date: 2026-08-15

---

## Step 1: driver.rs — InvariantCheckPoint / PipelineInvariant / InvariantViolation 追加

`fav/src/driver.rs` の末尾に `// --- v77.1.0: PipelineInvariant 型基盤 ---` コメントと型定義を追加する。

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum InvariantCheckPoint {
    Pre,
    Post,
    Both,
}

#[derive(Debug, Clone)]
pub struct PipelineInvariant {
    pub name:        String,
    pub expression:  String,
    pub check_point: InvariantCheckPoint,
}

#[derive(Debug, Clone)]
pub struct InvariantViolation {
    pub invariant_name: String,
    pub expected:       String,
    pub actual:         String,
}
```

---

## Step 2: driver.rs — check_count_invariant 追加

```rust
pub fn check_count_invariant(
    expected_max: usize,
    actual: usize,
    name: &str,
) -> Result<(), InvariantViolation> {
    if actual <= expected_max {
        Ok(())
    } else {
        Err(InvariantViolation {
            invariant_name: name.to_string(),
            expected:       format!("<= {}", expected_max),
            actual:         actual.to_string(),
        })
    }
}
```

---

## Step 3: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3736 テストが引き続き pass することを確認する（v771000_tests 追加前の状態）。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.1.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: driver.rs — v771000_tests モジュール追加

```rust
#[cfg(test)]
mod v771000_tests {
    use super::*;  // InvariantCheckPoint, PipelineInvariant, InvariantViolation, check_count_invariant を参照するため必須

    #[test]
    fn invariant_count_passes() { ... }

    #[test]
    fn invariant_count_violated() { ... }
}
```

---

## Step 6: Cargo.toml バージョン更新

`77.0.0` → `77.1.0`

また、driver.rs 内に存在する `77.0.0` バージョン文字列アサーションを `77.1.0` へ一括置換する（`replace_all: true`）。

---

## Step 7: versions/current.md 更新

進行中バージョンを v77.1.0 に、次に切る版を v77.2.0 に更新する。

---

## Step 8: 最終確認

`cargo test` が 3738 tests all pass であることを確認する。
