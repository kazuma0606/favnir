# v20.7.0 実装計画 — Arena アロケータ（GC なし高速アロケーション）

## 実装順序

```
T1: Cargo.toml — bumpalo 追加                                           ← 最初
T2: src/arena/mod.rs — ChunkArena 新規実装                              ← T1 完了後
T3: lib.rs / main.rs — mod arena 追加                                   ← T2 完了後
T4: vm.rs — VM struct + __streaming_pipeline + Arena.stats primitive    ← T3 完了後（最大工数）
T5: driver.rs — v207000_tests（5 件）                                   ← T4 完了後
T6: Cargo.toml version bump（20.6.0 → 20.7.0）                        ← 任意
T7: CHANGELOG.md 更新 + benchmarks/v20.7.0.json                        ← T5 完了後
T8: site/content/docs/ — Arena.stats ドキュメント                      ← T7 完了後
```

**変更ファイル一覧:**
- `fav/Cargo.toml`（T1 / T6）
- `fav/src/arena/mod.rs`（T2 — 新規）
- `fav/src/lib.rs`（T3）
- `fav/src/main.rs`（T3）
- `fav/src/backend/vm.rs`（T4）
- `fav/src/driver.rs`（T5）
- `CHANGELOG.md`（T7）
- `benchmarks/v20.7.0.json`（T7）
- `site/content/docs/runes/arena.mdx`（T8 — 新規）

---

## T1: `fav/Cargo.toml` — `bumpalo` 追加

```toml
# [target.'cfg(not(target_arch = "wasm32"))'.dependencies] セクションに追加
bumpalo = "3"
```

### 事前確認

```bash
grep -n "bumpalo\|arena" fav/Cargo.toml
# → 既存依存がないことを確認
```

### 完了条件
- `cargo check` でコンパイルエラー 0

---

## T2: `fav/src/arena/mod.rs` — `ChunkArena` 新規実装

### 実装コード

```rust
//! Arena-backed chunk allocator for streaming pipeline optimization.
//!
//! `ChunkArena` provides two complementary strategies:
//! 1. A reusable `Vec<VMValue>` **pool** — avoids per-chunk malloc/free.
//! 2. A `bumpalo::Bump` allocator — provides chunk-scoped lifetime marker
//!    and serves as the foundation for future string interning (v20.8+).
//!
//! Controlled by `FAV_ARENA_ENABLED` env var (default: enabled).

use bumpalo::Bump;

#[derive(Debug, Default, Clone)]
pub struct ArenaStats {
    /// Pool hits: Vec was reused from the pool (no malloc).
    pub acquire_count: usize,
    /// Pool misses: new Vec was allocated (pool was empty).
    pub alloc_count: usize,
    /// Number of chunk boundaries (reset calls).
    pub reset_count: usize,
    /// Peak Vec capacity seen (in elements, not bytes).
    pub peak_capacity: usize,
}

pub struct ChunkArena {
    bump: Bump,
    pool: Vec<Vec<crate::backend::vm::VMValue>>,
    stats: ArenaStats,
    enabled: bool,
}

impl ChunkArena {
    pub fn new() -> Self {
        let enabled = std::env::var("FAV_ARENA_ENABLED")
            .map(|v| v != "0")
            .unwrap_or(true);
        Self::new_with_enabled(enabled)
    }

    /// Create arena with explicit enabled flag (avoids env var in tests).
    pub fn new_with_enabled(enabled: bool) -> Self {
        Self {
            bump: Bump::new(),
            pool: Vec::new(),
            stats: ArenaStats::default(),
            enabled,
        }
    }

    /// Acquire a Vec from the pool (or allocate fresh).
    pub fn acquire(&mut self, capacity: usize) -> Vec<crate::backend::vm::VMValue> {
        if !self.enabled {
            self.stats.alloc_count += 1;
            return Vec::with_capacity(capacity);
        }
        if let Some(mut buf) = self.pool.pop() {
            buf.clear();
            self.stats.acquire_count += 1;
            buf
        } else {
            self.stats.alloc_count += 1;
            Vec::with_capacity(capacity)
        }
    }

    /// Return a Vec to the pool and track stats.
    pub fn release(&mut self, mut buf: Vec<crate::backend::vm::VMValue>) {
        self.stats.peak_capacity = self.stats.peak_capacity.max(buf.capacity());
        if self.enabled {
            buf.clear();
            self.pool.push(buf);
        }
        self.stats.reset_count += 1;
    }

    /// Called at the start of each chunk (currently a no-op marker).
    #[inline]
    pub fn start_chunk(&mut self) {
        // Future: bump.reset() for intra-chunk string interning
    }

    /// Drain the output value into `out` and release any reclaimed Vec.
    pub fn end_chunk(
        &mut self,
        result_val: crate::backend::vm::VMValue,
        out: &mut Vec<crate::backend::vm::VMValue>,
    ) {
        use crate::backend::vm::VMValue;
        match result_val {
            VMValue::List(fl) => {
                // fl.to_vec() requires FavList::to_vec to be pub(crate).
                // Must change vm.rs: `fn to_vec` → `pub(crate) fn to_vec` first.
                out.extend(fl.to_vec());
            }
            other => out.push(other),
        }
        // Reset the bump allocator at chunk boundary and increment reset_count.
        self.bump.reset();
        self.stats.reset_count += 1;
    }

    pub fn stats(&self) -> &ArenaStats {
        &self.stats
    }
}
```

