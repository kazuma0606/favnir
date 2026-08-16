# v78.7.0 実装計画 — Stream / Batch 統合実行モード

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `78.6.0` であることを確認
- `cargo test` が全 pass（3775 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

### Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾（`// --- v78.6.0: ...` ブロックの後）に以下を追加:

```rust
// --- v78.7.0: Stream / Batch 統合実行モード ---

/// 実行モードの選択肢。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionMode { Batch, Streaming, Adaptive }

/// モード選択の閾値設定。HashMap キーとして使用しないため Hash は付与しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionModeSelector {
    pub row_threshold:     u64,
    pub latency_target_ms: u64,
}
```

実装する関数:

**`select_execution_mode`**
- 引数: `est_rows: u64, latency_target_ms_req: u64, selector: &ExecutionModeSelector`
- 優先順位:
  1. `latency_target_ms_req < selector.latency_target_ms` → `Streaming`
  2. `est_rows > selector.row_threshold` → `Batch`
  3. それ以外 → `Adaptive`

---

### Step 3: CHANGELOG.md 更新（テスト追加より先）

```markdown
## [v78.7.0] — 2026-08-16 — Stream / Batch 統合実行モード

### Added
- `ExecutionMode` enum（Batch / Streaming / Adaptive、Debug / Clone / PartialEq / Eq / Hash 付き）: 実行モードの選択肢型
- `ExecutionModeSelector` 構造体（row_threshold: u64, latency_target_ms: u64、Debug / Clone / PartialEq / Eq 付き）: モード選択閾値設定型
- `select_execution_mode(est_rows: u64, latency_target_ms_req: u64, selector: &ExecutionModeSelector) -> ExecutionMode`: データ量・レイテンシ要件から最適モードを選択（latency 判定 → row 判定 → Adaptive の優先順位）

### Tests
- `mode_batch_for_large_data`: est_rows が row_threshold を超え latency 要件が緩い場合に Batch を返すことを検証
- `mode_streaming_for_low_latency`: latency 要件が selector 閾値より厳しい場合に Streaming を返すことを検証
```

---

### Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v787000_tests {
    use super::*;

    fn make_selector(row_threshold: u64, latency_ms: u64) -> ExecutionModeSelector {
        ExecutionModeSelector { row_threshold, latency_target_ms: latency_ms }
    }

    #[test]
    fn mode_batch_for_large_data() {
        // est_rows=10_000 > row_threshold=5_000、latency 要件は緩い（1000 >= 500）→ Batch
        let selector = make_selector(5_000, 500);
        let mode = select_execution_mode(10_000, 1_000, &selector);
        assert_eq!(mode, ExecutionMode::Batch, "large data should select Batch");
        // latency が同値（boundary）でも Batch を選択することを確認
        let mode_boundary = select_execution_mode(10_000, 500, &selector);
        assert_eq!(mode_boundary, ExecutionMode::Batch, "boundary latency should select Batch");
    }

    #[test]
    fn mode_streaming_for_low_latency() {
        // latency_req=50 < selector.latency_target_ms=500 → Streaming（data量に関わらず）
        let selector = make_selector(5_000, 500);
        let mode = select_execution_mode(100, 50, &selector);
        assert_eq!(mode, ExecutionMode::Streaming, "low latency should select Streaming");
        // 大量データでも latency 優先で Streaming を選択することを確認
        let mode_large = select_execution_mode(100_000, 50, &selector);
        assert_eq!(mode_large, ExecutionMode::Streaming, "latency takes priority over row count");
    }
}
```

---

### Step 5: Cargo.toml バージョン更新

- `version` を `"78.6.0"` → `"78.7.0"` に変更
- driver.rs 内の `78.6.0` バージョン文字列アサーションを `78.7.0` に一括更新（`replace_all: true`）
- `grep -c "78.6.0" fav/src/driver.rs` で **出力が 1** であることを確認（残るのは `// --- v78.6.0: !Parallel エフェクト統合 ---` の 1 件のみ）

---

### Step 6: versions/current.md 更新

- 進行中: `v78.7.0`（Stream / Batch 統合実行モード）
- 次: `v78.8.0`（実行計画キャッシュ）

---

### Step 7: 最終確認

- `cargo test v787000` → 2 tests pass
- `cargo test` → 全 pass（3777 tests）
- Cargo.toml version = `78.7.0`
- CHANGELOG 先頭 = `[v78.7.0]`
