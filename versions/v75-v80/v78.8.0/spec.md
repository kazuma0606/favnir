# v78.8.0 仕様書 — 実行計画キャッシュ

Date: 2026-08-16

---

## Background

v78.5.0 で `ExecutionPlan` / `format_execution_plan` を整備した。
v78.8.0 では同じパイプラインへの繰り返し実行で実行計画を再利用し、計画生成のオーバーヘッドを削減する `PlanCache` 型と操作関数を追加する。
キャッシュは `pipeline_hash` をキーとして `ExecutionPlan` を保持し、上限到達時は最も古いエントリ（`created_at` 最小）をエビクションする。

---

## Goals

- `PlanCacheEntry` 構造体（pipeline_hash / plan / created_at）を追加する
- `PlanCache` 構造体（entries / max_size）を追加する
- `lookup_plan` 関数でハッシュから実行計画を取得する（ライフタイム付き返値）
- `insert_plan` 関数でキャッシュに実行計画を挿入し、上限超過時は最古エントリをエビクションする
- 対象: `fav/src/driver.rs` のみ（他ファイル変更なし）

---

## Syntax / API

```rust
// PlanCacheEntry: キャッシュの 1 エントリ
// ExecutionPlan（f64 含む）を内包するため Eq / Hash は付与しない
pub struct PlanCacheEntry {
    pub pipeline_hash: String,
    pub plan:          ExecutionPlan,
    pub created_at:    i64,   // Unix timestamp（秒）
}

// PlanCache: 実行計画キャッシュ
// Vec<PlanCacheEntry> を内包するため Eq / Hash は付与しない
pub struct PlanCache {
    pub entries:  Vec<PlanCacheEntry>,
    pub max_size: usize,
}

// lookup_plan: hash が一致するエントリの &ExecutionPlan を返す
// 存在しない場合は None
pub fn lookup_plan<'a>(cache: &'a PlanCache, hash: &str) -> Option<&'a ExecutionPlan>

// insert_plan: キャッシュに plan を挿入する
// - hash が既存の場合は plan を上書き（created_at を更新）
// - entries.len() >= max_size（かつ hash が新規）の場合、
//   created_at が最小の既存エントリをエビクションしてから挿入
// - max_size == 0 の場合は挿入しない
pub fn insert_plan(cache: &mut PlanCache, hash: &str, plan: ExecutionPlan)
```

Favnir 構文例:
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

---

## 実装詳細

### `insert_plan` のエビクション戦略

oldest-first（挿入時刻基準）エビクションを採用する。`lookup_plan` のシグネチャが `&'a PlanCache`（不変参照）のため、アクセス時刻を更新する真の LRU は実現不可能。

- `max_size == 0` → 何もしない（早期リターン）
- `hash` が既存 → `plan` と `created_at` を上書き（エビクションなし）
- `entries.len() < max_size` → そのまま `push`
- `entries.len() >= max_size`（新規 hash）→ `created_at` 最小のエントリをエビクション後に `push`

### `created_at` の実装詳細と制約

`created_at` には `plan.total_cost.io_ops as i64` をタイムスタンプの代用として使用する。

制約:
- `io_ops` が `i64::MAX` を超える場合は未定義動作（現実的なデータ規模では非発生）
- 複数エントリの `io_ops` が同値の場合、エビクション対象は先に挿入されたエントリになる（`min_by_key` が最初の最小値を選択）
- `io_ops == 0` のプランは全て `created_at = 0` となるため、エビクション対象は常に先頭エントリになる

---

## Success Criteria

1. `PlanCacheEntry` / `PlanCache` が `driver.rs` に追加されコンパイルエラーなし
2. `lookup_plan` が正しく `Option<&'a ExecutionPlan>` を返す
3. `insert_plan` がエビクションロジックを正しく動作させる
4. Rust テスト 2 件（`v788000_tests` モジュール）が pass
5. `cargo test` 全 pass（3778 + 2 = 3780 tests）

---

## Error Codes

なし（本バージョンでは新規エラーコード追加なし）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `PlanCacheEntry` / `PlanCache` 構造体、`lookup_plan` / `insert_plan` 関数、`v788000_tests` モジュール追加 |
| `fav/Cargo.toml` | version を `78.7.0` → `78.8.0` に変更 |
| `fav/Cargo.lock` | Cargo.toml version 変更に伴う自動更新 |
| `CHANGELOG.md` | v78.8.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v78.8.0 に更新 |
