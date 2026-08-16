# Roadmap v78.1.0 〜 v79.0.0 — Execution Effects 1.0

Date: 2026-08-14
Status: 未着手（v78.0.0 完了後に開始）

マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)

---

## 前提

- 直前完了: v78.0.0「Verifiable Pipelines 宣言」（tests = 3758）
- 本スプリントは Phase 6「Favnir 3.0 宣言」の第 4 スプリント
- 目標: v79.0.0「Execution Effects 1.0 宣言」（tests = 3780）

### スプリントの性格

実行戦略をエフェクトで宣言する。`!Cached` / `!Adaptive` / `!Parallel` を
エフェクト型として統一し、パイプラインの「どう動くか」を型で制御する。
既存の `par [A, B]` 構文は `!Parallel` エフェクトに統合される。
A（新言語機能）50% + B（実行基盤）50% の構成。

### エフェクト設計方針

既存エフェクト（`!IO`, `!Http`, `!Snowflake` 等）は「外部副作用」を表す。
実行戦略エフェクトは「実行の**やり方**」を宣言する点で同じエフェクト系に属し、
`fav.toml` の `[effects.*]` セクションで設定する。

```toml
[effects.cached]
ttl_secs    = 300
strategy    = "lru"
max_entries = 1000

[effects.adaptive]
broadcast_threshold_rows = 10_000
default_parallelism      = 8

[effects.parallel]
threads         = 8
partition_count = 16
```

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v78.1.0 | `!Cached` エフェクト基盤 | 3760 + 3 = 3763 | 完了（code-reviewer 対応で境界値テスト +1 追加） |
| v78.2.0 | キャッシュ戦略型（LRU / FIFO / LFU） | 3763 + 3 = 3766 | 完了（code-reviewer 対応で max_entries==0 テスト +1 追加） |
| v78.3.0 | `!Adaptive` エフェクト基盤 | 3766 + 2 = 3768 | 未着手 |
| v78.4.0 | コスト推定モデル | 3768 + 2 = 3770 | 未着手 |
| v78.5.0 | `fav explain plan` 可視化 | 3770 + 2 = 3772 | 未着手 |
| v78.6.0 | `!Parallel` エフェクト統合 | 3772 + 3 = 3775 | 完了（code-reviewer 対応で境界値テスト +1 追加） |
| v78.7.0 | Stream / Batch 統合実行モード | 3775 + 3 = 3778 | 完了（code-reviewer 対応で Adaptive 境界値テスト +1 追加） |
| v78.8.0 | 実行計画キャッシュ | 3778 + 3 = 3781 | 完了（code-reviewer 対応で境界値テスト +1 追加） |
| v78.9.0 | 安定化・コードフリーズ | 3781 + 2 = 3783 | 未着手 |
| v79.0.0 | Execution Effects 1.0 宣言 ★クリーンアップ | 3783 + 4 = 3787 | 未着手 |

---

## v78.1.0 — `!Cached` エフェクト基盤

関数の結果をキャッシュすることを宣言するエフェクト。繰り返し呼ばれる参照データ取得に有効。

```favnir
fn get_exchange_rate(currency: String) -> Result<Float, String> !Cached {
    ctx.io.fetch(f"https://api.rates.io/{currency}")
    // → TTL 内は同じ currency への呼び出しをキャッシュから返す
}
```

**実装内容:**
- `CacheStrategy` enum（Lru, Fifo, Lfu）
- `CacheConfig` 構造体（ttl_secs: u64, strategy: CacheStrategy, max_entries: usize）
- `CacheEntry` 構造体（key: String, inserted_at: i64, hits: u64）
- `check_cache_valid(entry: &CacheEntry, now: i64, config: &CacheConfig) -> bool`

**完了条件**: Rust テスト 2 件（3760 + 2 = 3762）
> **注**: v78.0.0 実績は 3760（v77.8.0 code-reviewer 対応で +2 追加）。ベースを 3760 に修正。
- `cache_entry_valid_within_ttl`
- `cache_entry_expired`

---

## v78.2.0 — キャッシュ戦略型（LRU / FIFO / LFU）

各キャッシュ戦略の動作を型として表現し、ヒット率・エビクション数などの統計を扱う。

```bash
$ fav cache stats --pipeline pipeline.fav
Cache Stats:
  hits:      8432 (84.3%)
  misses:    1568
  evictions: 204
  strategy:  LRU (max=1000 entries)
```

**実装内容:**
- `CacheStats` 構造体（hits: u64, misses: u64, evictions: u64）
- `simulate_lru_cache(accesses: &[&str], max_entries: usize) -> CacheStats`
- `format_cache_stats_report(stats: &CacheStats) -> String`
- `hit_rate(stats: &CacheStats) -> f64`

