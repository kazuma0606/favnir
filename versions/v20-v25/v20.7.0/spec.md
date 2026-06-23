# v20.7.0 Spec — Arena アロケータ（GC なし高速アロケーション）

## 概要

v20.7.0 はストリーミングパイプライン実行中のアロケーションコストを根本的に削減する。

**問題**: `__streaming_pipeline` は chunk ごとに `Vec<VMValue>` を `alloc → use → free` しており、
chunk が多いほど malloc/free のオーバーヘッドが累積する。

**解決**: `ChunkArena`（`bumpalo::Bump` + Vec プール）を導入し、
1 chunk = 1 arena のライフタイムで中間 Vec を再利用する。
chunk 完了時に `bump.reset()` + Vec を pool に戻すことで、一括解放を実現する。

新 VM primitive `Arena.stats() -> Record` でアリーナ統計をユーザーが観測できるようにする。

**テーマ**: Runtime Excellence シリーズ第7弾 — GC なし高速アロケーション

---

## 動機と期待効果

### 現状の問題（`__streaming_pipeline`）

```rust
// vm.rs 現状（4744〜4754行目付近）
for chunk_items in items.chunks(chunk_size) {
    // ❌ 毎 chunk: Vec<VMValue> を新規 malloc
    let mut current = VMValue::List(FavList::new(chunk_items.to_vec()));
    for stage_fn in &stage_fns {
        current = self.call_value(artifact, stage_fn.clone(), vec![current])?;
    }
    match current {
        VMValue::List(fl) => result.extend(fl.to_vec()),  // ❌ また alloc + extend
        other => result.push(other),
    }
}
```

1000 行 × 1000 chunk の場合:
- `chunk_items.to_vec()` → 1000 回 malloc
- `fl.to_vec()` → 1000 回 malloc
- 計 2000 回の short-lived Vec alloc/free

### 最適化後（ChunkArena）

```rust
for chunk_items in items.chunks(chunk_size) {
    self.chunk_arena.start_chunk();
    // ✅ pool から Vec を取得（malloc なし）
    let mut buf = self.chunk_arena.acquire(chunk_items.len());
    buf.extend_from_slice(chunk_items);
    let mut current = VMValue::List(FavList::new(buf));
    for stage_fn in &stage_fns {
        current = self.call_value(artifact, stage_fn.clone(), vec![current])?;
    }
    // ✅ Vec を pool に返却（free なし）
    self.chunk_arena.end_chunk(current, &mut result);
}
```

### 期待改善（v20.6.0 比）

| ベンチマーク | v20.6.0 基準 | 期待改善 |
|---|---|---|
| `record_transform_1m_ms` | ~140ms（1M 行変換） | **+20〜40%** |
| `streaming_peak_memory_mb` | ~280MB（定常） | **-20%** |
| `chunk_alloc_overhead_ms` | ~45ms（chunk overhead） | **+2〜3x** |

---

## アーキテクチャ

### `ChunkArena` 設計

```rust
// src/arena/mod.rs

pub struct ArenaStats {
    pub acquire_count: usize,   // pool から取得した回数（pool hit）
    pub alloc_count: usize,     // 新規 malloc が必要だった回数（pool miss）
    pub reset_count: usize,     // chunk 完了で reset した回数
    pub peak_capacity: usize,   // Vec の最大 capacity（bytes）
}

pub struct ChunkArena {
    /// bumpalo bump allocator — 将来の文字列インターン用（現在は stats 追跡のみ）
    bump: bumpalo::Bump,
    /// Reusable Vec<VMValue> pool — chunk 間で再利用
    pool: Vec<Vec<VMValue>>,
    /// Allocation statistics
    stats: ArenaStats,
    /// Arena 有効フラグ（FAV_ARENA_ENABLED=0 で無効化）
    enabled: bool,
}

impl ChunkArena {
    pub fn new() -> Self { ... }
    pub fn acquire(&mut self, capacity: usize) -> Vec<VMValue> { ... }
    pub fn release(&mut self, buf: Vec<VMValue>) { ... }
    pub fn start_chunk(&mut self) { ... }
    pub fn end_chunk(&mut self, result_val: VMValue, out: &mut Vec<VMValue>) { ... }
    pub fn reset_bump(&mut self) { self.bump.reset(); }
    pub fn stats(&self) -> &ArenaStats { ... }
}
```

