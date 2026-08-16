# v78.4.0 仕様書 — コスト推定モデル

Date: 2026-08-16

---

## Background

v78.3.0 で `ExecutionStrategy` と `AdaptiveConfig` を導入し、行数ベースの単純なストラテジー選択を実装した。
v78.4.0 では CPU・メモリ・IO の 3 軸コスト推定モデルを追加し、コスト最小の戦略を選択できるようにする。
`CostEstimate` は v78.5.0 の `fav explain plan` 可視化でも参照される基盤型。

---

## Goals

1. `CostEstimate` 構造体を追加する（cpu_units: f64, memory_mb: f64, io_ops: u64）
2. `estimate_broadcast_cost(right_rows: u64) -> CostEstimate` を追加する
3. `estimate_hash_cost(left_rows: u64, right_rows: u64) -> CostEstimate` を追加する
4. `select_min_cost_strategy(estimates: &[(ExecutionStrategy, CostEstimate)]) -> ExecutionStrategy` を追加する
5. テスト 2 件を追加する（3768 → 3770）

---

## API 仕様

### `CostEstimate`

```rust
// f64 フィールドを含むため Eq / Hash は付与しない。PartialEq のみ。
#[derive(Debug, Clone, PartialEq)]
pub struct CostEstimate {
    pub cpu_units:  f64, // CPU 処理コスト（任意単位）
    pub memory_mb:  f64, // メモリ使用量（MB）
    pub io_ops:     u64, // IO 操作数
}
```

### `estimate_broadcast_cost`

```rust
pub fn estimate_broadcast_cost(right_rows: u64) -> CostEstimate
```

- Broadcast join は右テーブルをそのまま全ノードへ送るため、行数に比例してコストが増加する
- フォーミュラ:
  - `cpu_units  = right_rows as f64 * 0.01`
  - `memory_mb  = right_rows as f64 * 0.1`
  - `io_ops     = right_rows`

### `estimate_hash_cost`

```rust
pub fn estimate_hash_cost(left_rows: u64, right_rows: u64) -> CostEstimate
```

- Hash join は基本コストが高いが行数スケールは小さい（パーティション分散のため）
- フォーミュラ:
  - `cpu_units  = 5.0 + (left_rows + right_rows) as f64 * 0.0001`
  - `memory_mb  = (left_rows + right_rows) as f64 * 0.01`
  - `io_ops     = (left_rows + right_rows) / 2`

### `select_min_cost_strategy`

```rust
pub fn select_min_cost_strategy(
    estimates: &[(ExecutionStrategy, CostEstimate)],
) -> ExecutionStrategy
```

- `cpu_units` が最小のエントリの `ExecutionStrategy` を返す
- スライスが空の場合は `ExecutionStrategy::Auto` を返す（フォールバック）
- `cpu_units` が `f64::NAN` の場合の動作は未定義。呼び出し元が非 NaN 値を保証すること。
  実装上は `partial_cmp` が `None` を返した場合 `Ordering::Equal` で処理するが、
  結果の順序は保証しない。

---

## コスト比較例

```
$ fav explain plan pipeline.fav --estimate-cost
Join Strategy Analysis:
  BroadcastJoin: CPU=2.1 units, Mem=128MB, IO=45k ops  ← selected
  HashJoin:      CPU=5.8 units, Mem=512MB, IO=12k ops
  SortMerge:     CPU=8.2 units, Mem=256MB, IO=98k ops
```

---

## テストシナリオ

### `cost_estimate_broadcast_cheaper_for_small`

- `right_rows=100`, `left_rows=10_000`
- broadcast: `cpu = 100 * 0.01 = 1.0`
- hash: `cpu = 5.0 + (10_000 + 100) * 0.0001 ≈ 6.01`
- `select_min_cost_strategy` → `BroadcastJoin`

### `cost_estimate_hash_wins_for_large`

- `right_rows=50_000`, `left_rows=10_000`
- broadcast: `cpu = 50_000 * 0.01 = 500.0`
- hash: `cpu = 5.0 + (10_000 + 50_000) * 0.0001 = 11.0`
- `select_min_cost_strategy` → `HashJoin`

---

## Success Criteria

- `CostEstimate` が `Debug / Clone / PartialEq` を持つ（`Eq` / `Hash` は f64 のため付与しない）
- `estimate_broadcast_cost` / `estimate_hash_cost` が仕様どおりの値を返す
- `select_min_cost_strategy` が `cpu_units` 最小のストラテジーを返す
- 空スライスの場合 `ExecutionStrategy::Auto` を返す
- テスト 2 件（`cost_estimate_broadcast_cheaper_for_small` / `cost_estimate_hash_wins_for_large`）が pass する
- `cargo test` 全体が 3770 tests pass する

---

## Notes

- `changelog_has_v78_4_0` テストは x.0.0 宣言バージョンのみに追加する慣例につき、本バージョンでは追加しない。

---

## Files to Modify

- `fav/src/driver.rs` — 型・関数・テストモジュール追加
- `CHANGELOG.md` — v78.4.0 エントリ追加
- `fav/Cargo.toml` — version を `78.3.0` → `78.4.0` に更新
- `versions/current.md` — 進行中バージョン更新

---

## Error Codes

新規エラーコードなし。
