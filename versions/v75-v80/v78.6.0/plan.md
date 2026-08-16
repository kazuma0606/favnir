# v78.6.0 実装計画 — `!Parallel` エフェクト統合

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `78.5.0` であることを確認
- `cargo test` が全 pass（3772 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

### Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾に以下を追加（`// --- v78.5.0: ...` ブロックの後）:

```rust
// --- v78.6.0: !Parallel エフェクト統合 ---

/// 並列実行の設定。HashMap キーとして使用しないため Hash は付与しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelConfig {
    pub threads:         usize,
    pub partition_count: usize,
    pub partition_key:   String,
}

/// 1 パーティションの実行計画。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionPlan {
    pub partition_id:  usize,
    pub rows_estimate: u64,
    pub thread_id:     usize,
}
```

実装する関数:

**`plan_parallel_execution`**
- `partition_count == 0` → 空 Vec 返す
- 基本行数 = `total_rows / partition_count`
- 端数 = `total_rows % partition_count`（最後のパーティションに加算）
- `thread_id = partition_id % config.threads`（threads == 0 は 0）

**`format_parallel_plan`**
- 空スライス対応（"No partitions." を返す）
- ヘッダー: `Parallel Plan: {n} partitions / {threads} threads`（plans から計算）
- 各行: `  Partition {id}: ~{rows} rows  thread={thread_id}`
- フッター: `  Total rows: {total}`

---

### Step 3: CHANGELOG.md 更新（テスト追加より先）

```markdown
## [v78.6.0] — 2026-08-16 — !Parallel エフェクト統合

### Added
- `ParallelConfig` 構造体（threads: usize, partition_count: usize, partition_key: String、Debug / Clone / PartialEq / Eq 付き）
- `PartitionPlan` 構造体（partition_id: usize, rows_estimate: u64, thread_id: usize、Debug / Clone / PartialEq / Eq 付き）
- `plan_parallel_execution(total_rows: u64, config: &ParallelConfig) -> Vec<PartitionPlan>`: パーティション分割計画を生成
- `format_parallel_plan(plans: &[PartitionPlan]) -> String`: 並列計画をテキスト形式で可視化

### Tests
- `parallel_plan_creates_correct_partitions`: partition 数・thread 割り当て・format 出力を検証（端数 0 ケース）
- `parallel_plan_distributes_evenly`: 均等分散・合計行数一致・端数最終パーティション加算を検証
```

---

### Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v786000_tests {
    use super::*;

    fn make_config(threads: usize, partitions: usize) -> ParallelConfig {
        ParallelConfig {
            threads,
            partition_count: partitions,
            partition_key: "customer_id".to_string(),
        }
    }

    #[test]
    fn parallel_plan_creates_correct_partitions() {
        let config = make_config(4, 8);
        let plans = plan_parallel_execution(1000, &config);
        assert_eq!(plans.len(), 8, "partition count mismatch");
        // thread_id は partition_id % threads
        assert_eq!(plans[0].thread_id, 0);
        assert_eq!(plans[4].thread_id, 0);  // 4 % 4 == 0
        assert_eq!(plans[3].thread_id, 3);
        // format_parallel_plan がヘッダーを含むことを確認
        let output = format_parallel_plan(&plans);
        assert!(output.contains("Parallel Plan:"), "header missing");
        assert!(output.contains("Partition 0:"), "partition 0 missing");
        assert!(output.contains("Total rows:"), "total missing");
    }

    #[test]
    fn parallel_plan_distributes_evenly() {
        let config = make_config(2, 4);
        let plans = plan_parallel_execution(100, &config);
        assert_eq!(plans.len(), 4);
        // 100 / 4 = 25 rows each, no remainder
        let sum: u64 = plans.iter().map(|p| p.rows_estimate).sum();
        assert_eq!(sum, 100, "total rows mismatch");
        for p in &plans[..3] {
            assert_eq!(p.rows_estimate, 25, "uneven distribution");
        }
        // 端数確認: 101 rows / 4 partitions → base=25, remainder=1 → last=26
        let config2 = make_config(2, 4);
        let plans2 = plan_parallel_execution(101, &config2);
        let sum2: u64 = plans2.iter().map(|p| p.rows_estimate).sum();
        assert_eq!(sum2, 101, "remainder rows mismatch");
        assert_eq!(plans2[3].rows_estimate, 26, "last partition should absorb remainder");
    }
}
```

---

### Step 5: Cargo.toml バージョン更新

- `version` を `"78.5.0"` → `"78.6.0"` に変更
- driver.rs 内の `78.5.0` バージョン文字列アサーションを `78.6.0` に一括更新（`replace_all: true`）
- `grep -c "78.5.0" fav/src/driver.rs` で **出力が 1** であることを確認（残るのは `// --- v78.5.0: fav explain plan 可視化 ---` の 1 件のみ）

---

### Step 6: versions/current.md 更新

- 進行中: `v78.6.0`（!Parallel エフェクト統合）
- 次: `v78.7.0`（Stream / Batch 統合実行モード）

---

### Step 7: 最終確認

- `cargo test v786000` → 2 tests pass
- `cargo test` → 全 pass（3774 tests）
- Cargo.toml version = `78.6.0`
- CHANGELOG 先頭 = `[v78.6.0]`