### `VM` への統合

```rust
// vm.rs: VM struct に追加
pub struct VM {
    // ... existing fields ...
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) chunk_arena: ChunkArena,
}
```

### `__streaming_pipeline` の変更

```rust
"__streaming_pipeline" => {
    // ... args parsing unchanged ...

    let mut result: Vec<VMValue> = Vec::new();
    for chunk_items in items.chunks(chunk_size) {
        #[cfg(not(target_arch = "wasm32"))]
        self.chunk_arena.start_chunk();

        #[cfg(not(target_arch = "wasm32"))]
        let mut buf = self.chunk_arena.acquire(chunk_items.len());
        #[cfg(target_arch = "wasm32")]
        let mut buf = Vec::with_capacity(chunk_items.len());

        buf.extend_from_slice(chunk_items);
        let mut current = VMValue::List(FavList::new(buf));

        for stage_fn in &stage_fns {
            current = self.call_value(artifact, stage_fn.clone(), vec![current])?;
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.chunk_arena.end_chunk(current, &mut result);
        #[cfg(target_arch = "wasm32")]
        match current {
            VMValue::List(fl) => result.extend(fl.to_vec()),
            other => result.push(other),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    self.chunk_arena.reset_bump();

    Ok(VMValue::List(FavList::new(result)))
}
```

### `Arena.stats` primitive

`Arena.stats` は `self.chunk_arena`（VM の self フィールド）にアクセスするため、
`vm_call_builtin`（自由関数）ではなく **`call_builtin`（`&mut self` メソッド）** に追加する。

```rust
// call_builtin メソッド内に追加（vm_call_builtin ではない）
"Arena.stats" => {
    if !args.is_empty() {
        return Err(self.error(artifact, "Arena.stats: expected 0 arguments"));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let s = self.chunk_arena.stats();
        // Returns Record { acquire_count: Int, alloc_count: Int,
        //                  reset_count: Int, peak_capacity: Int }
        Ok(ok_vm(VMValue::Record(arena_stats_to_record(s))))
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(err_vm(VMValue::Str("Arena.stats: not supported on wasm32".to_string())))
    }
}
```

### `FAV_ARENA_ENABLED` 環境変数

```bash
FAV_ARENA_ENABLED=0 fav run pipeline.fav   # arena 無効（デバッグ用）
FAV_ARENA_ENABLED=1 fav run pipeline.fav   # arena 有効（デフォルト）
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `bumpalo = "3"` を native-only deps に追加 / version `20.6.0` → `20.7.0` |
| `fav/src/arena/mod.rs` | `ChunkArena` / `ArenaStats` 新規実装 |
| `fav/src/lib.rs` | `#[cfg(not(wasm32))] mod arena;` 追加 |
| `fav/src/main.rs` | `#[cfg(not(wasm32))] mod arena;` 追加 |
| `fav/src/backend/vm.rs` | `VM` struct に `chunk_arena` フィールド追加 / `__streaming_pipeline` 最適化 / `"Arena.stats"` primitive 追加 / `is_known_builtin_namespace` に `"Arena"` 追加 |
| `fav/src/driver.rs` | `v207000_tests`（5 件） |
| `CHANGELOG.md` | v20.7.0 エントリ追加 |
| `benchmarks/v20.7.0.json` | 実測ベンチマーク結果 |
| `site/content/docs/runes/arena.mdx` | `Arena.stats` ドキュメント追加 |

---

