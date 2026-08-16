# v78.7.0 仕様書 — Stream / Batch 統合実行モード

Date: 2026-08-16

---

## Background

v78.6.0 で並列実行の基盤（`ParallelConfig` / `PartitionPlan`）を整備した。
v78.7.0 では、同一パイプラインをデータ量・レイテンシ要件に応じて Streaming / Batch / Adaptive で自動切り替えする `ExecutionMode` 型と選択ロジックを追加する。
既存の `!Adaptive` エフェクト基盤（v78.3.0）・コスト推定（v78.4.0）・実行計画可視化（v78.5.0）と組み合わせて、実行モードを型レベルで宣言的に制御できる基盤を形成する。

---

## Goals

- `ExecutionMode` enum（Batch / Streaming / Adaptive）を追加する
- `ExecutionModeSelector` 構造体（row_threshold: u64, latency_target_ms: u64）を追加する
- `select_execution_mode` 関数でデータ量・レイテンシ要件から最適モードを選択する
- 対象: `fav/src/driver.rs` のみ（他ファイル変更なし）

---

## Syntax / API

```rust
// ExecutionMode: 実行モードの選択肢
pub enum ExecutionMode { Batch, Streaming, Adaptive }

// ExecutionModeSelector: モード選択の閾値設定（HashMap キーとして使用しないため Hash は省略）
pub struct ExecutionModeSelector {
    pub row_threshold:      u64,  // この行数を超えたら Batch を優先
    pub latency_target_ms:  u64,  // この値より小さいレイテンシ要求なら Streaming を優先
}

// select_execution_mode: データ量とレイテンシ要件からモードを選択
// 優先順位:
//   1. latency_target_ms_req < selector.latency_target_ms → Streaming
//   2. est_rows > selector.row_threshold                  → Batch
//   3. それ以外                                           → Adaptive
pub fn select_execution_mode(
    est_rows: u64,
    latency_target_ms_req: u64,
    selector: &ExecutionModeSelector,
) -> ExecutionMode
```

Favnir 構文例:
```favnir
fn ingest(ctx: AppCtx) -> Result<Unit, String> !Adaptive {
    bind mode <- ExecutionMode.select(ctx.config)
    match mode {
        Streaming -> stream_ingest(ctx)
        Batch     -> batch_ingest(ctx)
        Adaptive  -> batch_ingest(ctx)
    }
}
```

---

## 選択ロジック詳細

| 条件 | モード | 理由 |
|---|---|---|
| `latency_req < selector.latency_target_ms` | Streaming | レイテンシ要件が閾値より厳しい → 低遅延優先 |
| `est_rows > selector.row_threshold` | Batch | 大量データ → スループット優先 |
| それ以外 | Adaptive | 自動調整（デフォルト） |

latency 判定を row 判定より先に行う（低レイテンシ要件はデータ量より優先）。

---

## Success Criteria

1. `ExecutionMode` / `ExecutionModeSelector` が `driver.rs` に追加されコンパイルエラーなし
2. `select_execution_mode` が優先順位に従って正しいモードを返す
3. Rust テスト 2 件（`v787000_tests` モジュール）が pass
4. `cargo test` 全 pass（3775 + 2 = 3777 tests）

---

## Error Codes

なし（本バージョンでは新規エラーコード追加なし）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `ExecutionMode` enum、`ExecutionModeSelector` 構造体、`select_execution_mode` 関数、`v787000_tests` モジュール追加 |
| `fav/Cargo.toml` | version を `78.6.0` → `78.7.0` に変更 |
| `fav/Cargo.lock` | Cargo.toml version 変更に伴う自動更新 |
| `CHANGELOG.md` | v78.7.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v78.7.0 に更新 |