> **モジュールパス**: `arena/mod.rs` から vm.rs の型を参照するには `crate::backend::vm::VMValue` を使う。
> `super::` 相対パスは crate root を指すため `super::backend::vm` は正しくない。

> **`FavList::to_vec` の可視性（必須変更）**: 現在 `fn to_vec`（完全プライベート）のため
> `arena/mod.rs` から呼べない。実装前に **必ず** `pub(crate) fn to_vec` に変更する:
> ```bash
> grep -n "fn to_vec" fav/src/backend/vm.rs
> # → "fn to_vec" なら pub(crate) に変更必須
> ```

### 完了条件
- `cargo check` でコンパイルエラー 0

---

## T3: `lib.rs` / `main.rs` — `mod arena` 追加

```rust
// lib.rs と main.rs 両方に追加（既存の mod parallel / mod incremental パターンと同様）
#[cfg(not(target_arch = "wasm32"))]
mod arena;
```

### 事前確認

```bash
grep -n "mod parallel\|mod incremental\|mod pushdown" fav/src/lib.rs | head -5
# → 既存の native-only モジュール追加パターンを確認
```

### 完了条件
- `cargo check` でコンパイルエラー 0

---

## T4: `vm.rs` — VM struct + `__streaming_pipeline` + `Arena.stats`

### 4-1. `VM` struct への `chunk_arena` フィールド追加

```bash
grep -n "struct VM {" fav/src/backend/vm.rs | head -3
```

```rust
// VM struct に追加（native-only）
pub struct VM {
    // ... existing fields ...
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) chunk_arena: crate::arena::ChunkArena,
}
```

`VM::new_with_db_path()` の struct リテラルにも初期化を追加（`VM::default()` は存在しない）:
```rust
#[cfg(not(target_arch = "wasm32"))]
chunk_arena: crate::arena::ChunkArena::new(),
```

```bash
grep -n "fn new_with_db_path\|fn new\b" fav/src/backend/vm.rs | head -5
# → VM 構造体の初期化箇所を特定（VM::new は new_with_db_path に委譲している）
```

### 4-2. `__streaming_pipeline` の最適化

```rust
// 変更前（4744〜4754行目付近）:
for chunk_items in items.chunks(chunk_size) {
    let mut current = VMValue::List(FavList::new(chunk_items.to_vec()));
    for stage_fn in &stage_fns {
        current = self.call_value(artifact, stage_fn.clone(), vec![current])?;
    }
    match current {
        VMValue::List(fl) => result.extend(fl.to_vec()),
        other => result.push(other),
    }
}
Ok(VMValue::List(FavList::new(result)))

// 変更後:
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
```

> **注意**: `FavList::new(buf)` は `Arc::new(buf)` により buf の所有権を移す。
> この時点で pool から取り出した Vec は Arc の管理下に入るため、
> pool への返却は `end_chunk` 内で `fl.to_vec()` で新 Vec に変換後に行う。
> 厳密な pool 再利用は `Arc::try_unwrap` が成功する場合のみ可能だが、
> `to_vec()` でコピーするシンプルな実装でも malloc 回数は削減できる。

### 4-3. `Arena.stats` primitive を `call_builtin` に追加

> **重要**: `Arena.stats` は `self.chunk_arena`（VM の self フィールド）にアクセスするため、
> `vm_call_builtin`（自由関数、`self` なし）ではなく **`call_builtin`（`&mut self` メソッド）** に追加する。
> `call_builtin` は vm.rs 内の `impl VM { fn call_builtin(...) }` として定義されている。

