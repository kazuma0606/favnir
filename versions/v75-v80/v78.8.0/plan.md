# v78.8.0 実装計画 — 実行計画キャッシュ

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `78.7.0` であることを確認
- `cargo test` が全 pass（3778 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

### Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾（`// --- v78.7.0: ...` ブロックの後）に以下を追加:

```rust
// --- v78.8.0: 実行計画キャッシュ ---

/// キャッシュの 1 エントリ。`ExecutionPlan`（f64 含む）を内包するため Eq / Hash は付与しない。
#[derive(Debug, Clone, PartialEq)]
pub struct PlanCacheEntry {
    pub pipeline_hash: String,
    pub plan:          ExecutionPlan,
    pub created_at:    i64,
}

/// 実行計画キャッシュ。`Vec<PlanCacheEntry>` を内包するため Eq / Hash は付与しない。
#[derive(Debug, Clone, PartialEq)]
pub struct PlanCache {
    pub entries:  Vec<PlanCacheEntry>,
    pub max_size: usize,
}
```

実装する関数:

**`lookup_plan`**
```rust
pub fn lookup_plan<'a>(cache: &'a PlanCache, hash: &str) -> Option<&'a ExecutionPlan> {
    cache.entries.iter()
        .find(|e| e.pipeline_hash == hash)
        .map(|e| &e.plan)
}
```

**`insert_plan`**
- `max_size == 0` → 即リターン
- `hash` が既存 → `entry.plan` と `entry.created_at` をフィールド更新（早期リターン）
- 新規 + `len >= max_size` → `created_at` 最小のインデックスを見つけて `swap_remove` または `remove` → `push`
- 新規 + `len < max_size` → `push`

```rust
pub fn insert_plan(cache: &mut PlanCache, hash: &str, plan: ExecutionPlan) {
    if cache.max_size == 0 { return; }
    let created_at = plan.total_cost.io_ops as i64;  // 簡易タイムスタンプ代用
    // 既存エントリの上書き
    if let Some(entry) = cache.entries.iter_mut().find(|e| e.pipeline_hash == hash) {
        entry.plan = plan;
        entry.created_at = created_at;
        return;
    }
    // max_size 到達時: created_at 最小エントリをエビクション
    if cache.entries.len() >= cache.max_size {
        let oldest_idx = cache.entries.iter().enumerate()
            .min_by_key(|(_, e)| e.created_at)
            .map(|(i, _)| i)
            .unwrap_or(0);
        cache.entries.remove(oldest_idx);
    }
    cache.entries.push(PlanCacheEntry { pipeline_hash: hash.to_string(), plan, created_at });
}
```

---

### Step 3: CHANGELOG.md 更新（テスト追加より先）

```markdown
## [v78.8.0] — 2026-08-16 — 実行計画キャッシュ

### Added
- `PlanCacheEntry` 構造体（pipeline_hash: String, plan: ExecutionPlan, created_at: i64、Debug / Clone / PartialEq 付き、Eq/Hash なし）
- `PlanCache` 構造体（entries: Vec<PlanCacheEntry>, max_size: usize、Debug / Clone / PartialEq 付き、Eq/Hash なし）
- `lookup_plan<'a>(cache: &'a PlanCache, hash: &str) -> Option<&'a ExecutionPlan>`: ハッシュから実行計画を取得
- `insert_plan(cache: &mut PlanCache, hash: &str, plan: ExecutionPlan)`: キャッシュに挿入（上書き / エビクション付き）

### Tests
- `plan_cache_hit`: 挿入後に lookup が Some を返すことを検証
- `plan_cache_evicts_oldest_on_full`: max_size 到達後の挿入で最古エントリがエビクションされることを検証
```

---

### Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v788000_tests {
    use super::*;

    fn make_empty_plan(pipeline: &str) -> ExecutionPlan {
        ExecutionPlan {
            pipeline:   pipeline.to_string(),
            stages:     vec![],
            total_cost: CostEstimate { cpu_units: 0.0, memory_mb: 0.0, io_ops: 0 },
        }
    }

    fn make_cache(max_size: usize) -> PlanCache {
        PlanCache { entries: vec![], max_size }
    }

    #[test]
    fn plan_cache_hit() {
        let mut cache = make_cache(4);
        let plan = make_empty_plan("OrderPipeline");
        insert_plan(&mut cache, "hash_order", plan.clone());
        // lookup → Some
        let result = lookup_plan(&cache, "hash_order");
        assert!(result.is_some(), "expected cache hit");
        assert_eq!(result.unwrap().pipeline, "OrderPipeline");
        // 存在しない hash → None
        let miss = lookup_plan(&cache, "hash_missing");
        assert!(miss.is_none(), "expected cache miss");
    }

    #[test]
    fn plan_cache_evicts_oldest_on_full() {
        let mut cache = make_cache(2);
        // created_at: io_ops を代用（0, 0, 0 → 全て 0 なのでエビクション順が最初になる）
        // io_ops に異なる値を使い created_at を区別する
        let plan_a = ExecutionPlan {
            pipeline: "A".to_string(), stages: vec![],
            total_cost: CostEstimate { cpu_units: 0.0, memory_mb: 0.0, io_ops: 10 },
        };
        let plan_b = ExecutionPlan {
            pipeline: "B".to_string(), stages: vec![],
            total_cost: CostEstimate { cpu_units: 0.0, memory_mb: 0.0, io_ops: 20 },
        };
        let plan_c = ExecutionPlan {
            pipeline: "C".to_string(), stages: vec![],
            total_cost: CostEstimate { cpu_units: 0.0, memory_mb: 0.0, io_ops: 30 },
        };
        insert_plan(&mut cache, "hash_a", plan_a);  // created_at=10
        insert_plan(&mut cache, "hash_b", plan_b);  // created_at=20
        // max_size=2 → plan_a（created_at=10 が最小）がエビクションされ plan_c が挿入される
        insert_plan(&mut cache, "hash_c", plan_c);  // created_at=30
        assert!(lookup_plan(&cache, "hash_a").is_none(), "oldest entry should be evicted");
        assert!(lookup_plan(&cache, "hash_b").is_some(), "newer entry should remain");
        assert!(lookup_plan(&cache, "hash_c").is_some(), "newly inserted entry should exist");
    }
}
```

---

### Step 5: Cargo.toml バージョン更新

- `version` を `"78.7.0"` → `"78.8.0"` に変更
- driver.rs 内の `78.7.0` バージョン文字列アサーションを `78.8.0` に一括更新（`replace_all: true`）
- `grep -c "78.7.0" /c/Users/yoshi/favnir/fav/src/driver.rs` で **出力が 1** であることを確認（残るのは `// --- v78.7.0: Stream / Batch 統合実行モード ---` の 1 件のみ）

---

### Step 6: versions/current.md 更新

- 進行中: `v78.8.0`（実行計画キャッシュ）
- 次: `v78.9.0`（安定化・コードフリーズ）

---

### Step 7: 最終確認

- `cargo test v788000` → 2 tests pass
- `cargo test` → 全 pass（3780 tests）
- Cargo.toml version = `78.8.0`
- CHANGELOG 先頭 = `[v78.8.0]`
