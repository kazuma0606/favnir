# v78.4.0 実装計画 — コスト推定モデル

---

## Step 1: 事前確認

- `fav/Cargo.toml` のバージョンが `78.3.0` であることを確認
- `cargo test` が全 pass（3768 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

## Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾（v78.3.0 テストモジュールの直後）に以下を追加する。

```rust
// --- v78.4.0: コスト推定モデル ---

// f64 フィールドを含むため Eq / Hash は付与しない
#[derive(Debug, Clone, PartialEq)]
pub struct CostEstimate {
    pub cpu_units: f64,
    pub memory_mb: f64,
    pub io_ops:    u64,
}

pub fn estimate_broadcast_cost(right_rows: u64) -> CostEstimate {
    CostEstimate {
        cpu_units: right_rows as f64 * 0.01,
        memory_mb: right_rows as f64 * 0.1,
        io_ops:    right_rows,
    }
}

pub fn estimate_hash_cost(left_rows: u64, right_rows: u64) -> CostEstimate {
    CostEstimate {
        cpu_units: 5.0 + (left_rows + right_rows) as f64 * 0.0001,
        memory_mb: (left_rows + right_rows) as f64 * 0.01,
        io_ops:    (left_rows + right_rows) / 2,
    }
}

pub fn select_min_cost_strategy(
    estimates: &[(ExecutionStrategy, CostEstimate)],
) -> ExecutionStrategy {
    estimates
        .iter()
        .min_by(|(_, a), (_, b)| a.cpu_units.partial_cmp(&b.cpu_units).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(s, _)| s.clone())
        .unwrap_or(ExecutionStrategy::Auto)
}
```

`cargo build` でコンパイルエラーがないことを確認する。

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭（`[v78.3.0]` エントリの前）に v78.4.0 エントリを追加する。

```markdown
## [v78.4.0] — 2026-08-16 — コスト推定モデル

### Added
- `CostEstimate` 構造体（cpu_units: f64, memory_mb: f64, io_ops: u64、Debug / Clone / PartialEq 付き）: ...
- `estimate_broadcast_cost(right_rows: u64) -> CostEstimate`: ...
- `estimate_hash_cost(left_rows: u64, right_rows: u64) -> CostEstimate`: ...
- `select_min_cost_strategy(estimates: &[(ExecutionStrategy, CostEstimate)]) -> ExecutionStrategy`: ...

### Tests
- `cost_estimate_broadcast_cheaper_for_small`: ...
- `cost_estimate_hash_wins_for_large`: ...
```

---

## Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v784000_tests {
    use super::*;

    #[test]
    fn cost_estimate_broadcast_cheaper_for_small() {
        // right=100, left=10_000
        // broadcast cpu = 100 * 0.01 = 1.0
        // hash     cpu = 5.0 + 10_100 * 0.0001 = 6.01
        // → select_min_cost_strategy returns BroadcastJoin
        let b = estimate_broadcast_cost(100);
        let h = estimate_hash_cost(10_000, 100);
        let estimates = vec![
            (ExecutionStrategy::BroadcastJoin, b),
            (ExecutionStrategy::HashJoin, h),
        ];
        let winner = select_min_cost_strategy(&estimates);
        assert_eq!(winner, ExecutionStrategy::BroadcastJoin);
    }

    #[test]
    fn cost_estimate_hash_wins_for_large() {
        // right=50_000, left=10_000
        // broadcast cpu = 50_000 * 0.01 = 500.0
        // hash     cpu = 5.0 + 60_000 * 0.0001 = 11.0
        // → select_min_cost_strategy returns HashJoin
        let b = estimate_broadcast_cost(50_000);
        let h = estimate_hash_cost(10_000, 50_000);
        let estimates = vec![
            (ExecutionStrategy::BroadcastJoin, b),
            (ExecutionStrategy::HashJoin, h),
        ];
        let winner = select_min_cost_strategy(&estimates);
        assert_eq!(winner, ExecutionStrategy::HashJoin);
    }
}
```

`cargo test v784000` で 2 件 pass を確認する。

---

## Step 5: Cargo.toml バージョン更新

- `version` を `"78.3.0"` → `"78.4.0"` に変更
- driver.rs 内のバージョン文字列アサーションを `78.3.0` → `78.4.0` に一括更新（`replace_all: true`）
- **replace_all 後に** `grep -c "78.3.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.3.0: !Adaptive エフェクト基盤 ---` セクションコメントの 1 件のみ

---

## Step 6: versions/current.md 更新

- `## 進行中バージョン` 欄を `**v78.4.0**（コスト推定モデル）` に更新
- `## 次に切る版` 欄を `**v78.5.0**（fav explain plan 可視化）` に更新

---

## Step 7: 最終確認

- `cargo test` が全 pass（3770 tests）であることを確認
- `cargo test v784000` で 2 件 pass を確認
- `fav/Cargo.toml` のバージョンが `78.4.0` であることを確認
- `CHANGELOG.md` の先頭が `[v78.4.0]` であることを確認