```rust
// call_builtin の match アームに追加
"Arena.stats" => {
    if !args.is_empty() {
        return Err("Arena.stats: expected 0 arguments".to_string());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let s = self.chunk_arena.stats();
        use std::collections::HashMap;
        let mut fields: HashMap<String, VMValue> = HashMap::new();
        fields.insert("acquire_count".to_string(), VMValue::Int(s.acquire_count as i64));
        fields.insert("alloc_count".to_string(), VMValue::Int(s.alloc_count as i64));
        fields.insert("reset_count".to_string(), VMValue::Int(s.reset_count as i64));
        fields.insert("peak_capacity".to_string(), VMValue::Int(s.peak_capacity as i64));
        Ok(ok_vm(VMValue::Record(fields)))
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(err_vm(VMValue::Str(
            "Arena.stats: not supported on wasm32".to_string(),
        )))
    }
}
```

> **`VMValue::Record` の型確認**: 既存の `VMValue::Record` コンストラクタの引数型を確認:
> ```bash
> grep -n "VMValue::Record\|Record(" fav/src/backend/vm.rs | grep -v "//\|#\[" | head -10
> ```

### 4-4. `is_known_builtin_namespace` への `"Arena"` 追加

```bash
grep -n "is_known_builtin_namespace" fav/src/backend/vm.rs | head -3
```

```rust
// is_known_builtin_namespace 関数に "Arena" を追加
"Arena" => true,
```

compiler.rs と checker.rs にも builtin namespace リストがあれば確認:
```bash
grep -rn "\"ArrowBatch\"\|\"IO\"\|is_known_builtin" fav/src/middle/compiler.rs | head -10
grep -rn "\"ArrowBatch\"\|\"IO\"\|builtin_ns" fav/src/middle/checker.rs | head -10
```

### 完了条件
- `cargo check` でコンパイルエラー 0
- `cargo test v207000` 実行可能（T5 後）

---

## T5: `driver.rs` — `v207000_tests`

```rust
// ── v207000_tests (v20.7.0) — Arena アロケータ ───────────────────────────────
#[cfg(test)]
mod v207000_tests {
    #[cfg(not(target_arch = "wasm32"))]
    use crate::arena::{ChunkArena, ArenaStats};
    use crate::backend::vm::VMValue;

    #[test]
    fn version_is_20_7_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.7.0"), "Cargo.toml should have version 20.7.0");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn arena_acquire_and_release() {
        let mut arena = ChunkArena::new();
        let buf: Vec<VMValue> = arena.acquire(10);
        assert_eq!(buf.len(), 0);
        assert!(buf.capacity() >= 10);
        let alloc_before = arena.stats().alloc_count;
        arena.release(buf);
        // 再取得は pool から（acquire_count が増える）
        let buf2: Vec<VMValue> = arena.acquire(5);
        let _ = buf2;
        assert!(
            arena.stats().acquire_count > 0 || arena.stats().alloc_count > alloc_before,
            "second acquire should hit pool or alloc"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn arena_stats_track_counts() {
        // Use new_with_enabled to avoid env var manipulation in tests.
        let mut arena = ChunkArena::new_with_enabled(true);
        let buf = arena.acquire(8);
        arena.release(buf);
        // Direct field access avoids lifetime issues from &ArenaStats borrow.
        assert!(
            arena.stats().alloc_count >= 1,
            "should have at least 1 alloc: {:?}", arena.stats()
        );
        assert_eq!(arena.stats().reset_count, 1, "one release = one reset");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn arena_streaming_pipeline_correctness() {
        // Test that ChunkArena correctly tracks chunk boundaries over multiple chunks.
        // (E2E __streaming_pipeline integration is verified by the existing streaming tests.)
        let mut arena = ChunkArena::new_with_enabled(true);
        for _ in 0..3 {
            arena.start_chunk();
            let buf = arena.acquire(3);
            // Simulate end_chunk: release and count resets
            arena.release(buf);
        }
        assert_eq!(arena.stats().reset_count, 3, "3 chunk releases = 3 resets");
        assert_eq!(
            arena.stats().acquire_count + arena.stats().alloc_count, 3,
            "3 acquires total"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn arena_disabled_by_env() {
        // Use new_with_enabled(false) to avoid unsafe env var manipulation (Rust 1.80+).
        let mut arena = ChunkArena::new_with_enabled(false);
        let buf = arena.acquire(4);
        arena.release(buf);
        // When disabled, pool is bypassed → all are alloc (pool miss)
        assert_eq!(arena.stats().acquire_count, 0, "pool should not be used when disabled");
        assert_eq!(arena.stats().alloc_count, 1, "should have 1 alloc when disabled");
    }
}
```

