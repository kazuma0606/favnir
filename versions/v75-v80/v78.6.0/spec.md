# v78.6.0 仕様書 — `!Parallel` エフェクト統合

Date: 2026-08-16

---

## Background

v78.1.0〜v78.5.0 で `!Cached` / `!Adaptive` エフェクト基盤・コスト推定・実行計画可視化を整備した。
v78.6.0 では `!Parallel` エフェクトを統合し、パイプラインの並列分割設定を型として宣言的に制御できる基盤を追加する。
既存の `par [A, B]` 構文はステージレベルの並列化だが、`!Parallel` エフェクトはデータ分割（シャーディング）レベルの並列設定を表現する。

---

## Goals

- `ParallelConfig` 構造体を追加して並列スレッド数・パーティション数・パーティションキーを保持する
- `PartitionPlan` 構造体を追加して個々の分割計画（パーティション ID・行数推定・スレッド ID）を表現する
- `plan_parallel_execution` 関数で `total_rows` と設定から `Vec<PartitionPlan>` を生成する
- `format_parallel_plan` 関数で並列計画の可読テキスト出力を生成する
- 対象: `fav/src/driver.rs` のみ（他ファイル変更なし）

---

## Syntax / API

```rust
// ParallelConfig: 並列実行の設定
pub struct ParallelConfig {
    pub threads:         usize,
    pub partition_count: usize,
    pub partition_key:   String,
}

// PartitionPlan: 1 パーティションの実行計画
pub struct PartitionPlan {
    pub partition_id:  usize,
    pub rows_estimate: u64,
    pub thread_id:     usize,
}

// plan_parallel_execution: total_rows を partition_count で分割し Vec<PartitionPlan> を生成
// - partition_count == 0 の場合は空 Vec を返す
// - thread_id は partition_id % threads で割り当て（threads == 0 のときは 0 を返す）
// - 端数行（total_rows % partition_count）は最後のパーティションに加算
pub fn plan_parallel_execution(total_rows: u64, config: &ParallelConfig) -> Vec<PartitionPlan>

// format_parallel_plan: 並列計画をテキスト形式で可視化
// - 空スライスの場合は "No partitions." を返す
// 出力形式:
//   Parallel Plan: {partition_count} partitions / {threads} threads
//     Partition 0: ~{rows} rows  thread={thread_id}
//     ...
//     Total rows: {total}
pub fn format_parallel_plan(plans: &[PartitionPlan]) -> String
```

Favnir 構文例:
```favnir
fn process_shards(ctx: AppCtx) -> Result<List<Row>, String> !Parallel {
    bind results <- List.map(shards, process_shard)
    Result.ok(List.flatten(results))
}
```

---

## Success Criteria

1. `ParallelConfig` / `PartitionPlan` が `driver.rs` に追加されコンパイルエラーなし
2. `plan_parallel_execution` が正しいパーティション数の `Vec<PartitionPlan>` を返す
3. `format_parallel_plan` がヘッダー・各パーティション行・トータル行を含む文字列を返す
4. Rust テスト 2 件（`v786000_tests` モジュール）が pass
5. `cargo test` 全 pass（3772 + 2 = 3774 tests）

---

## Error Codes

なし（本バージョンでは新規エラーコード追加なし）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `ParallelConfig` / `PartitionPlan` 構造体、`plan_parallel_execution` / `format_parallel_plan` 関数、`v786000_tests` モジュール追加 |
| `fav/Cargo.toml` | version を `78.5.0` → `78.6.0` に変更 |
| `CHANGELOG.md` | v78.6.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v78.6.0 に更新 |
