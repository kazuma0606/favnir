# v78.9.0 実装計画 — 安定化・コードフリーズ

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `78.8.0` であることを確認
- `cargo test` が全 pass（3781 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

```markdown
## [v78.9.0] — 2026-08-16 — 安定化・コードフリーズ

### Added
- なし（新機能追加なし。バグ修正のみ受け入れ）

### Tests
- `execution_effects_full_sprint_all_stable`: v78.1〜v78.8 全型・関数の横断統合テスト
- `execution_effects_e2e_pipeline_runs`: パイプライン全体（モード選択 → コスト推定 → 計画生成 → キャッシュ → 取得 → 可視化）の E2E 動作確認
```

---

### Step 3: driver.rs — テストモジュール追加

`fav/src/driver.rs` の末尾に追加:

```rust
// --- v78.9.0: 安定化・コードフリーズ ---
// v789000_tests: execution_effects_full_sprint_all_stable, execution_effects_e2e_pipeline_runs (2 tests)

#[cfg(test)]
mod v789000_tests {
    use super::*;

    #[test]
    fn execution_effects_full_sprint_all_stable() {
        // v78.1: CacheEntry / check_cache_valid
        let config = CacheConfig { ttl_secs: 60, strategy: CacheStrategy::Lru, max_entries: 100 };
        let entry = CacheEntry { key: "k1".to_string(), inserted_at: 0, hits: 0 };
        assert!(check_cache_valid(&entry, 30, &config), "v78.1: cache should be valid within TTL");

        // v78.2: simulate_lru_cache / hit_rate
        let stats = simulate_lru_cache(&["a", "b", "a"], 2);
        assert!(hit_rate(&stats) >= 0.0, "v78.2: hit_rate should be non-negative");

        // v78.3: select_join_strategy
        let a_config = AdaptiveConfig { broadcast_threshold_rows: 1_000, default_parallelism: 4 };
        let strategy = select_join_strategy(50_000, 500, &a_config);
        assert_eq!(strategy, ExecutionStrategy::BroadcastJoin, "v78.3: small right table → BroadcastJoin");

        // v78.4: estimate_broadcast_cost / select_min_cost_strategy
        let bc = estimate_broadcast_cost(500);
        let hc = estimate_hash_cost(50_000, 500);
        let chosen = select_min_cost_strategy(&[
            (ExecutionStrategy::BroadcastJoin, bc),
            (ExecutionStrategy::HashJoin, hc),
        ]);
        assert_eq!(chosen, ExecutionStrategy::BroadcastJoin, "v78.4: broadcast cheaper for small table");

        // v78.5: format_execution_plan
        let plan = ExecutionPlan {
            pipeline: "StableCheck".to_string(),
            stages: vec![],
            total_cost: CostEstimate { cpu_units: 1.0, memory_mb: 64.0, io_ops: 100 },
        };
        let fmt = format_execution_plan(&plan);
        assert!(fmt.contains("Execution Plan: StableCheck"), "v78.5: format output correct");

        // v78.6: plan_parallel_execution
        let par_config = ParallelConfig { threads: 4, partition_count: 8, partition_key: "id".to_string() };
        let partitions = plan_parallel_execution(1000, &par_config);
        assert_eq!(partitions.len(), 8, "v78.6: partition count correct");

        // v78.7: select_execution_mode
        let selector = ExecutionModeSelector { row_threshold: 5_000, latency_target_ms: 500 };
        let mode = select_execution_mode(100, 1_000, &selector);
        assert_eq!(mode, ExecutionMode::Adaptive, "v78.7: small data + loose latency → Adaptive");

        // v78.8: insert_plan / lookup_plan
        let mut cache = PlanCache { entries: vec![], max_size: 4 };
        insert_plan(&mut cache, "stable_hash", plan.clone());
        assert!(lookup_plan(&cache, "stable_hash").is_some(), "v78.8: cache hit after insert");
    }

    #[test]
    fn execution_effects_e2e_pipeline_runs() {
        // Step 1: モード選択
        let selector = ExecutionModeSelector { row_threshold: 5_000, latency_target_ms: 500 };
        let mode = select_execution_mode(10_000, 1_000, &selector);
        assert_eq!(mode, ExecutionMode::Batch, "e2e: large data → Batch");

        // Step 2: コスト推定
        let bc = estimate_broadcast_cost(500);
        let hc = estimate_hash_cost(10_000, 500);

        // Step 3: 最小コスト戦略選択
        // bc.clone(): select_min_cost_strategy に渡した後 PlanStage.cost にムーブするため clone が必要
        let strategy = select_min_cost_strategy(&[
            (ExecutionStrategy::BroadcastJoin, bc.clone()),
            (ExecutionStrategy::HashJoin, hc),
        ]);
        assert_eq!(strategy, ExecutionStrategy::BroadcastJoin, "e2e: broadcast cheaper for small right");

        // Step 4: 実行計画構築 + 可視化
        let plan = ExecutionPlan {
            pipeline: "E2EPipeline".to_string(),
            stages: vec![PlanStage {
                name: "Join".to_string(),
                operation: "BroadcastJoin".to_string(),
                cost: bc,
                strategy: Some(strategy),
            }],
            total_cost: CostEstimate { cpu_units: 5.0, memory_mb: 50.0, io_ops: 500 },
        };
        let fmt = format_execution_plan(&plan);
        assert!(fmt.contains("E2EPipeline"), "e2e: plan contains pipeline name");
        assert!(fmt.contains("BroadcastJoin"), "e2e: plan contains strategy");

        // Step 5: キャッシュ挿入 → 取得
        let mut cache = PlanCache { entries: vec![], max_size: 4 };
        insert_plan(&mut cache, "e2e_hash", plan);
        let cached = lookup_plan(&cache, "e2e_hash");
        assert!(cached.is_some(), "e2e: cache hit");
        assert_eq!(cached.unwrap().pipeline, "E2EPipeline", "e2e: pipeline name preserved in cache");
    }
}
```

---

### Step 4: Cargo.toml バージョン更新

- `version` を `"78.8.0"` → `"78.9.0"` に変更
- driver.rs 内の `78.8.0` バージョン文字列アサーションを `78.9.0` に一括更新（`replace_all: true`）
- `grep -c "78.8.0" /c/Users/yoshi/favnir/fav/src/driver.rs` で **出力が 1** であることを確認（Git Bash で実行）
  - `replace_all: true` で約 28 件のアサーション行（`contains("version = \"78.8.0\"")`）が一括更新されるため、置換後の残存は コメント行 `// --- v78.8.0: 実行計画キャッシュ ---` の 1 件のみになる（29 → 1 に減少）

---

### Step 5: versions/current.md 更新

- 進行中: `v78.9.0`（安定化・コードフリーズ）
- 次: `v79.0.0`（Execution Effects 1.0 宣言）

---

### Step 6: 最終確認

- `cargo test v789000` → 2 tests pass
- `cargo test` → 全 pass（3783 tests）
- Cargo.toml version = `78.9.0`
- CHANGELOG 先頭 = `[v78.9.0]`