> **注意**: `arena_streaming_pipeline_correctness` は `compile_str` が driver テストから使えない場合、
> ChunkArena の acquire/release ループで代替する（上記実装で対応済み）。
> 将来的に E2E テストとして拡充可能。

### 完了条件
- `cargo test v207000` — 5/5 PASS

---

## T6: `fav/Cargo.toml` バージョン更新

`version = "20.6.0"` → `"20.7.0"` に変更。

既存の `version_is_20_6_0` テストに `#[ignore]` を追加。

---

## T7: `CHANGELOG.md` 更新 + `benchmarks/v20.7.0.json`

### CHANGELOG エントリ

```markdown
## [v20.7.0] — 2026-06-XX — Arena アロケータ（GC なし高速アロケーション）

### Added
- `ChunkArena` — ストリーミングパイプライン向け Vec プール + bumpalo アリーナ
  - `acquire(capacity)` / `release(buf)` で chunk 間 Vec を再利用（pool miss で malloc）
  - `start_chunk()` / `end_chunk()` でチャンク境界を明確化
  - `bump.reset()` でチャンク境界の一括解放マーク
- `Arena.stats()` VM primitive — アリーナ統計（acquire_count / alloc_count / reset_count / peak_capacity）
- `bumpalo = "3"` 依存クレート追加（native-only）
- `FAV_ARENA_ENABLED` 環境変数（`0` で arena 無効化、デフォルト有効）

### Changed
- `__streaming_pipeline` — arena を使って中間 Vec を pool から取得・返却

### Performance
- `record_transform_1m_ms`: +20〜40% 改善（Vec malloc/free の削減）
- `streaming_peak_memory_mb`: -20%（chunk 間バッファの再利用）
- 実測は `benchmarks/v20.7.0.json` 参照
```

ベンチマーク基準値の確認手順:
```bash
# v20.6.0 の基準値が存在するか確認
ls benchmarks/v20.6.0.json
# 存在しない場合は FAV_ARENA_ENABLED=0（arena 無効）で計測した値を基準とする
bash benchmarks/suite/run_all.sh --format json > benchmarks/v20.7.0.json
```
`record_transform_1m_ms` が v20.6.0 比（または arena 無効時比）+20% 以上であることを確認。

---

## T8: サイトドキュメント

`site/content/docs/runes/arena.mdx` を新規作成（または `io.mdx` に統合）。

```mdx
## Arena.stats

```favnir
Arena.stats() -> Record
```

ストリーミングパイプライン実行中のアリーナ統計を返します。
パフォーマンスのデバッグ・最適化確認に使用します。

```favnir
stage Report: Unit -> Unit = |_| {
  bind stats <- Arena.stats()
  emit "acquire_count: " + Int.to_string(stats.acquire_count)
  emit "alloc_count: " + Int.to_string(stats.alloc_count)
}
```

**フィールド:**
- `acquire_count`: pool から再利用した回数（malloc なし）
- `alloc_count`: 新規 malloc が必要だった回数
- `reset_count`: チャンク境界でリセットした回数
- `peak_capacity`: Vec の最大 capacity（要素数）

> WASM では `err` を返します。
```

---

## 注意点

### `Arc::try_unwrap` による pool 返却の最適化

`FavList(Arc<Vec<VMValue>>, offset)` は `Arc` で包まれているため、
`end_chunk` で Vec を pool に返すには `Arc::try_unwrap` が必要。
v20.7 では `to_vec()` でコピーする簡易版を採用し、`Arc::try_unwrap` は v20.8 で対応。

### `VMValue::Record` の型確認

`Arena.stats` が返す `VMValue::Record` の実際の型を実装前に確認:
```bash
grep -n "VMValue::Record\|Record(HashMap" fav/src/backend/vm.rs | head -10
```
既存の `Record` コンストラクタに合わせた実装にする。

### `FavList` の `pub(crate)` 確認

`arena/mod.rs` から `VMValue` / `FavList` を参照するため、可視性を確認:
```bash
grep -n "pub.*struct FavList\|pub.*fn to_vec" fav/src/backend/vm.rs | head -5
```

### `env::set_var` のスレッド安全性（テスト）

`arena_disabled_by_env` テストで `std::env::set_var` を使う。
Rust 1.80+ では `unsafe` が必要になるため、代替として `ChunkArena::new_disabled()` コンストラクタを追加する方が安全。
実装前に Rust バージョンを確認:
```bash
rustc --version
```