## テスト（v207000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_7_0` | `Cargo.toml` に `"20.7.0"` が含まれる |
| `arena_acquire_and_release` | `ChunkArena::acquire` → `release` で pool に返却され再利用される |
| `arena_stats_track_counts` | acquire/release 後に `stats.acquire_count` / `stats.reset_count` が正しく更新される |
| `arena_streaming_pipeline_correctness` | 3 回 acquire/release で `reset_count == 3`（chunk 境界追跡の確認） |
| `arena_disabled_by_env` | `ChunkArena::new_with_enabled(false)` 時に pool が使われず `acquire_count == 0` |

---

## 完了条件

- [ ] `ChunkArena::acquire` / `release` が Vec を pool から再利用する
- [ ] `__streaming_pipeline` が arena を使って中間 Vec を再利用する
- [ ] `Arena.stats()` primitive が正しい統計を返す
- [ ] `ChunkArena::new_with_enabled(false)` で arena を無効化できる（`FAV_ARENA_ENABLED=0` 環境変数でも同様）
- [ ] `is_known_builtin_namespace` に `"Arena"` が追加されている（vm.rs / compiler.rs / checker.rs）
- [ ] WASM ビルドが `cfg(not(wasm32))` ガードでコンパイルを通る
- [ ] `cargo test v207000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `fav/Cargo.toml` version が `20.7.0`
- [ ] `CHANGELOG.md` に v20.7.0 エントリが追加されている
- [ ] `benchmarks/v20.7.0.json` が生成されている
- [ ] `record_transform_1m_ms` が v20.6.0 比 +20% 以上改善（ストリーミングパイプライン）

---

## 技術ノート

### `bumpalo::Bump` の役割（v20.7 時点）

v20.7 では `Bump` を直接 VMValue アロケーションには使わない（`VMValue` の各バリアントが `Arc`/`String` 等のデストラクタを持つため、bumpalo の制約に反する）。v20.7 の `Bump` の役割:
1. `bump.reset()` でチャンク境界を明確にマーク（stats tracking）
2. 将来（v20.8+）の文字列バイトインターンに備えた基盤

### Vec プールが有効な理由

`FavList::new(buf)` は `Arc::new(buf)` を呼ぶため、buf の所有権が移る。
chunk 処理後に `Arc` が drop されると `Vec<VMValue>` は解放される。
プールでは `Arc` の drop タイミングを制御して、同一の `Vec` メモリを再利用する。

ただし `FavList::new` に `Arc::new` が含まれる現実装では、
プールが最も効果を発揮するのは「acquired Vec → chunk 処理 → FavList に渡す前の段階」である。
具体的には `buf` を構築して `chunk_items.to_vec()` の代替として使う部分が最大の削減ポイント。

### `FavList::to_vec` の可視性（必須変更）

`FavList::to_vec` は現在 `fn to_vec`（完全プライベート）。
`arena/mod.rs` の `end_chunk` から呼ぶために **`pub(crate) fn to_vec` に変更必須**（任意ではない）。
実装前に `grep -n "fn to_vec" fav/src/backend/vm.rs` で確認し、必ず `pub(crate)` に変更する。

### `is_known_builtin_namespace` への `"Arena"` 追加

`vm.rs` の `is_known_builtin_namespace` に `"Arena"` を追加する必要がある。
`compiler.rs` と `checker.rs` にも builtin namespace リストがあれば同様に追加（必須確認）。

### `bumpalo` の v20.7 における役割

v20.7 では `Bump` を直接 VMValue アロケーションには使わない（`VMValue` の各バリアントが
`Arc`/`String` 等のデストラクタを持つため bumpalo の制約に反する）。
v20.7 の実質的な最適化は **Vec プール再利用**（malloc/free 削減）であり、
`bumpalo` は chunk 境界マーカー + v20.8 文字列インターンの基盤として導入する。
ロードマップの「一括解放」はこの Vec プール返却を指している。

### スコープ外（v20.8 以降）

- `bumpalo` による文字列バイトのアリーナアロケーション
- `FavList` の arena-backed 版（`Arc` を避けたゼロコピー実装）
- DB コネクションプール（v20.8 テーマ）
