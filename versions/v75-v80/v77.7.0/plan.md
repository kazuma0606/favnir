# v77.7.0 実装計画 — 反例自動生成

Date: 2026-08-16

---

## Step 1: driver.rs — CounterExampleResult 追加

`fav/src/driver.rs` の末尾に `// --- v77.7.0: 反例自動生成 ---` コメントと型定義を追加する。

```rust
// --- v77.7.0: 反例自動生成 ---

/// 反例生成結果。`f64` フィールドを含むため `PartialEq` / `Eq` は derive しない。
#[derive(Debug, Clone)]
pub struct CounterExampleResult {
    pub invariant_name: String,
    pub example:        Vec<f64>,
    pub violates:       bool,
}
```

---

## Step 2: driver.rs — generate_counter_example_values 追加

```rust
pub fn generate_counter_example_values(inv: &AggregateInvariant, seed: u64) -> CounterExampleResult {
    // DESIGN: v77.7.0 では seed % 2 による 2 パターンの擬似ランダム化のみ実装する。
    // 偶数シード: adversarial 候補（負値を含む、NonNegative では違反を引き起こす）
    // 奇数シード: 安全な正値候補（NonNegative では違反しない）
    // 本格的な乱数生成（PRNG 等）は将来の v78.x 以降で実装する。
    let candidates: Vec<f64> = if seed % 2 == 0 {
        vec![0.0, -0.001, -1.0, 1.0]
    } else {
        vec![0.0, 0.001, 1.0, 100.0]
    };
    // NOTE: `inv` はすでに `&AggregateInvariant` 型なので `inv` をそのまま渡す（`&inv` は二重参照になるため誤り）
    let result = check_aggregate_invariant(&candidates, inv);
    let violates = result.is_err();
    CounterExampleResult {
        invariant_name: inv.column.clone(),
        example:        candidates,
        violates,
    }
}
```

---

## Step 3: cargo test（既存テスト通過確認）

`cargo test` を実行し、既存の 3748 テストが引き続き pass することを確認する（v777000_tests 追加前の状態）。

---

## Step 4: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v77.7.0 エントリを追加する（テストモジュール追加より先）。

---

## Step 5: driver.rs — v777000_tests モジュール追加

```rust
#[cfg(test)]
mod v777000_tests {
    use super::*;

    #[test]
    fn counter_example_finds_violation() { ... }

    #[test]
    fn counter_example_none_for_trivially_valid() { ... }
}
```

---

## Step 6: Cargo.toml バージョン更新

`77.6.0` → `77.7.0`

また、driver.rs 内の `77.6.0` バージョン文字列アサーションを `77.7.0` へ一括置換する（`replace_all: true`）。

> **注意**: `replace_all: true` はセクションコメント `// --- v77.6.0: 証明付き CI 統合 ---` 内の `77.6.0` にも反応し、`// --- v77.7.0: 証明付き CI 統合 ---` に書き換えてしまう。**replace_all 実行後に必ず** `grep "v77.6.0" fav/src/driver.rs` を実行し、`// --- v77.6.0: 証明付き CI 統合 ---` が残っていることを確認する。書き換わっていた場合は手動で `v77.6.0` に戻すこと。

---

## Step 7: versions/current.md 更新

進行中バージョンを v77.7.0 に、次に切る版を v77.8.0 に更新する。

---

## Step 8: 最終確認

`cargo test` が 3750 tests all pass であることを確認する。