**完了条件**: Rust テスト 2 件（3760 + 2 = 3762）
- `lru_evicts_least_recently_used`
- `cache_hit_rate_calculated`

---

## v78.3.0 — `!Adaptive` エフェクト基盤

実行戦略をランタイム統計に基づいて自動選択することを宣言するエフェクト。

```favnir
fn join_customers(ctx: AppCtx) -> Result<List<Row>, String> !Adaptive {
    bind customers <- ctx.io.query("SELECT * FROM customers")
    bind orders    <- ctx.io.query("SELECT * FROM orders")
    // → row 数に応じて broadcast / hash join を自動選択
    Result.ok(customers |> join(orders, on: "id"))
}
```

**実装内容:**
- `ExecutionStrategy` enum（BroadcastJoin, HashJoin, SortMergeJoin, Auto）
- `AdaptiveConfig` 構造体（broadcast_threshold_rows: u64, default_parallelism: usize）
- `select_join_strategy(left_rows: u64, right_rows: u64, config: &AdaptiveConfig) -> ExecutionStrategy`
- `format_strategy_selected(strategy: &ExecutionStrategy) -> String`

**完了条件**: Rust テスト 2 件（3766 + 2 = 3768）
- `adaptive_selects_broadcast_for_small_table`
- `adaptive_selects_hash_for_large_table`

---

## v78.4.0 — コスト推定モデル

各実行戦略のコスト（CPU・メモリ・IO）を推定し、最適戦略を選択するモデル。

```bash
$ fav explain plan pipeline.fav --estimate-cost
Join Strategy Analysis:
  BroadcastJoin: CPU=2.1 units, Mem=128MB,  IO=45k ops  ← selected
  HashJoin:      CPU=5.8 units, Mem=512MB,  IO=12k ops
  SortMerge:     CPU=8.2 units, Mem=256MB,  IO=98k ops
```

**実装内容:**
- `CostEstimate` 構造体（cpu_units: f64, memory_mb: f64, io_ops: u64）
- `estimate_broadcast_cost(right_rows: u64) -> CostEstimate`
- `estimate_hash_cost(left_rows: u64, right_rows: u64) -> CostEstimate`
- `select_min_cost_strategy(estimates: &[(ExecutionStrategy, CostEstimate)]) -> ExecutionStrategy`

**完了条件**: Rust テスト 2 件（3768 + 2 = 3770）
- `cost_estimate_broadcast_cheaper_for_small`
- `cost_estimate_hash_wins_for_large`

---

## v78.5.0 — `fav explain plan` 可視化

パイプラインの実行計画をテキスト形式で可視化するコマンド。

```bash
$ fav explain plan pipeline.fav
Execution Plan: OrderPipeline
  Stage 1: LoadOrders        [IO]       cost=1.2 units
  Stage 2: JoinCustomers     [Adaptive] cost=2.1 units  → BroadcastJoin
  Stage 3: AggregateRegion   [Cached]   cost=0.3 units  → cache hit expected
  ───────────────────────────────────────────────────
  Total: 3.6 units  |  Memory peak: 128MB
```

**実装内容:**
- `PlanStage` 構造体（name: String, operation: String, cost: CostEstimate, strategy: Option<ExecutionStrategy>）
- `ExecutionPlan` 構造体（pipeline: String, stages: Vec<PlanStage>, total_cost: CostEstimate）
- `format_execution_plan(plan: &ExecutionPlan) -> String`

**完了条件**: Rust テスト 2 件（3770 + 2 = 3772）
- `explain_plan_format_output`
- `explain_plan_total_cost_summed`

---

## v78.6.0 — `!Parallel` エフェクト統合

既存の `par [A, B]` 構文と `!Parallel` エフェクトを統合し、並列設定を宣言的に制御する。

```favnir
fn process_shards(ctx: AppCtx) -> Result<List<Row>, String> !Parallel {
    bind results <- List.map(shards, process_shard)
    // → fav.toml の [effects.parallel] 設定に従ってスレッド分割
    Result.ok(List.flatten(results))
}
```

**実装内容:**
- `ParallelConfig` 構造体（threads: usize, partition_count: usize, partition_key: String）
- `PartitionPlan` 構造体（partition_id: usize, rows_estimate: u64, thread_id: usize）
- `plan_parallel_execution(total_rows: u64, config: &ParallelConfig) -> Vec<PartitionPlan>`
- `format_parallel_plan(plans: &[PartitionPlan]) -> String`

**完了条件**: Rust テスト 2 件（3772 + 2 = 3774）
- `parallel_plan_creates_correct_partitions`
- `parallel_plan_distributes_evenly`

---

## v78.7.0 — Stream / Batch 統合実行モード

