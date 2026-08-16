# v78.3.0 実装計画 — `!Adaptive` エフェクト基盤

---

## Step 1: 事前確認

- `fav/Cargo.toml` のバージョンが `78.2.0` であることを確認
- `cargo test` が全 pass（3766 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

## Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾（v78.2.0 テストモジュールの直後）に以下を追加する。

```rust
// --- v78.3.0: !Adaptive エフェクト基盤 ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionStrategy {
    BroadcastJoin,
    HashJoin,
    SortMergeJoin,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveConfig {
    pub broadcast_threshold_rows: u64,
    pub default_parallelism:      usize,
}

pub fn select_join_strategy(
    left_rows: u64,
    right_rows: u64,
    config: &AdaptiveConfig,
) -> ExecutionStrategy {
    let smaller = left_rows.min(right_rows);
    if smaller <= config.broadcast_threshold_rows {
        ExecutionStrategy::BroadcastJoin
    } else {
        ExecutionStrategy::HashJoin
    }
}

pub fn format_strategy_selected(strategy: &ExecutionStrategy) -> String {
    match strategy {
        ExecutionStrategy::BroadcastJoin  => "Strategy: BroadcastJoin (small table detected)".to_string(),
        ExecutionStrategy::HashJoin       => "Strategy: HashJoin (large table, hash partition)".to_string(),
        ExecutionStrategy::SortMergeJoin  => "Strategy: SortMergeJoin (sorted stream merge)".to_string(),
        ExecutionStrategy::Auto           => "Strategy: Auto (runtime selection)".to_string(),
    }
}
```

`cargo build` でコンパイルエラーがないことを確認する。

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭（`[v78.2.0]` エントリの前）に v78.3.0 エントリを追加する。

```markdown
## [v78.3.0] — 2026-08-16 — !Adaptive エフェクト基盤

### Added
- `ExecutionStrategy` enum（BroadcastJoin / HashJoin / SortMergeJoin / Auto、Debug / Clone / PartialEq / Eq / Hash 付き）: ...
- `AdaptiveConfig` 構造体（broadcast_threshold_rows: u64, default_parallelism: usize、Debug / Clone / PartialEq / Eq 付き）: ...
- `select_join_strategy(left_rows: u64, right_rows: u64, config: &AdaptiveConfig) -> ExecutionStrategy`: ...
- `format_strategy_selected(strategy: &ExecutionStrategy) -> String`: ...

### Tests
- `adaptive_selects_broadcast_for_small_table`: ...
- `adaptive_selects_hash_for_large_table`: ...
```

---

## Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v783000_tests {
    use super::*;

    #[test]
    fn adaptive_selects_broadcast_for_small_table() {
        // right_rows=500 <= broadcast_threshold_rows=1000 → BroadcastJoin
        let config = AdaptiveConfig {
            broadcast_threshold_rows: 1000,
            default_parallelism:      4,
        };
        let strategy = select_join_strategy(100_000, 500, &config);
        assert_eq!(strategy, ExecutionStrategy::BroadcastJoin);
        let label = format_strategy_selected(&strategy);
        assert!(label.contains("BroadcastJoin"));
    }

    #[test]
    fn adaptive_selects_hash_for_large_table() {
        // min(50_000, 80_000)=50_000 > broadcast_threshold_rows=1000 → HashJoin
        let config = AdaptiveConfig {
            broadcast_threshold_rows: 1000,
            default_parallelism:      4,
        };
        let strategy = select_join_strategy(50_000, 80_000, &config);
        assert_eq!(strategy, ExecutionStrategy::HashJoin);
        let label = format_strategy_selected(&strategy);
        assert!(label.contains("HashJoin"));
    }
}
```

`cargo test v783000` で 2 件 pass を確認する。

---

## Step 5: Cargo.toml バージョン更新

- `version` を `"78.2.0"` → `"78.3.0"` に変更
- driver.rs 内のバージョン文字列アサーションを `78.2.0` → `78.3.0` に一括更新（`replace_all: true`）
- **replace_all 後に** `grep "78.2.0" fav/src/driver.rs` を実行し、以下を確認する:
  - `// --- v78.2.0: キャッシュ戦略型 ---` セクションコメントが **1 件**残っていること
  - それ以外の `78.2.0` 文字列が 0 件であること（アサーションが書き換わっていないこと）
  - もし書き換わっていたら手動で `78.2.0` に戻す

---

## Step 6: versions/current.md 更新

- `## 進行中バージョン` 欄を `**v78.3.0**（!Adaptive エフェクト基盤）` に更新
- `## 次に切る版` 欄を `**v78.4.0**（コスト推定モデル）` に更新

---

## Step 7: 最終確認

- `cargo test` が全 pass（3768 tests）であることを確認
- `cargo test v783000` で 2 件 pass を確認
- `fav/Cargo.toml` のバージョンが `78.3.0` であることを確認
- `CHANGELOG.md` の先頭が `[v78.3.0]` であることを確認
