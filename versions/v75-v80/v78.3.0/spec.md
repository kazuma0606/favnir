# v78.3.0 仕様書 — `!Adaptive` エフェクト基盤

Date: 2026-08-16

---

## Background

v78.1.0 / v78.2.0 でキャッシュエフェクト（`!Cached`）の型基盤と LRU 統計を導入した。
v78.3.0 では実行戦略を自動選択する `!Adaptive` エフェクトの型基盤を追加する。
具体的には、テーブルの行数に基づいて broadcast join / hash join / sort-merge join を
ランタイムで選択するロジックを型で表現する。

`ExecutionStrategy` enum と `AdaptiveConfig` 構造体は、v78.4.0 のコスト推定モデルおよび
v78.5.0 の `fav explain plan` 可視化で参照される基盤型となる。

---

## Goals

1. `ExecutionStrategy` enum を追加する（BroadcastJoin / HashJoin / SortMergeJoin / Auto）
2. `AdaptiveConfig` 構造体を追加する（broadcast_threshold_rows: u64, default_parallelism: usize）
3. `select_join_strategy` 関数を追加する（行数比較でストラテジーを選択）
4. `format_strategy_selected` 関数を追加する（選択されたストラテジーの可読表現）
5. テスト 2 件を追加する（3766 → 3768）

---

## API 仕様

### `ExecutionStrategy`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionStrategy {
    BroadcastJoin,  // 小テーブルを全ノードにブロードキャスト
    HashJoin,       // ハッシュ分割して並列 join
    SortMergeJoin,  // ソート済みストリームをマージ join
    Auto,           // ランタイムで自動選択（デフォルト）
}
```

### `AdaptiveConfig`

```rust
// Hash は付与しない（設定構造体を HashMap のキーとして使用しないため）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveConfig {
    pub broadcast_threshold_rows: u64,   // この行数以下なら BroadcastJoin を選択
    pub default_parallelism:      usize, // HashJoin / SortMergeJoin のデフォルト並列度
}
```

### `select_join_strategy`

```rust
pub fn select_join_strategy(
    left_rows: u64,
    right_rows: u64,
    config: &AdaptiveConfig,
) -> ExecutionStrategy
```

ロジック:
- `min(left_rows, right_rows) <= config.broadcast_threshold_rows` → `BroadcastJoin`
- それ以外 → `HashJoin`
- （SortMergeJoin / Auto は v78.4.0 以降のコスト推定で選択される予定）

### `format_strategy_selected`

```rust
pub fn format_strategy_selected(strategy: &ExecutionStrategy) -> String
```

出力例:

| variant | 出力文字列 |
|---|---|
| `BroadcastJoin` | `"Strategy: BroadcastJoin (small table detected)"` |
| `HashJoin` | `"Strategy: HashJoin (large table, hash partition)"` |
| `SortMergeJoin` | `"Strategy: SortMergeJoin (sorted stream merge)"` |
| `Auto` | `"Strategy: Auto (runtime selection)"` |

---

## Favnir コード例

```favnir
fn join_customers(ctx: AppCtx) -> Result<List<Row>, String> !Adaptive {
    bind customers <- ctx.io.query("SELECT * FROM customers")
    bind orders    <- ctx.io.query("SELECT * FROM orders")
    // → row 数に応じて broadcast / hash join を自動選択
    Result.ok(customers |> join(orders, on: "id"))
}
```

※ `!Adaptive` エフェクトの型チェック統合は将来バージョン。v78.3.0 は型基盤のみ。

---

## Success Criteria

- `ExecutionStrategy` が `Debug / Clone / PartialEq / Eq / Hash` を持つ
- `AdaptiveConfig` が `Debug / Clone / PartialEq / Eq` を持つ
- `select_join_strategy` が `min(left, right) <= threshold` で `BroadcastJoin` を返す
- `select_join_strategy` が大テーブルで `HashJoin` を返す
- `format_strategy_selected` が 4 variant すべてに対し想定文字列を返す
- テスト 2 件（`adaptive_selects_broadcast_for_small_table` / `adaptive_selects_hash_for_large_table`）が pass する
- `cargo test` 全体が 3768 tests pass する

---

## Notes

- `changelog_has_v78_3_0` テストは x.0.0 宣言バージョンのみに追加する慣例につき、本バージョンでは追加しない。

---

## Files to Modify

- `fav/src/driver.rs` — 型・関数・テストモジュール追加
- `CHANGELOG.md` — v78.3.0 エントリ追加
- `fav/Cargo.toml` — version を `78.2.0` → `78.3.0` に更新
- `versions/current.md` — 進行中バージョン更新

---

## Error Codes

新規エラーコードなし。