同一パイプラインをデータ量・レイテンシ要件に応じて Streaming / Batch で自動切り替えする。

```favnir
fn ingest(ctx: AppCtx) -> Result<Unit, String> !Adaptive {
    bind mode <- ExecutionMode.select(ctx.config)
    match mode {
        Streaming -> stream_ingest(ctx)
        Batch     -> batch_ingest(ctx)
        Adaptive  -> batch_ingest(ctx)   // デフォルト fallback
    }
}
```

**実装内容:**
- `ExecutionMode` enum（Batch, Streaming, Adaptive）
- `ExecutionModeSelector` 構造体（row_threshold: u64, latency_target_ms: u64）
- `select_execution_mode(est_rows: u64, latency_target_ms_req: u64, selector: &ExecutionModeSelector) -> ExecutionMode`

**完了条件**: Rust テスト 2 件（3775 + 2 = 3777）
- `mode_batch_for_large_data`
- `mode_streaming_for_low_latency`

---

## v78.8.0 — 実行計画キャッシュ

同じパイプラインへの繰り返し実行で実行計画を再利用し、計画生成のオーバーヘッドを削減する。

```favnir
bind plan <- PlanCache.lookup(pipeline_hash)
match plan {
    Some(p) -> p
    None    -> {
        bind p  <- plan_pipeline(pipeline)
        bind _  <- PlanCache.insert(pipeline_hash, p)
        p
    }
}
```

**実装内容:**
- `PlanCacheEntry` 構造体（pipeline_hash: String, plan: ExecutionPlan, created_at: i64）
- `PlanCache` 構造体（entries: Vec<PlanCacheEntry>, max_size: usize）
- `lookup_plan<'a>(cache: &'a PlanCache, hash: &str) -> Option<&'a ExecutionPlan>`
- `insert_plan(cache: &mut PlanCache, hash: &str, plan: ExecutionPlan)` — oldest-first エビクション付き（`created_at` 最小エントリを削除、注: `lookup_plan` が `&'a PlanCache` を取るため真の LRU は実現不可）

**完了条件**: Rust テスト 2 件（3778 + 2 = 3780）
- `plan_cache_hit`
- `plan_cache_evicts_oldest_on_full`

---

## v78.9.0 — 安定化・コードフリーズ（Execution Effects 前最終調整）

v78.1〜v78.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- v78.1〜v78.8 の全テスト通過確認（`cargo test` 全 pass）
- `!Cached` / `!Adaptive` / `!Parallel` エフェクトの E2E 動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3781 + 2 = 3783）
- `execution_effects_full_sprint_all_stable`
- `execution_effects_e2e_pipeline_runs`

---

## v79.0.0 — Execution Effects 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「`!Cached` がメモを持ち、`!Adaptive` が状況を読み、`!Parallel` が仕事を分ける。
>  実行戦略が型となった Favnir は、最適解を自ら選ぶ。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `79.0.0` に更新
- `CHANGELOG.md` に v79.0.0 エントリを追加
- `MILESTONE.md` に「Execution Effects 1.0」を追記
- `README.md` に v79.0 達成を追記
- `versions/current.md` を更新

**完了条件**: `v79000_tests` 4 件（3783 + 4 = 3787）
- `cargo_toml_version_is_79_0_0`
- `changelog_has_v79_0_0`
- `milestone_has_execution_effects`
- `readme_mentions_execution_effects`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v78.0.0（ベース） | 3,760 | —（v77.8.0 code-reviewer 対応で +2 追加、計画 3,758 → 実績 3,760） |
| v78.1.0 | 3,763 | +3（計画 +2、code-reviewer 対応で境界値テスト +1 追加） |
| v78.2.0 | 3,766 | +3（計画 +2、code-reviewer 対応で max_entries==0 テスト +1 追加） |
| v78.3.0 | 3,768 | +2 |
| v78.4.0 | 3,770 | +2 |
| v78.5.0 | 3,772 | +2 |
| v78.6.0 | 3,775 | +3（計画 +2、code-reviewer 対応で境界値テスト +1 追加） |
| v78.7.0 | 3,778 | +3（計画 +2、code-reviewer 対応で Adaptive 境界値テスト +1 追加） |
| v78.8.0 | 3,781 | +3（計画 +2、code-reviewer 対応で境界値テスト +1 追加） |
| v78.9.0 | 3,783 | +2 |
| v79.0.0（宣言） | 3,787 | +4 |

**本スプリント合計**: +27 tests（3,760 → 3,787）

---

## 参考リンク

- マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)
- 前スプリント: [roadmap-v77.1-v78.0.md](roadmap-v77.1-v78.0.md)
- 次スプリント: [roadmap-v79.1-v80.0.md](roadmap-v79.1-v80.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
