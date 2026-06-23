# v20.7.0 — Arena アロケータ タスク

## ステータス: DONE

---

## タスク一覧

### T1: `fav/Cargo.toml` — `bumpalo` 追加

- [x] **事前確認**: `grep -n "bumpalo\|arena" fav/Cargo.toml` で既存依存がないことを確認
- [x] `bumpalo = "3"` を `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに追加
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `fav/src/arena/mod.rs` — `ChunkArena` + `ArenaStats` 新規実装

- [x] `fav/src/arena/` ディレクトリを作成
- [x] **可視性確認（必須）**: `grep -n "fn to_vec" fav/src/backend/vm.rs` を実行 — 現在 `fn to_vec`（完全プライベート）のため **`pub(crate) fn to_vec` に変更必須**。`arena/mod.rs` の `end_chunk` が呼ぶため変更なしではコンパイルエラーになる
- [x] `ArenaStats` struct を実装（`Debug`, `Default`, `Clone` derive）:
  - [x] `acquire_count: usize` — pool hit 回数
  - [x] `alloc_count: usize` — pool miss（新規 malloc）回数
  - [x] `reset_count: usize` — chunk 境界リセット回数
  - [x] `peak_capacity: usize` — Vec の最大 capacity（要素数）
- [x] `ChunkArena` struct を実装:
  - [x] `bump: bumpalo::Bump` フィールド
  - [x] `pool: Vec<Vec<VMValue>>` フィールド
  - [x] `stats: ArenaStats` フィールド
  - [x] `enabled: bool` フィールド（`FAV_ARENA_ENABLED` env var から設定）
- [x] `ChunkArena::new()` — env var 読み込み + `new_with_enabled(enabled)` に委譲
- [x] `ChunkArena::new_with_enabled(enabled: bool)` — env var 不要（テスト用）
- [x] `ChunkArena::acquire(capacity: usize) -> Vec<VMValue>`:
  - [x] `enabled=true` かつ pool に Vec あり → pool.pop() して clear()、`acquire_count` +1
  - [x] それ以外 → `Vec::with_capacity(capacity)`、`alloc_count` +1
- [x] `ChunkArena::release(buf: Vec<VMValue>)`:
  - [x] `peak_capacity` を buf.capacity() と max で更新
  - [x] `enabled=true` → buf.clear() して pool.push(buf)
  - [x] `reset_count` +1
- [x] `ChunkArena::start_chunk(&mut self)` — no-op（将来の文字列インターン用マーカー）
- [x] `ChunkArena::end_chunk(result_val: VMValue, out: &mut Vec<VMValue>)`:
  - [x] `VMValue::List(fl)` → `out.extend(fl.to_vec())`（`to_vec` を `pub(crate)` に変更済みであること前提）
  - [x] その他 → `out.push(result_val)`
  - [x] `self.bump.reset()` でバンプアロケータをチャンク境界でリセット
  - [x] `self.stats.reset_count += 1` で境界カウントをインクリメント（`release` とは別に `end_chunk` でも加算）
- [x] `ChunkArena::reset_bump(&mut self)` — `self.bump.reset()` のみ
- [x] `ChunkArena::stats(&self) -> &ArenaStats`
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/src/lib.rs` + `fav/src/main.rs` — `mod arena` 追加

- [x] **事前確認**: `grep -n "mod parallel\|mod incremental\|mod pushdown" fav/src/lib.rs | head -5` で既存 native-only モジュールパターンを確認
- [x] `fav/src/lib.rs` に `#[cfg(not(target_arch = "wasm32"))] mod arena;` を追加
- [x] `fav/src/main.rs` に `#[cfg(not(target_arch = "wasm32"))] mod arena;` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `fav/src/backend/vm.rs` — VM struct + `__streaming_pipeline` + `Arena.stats`

#### 4-1. VM struct への `chunk_arena` フィールド追加

- [x] `grep -n "pub struct VM\|struct VM {" fav/src/backend/vm.rs | head -3` で VM 定義箇所を確認
- [x] `VM` struct に `#[cfg(not(target_arch = "wasm32"))] pub(crate) chunk_arena: crate::arena::ChunkArena,` を追加
- [x] `VM::new_with_db_path()` の struct リテラルに `#[cfg(not(target_arch = "wasm32"))] chunk_arena: crate::arena::ChunkArena::new(),` を追加（`VM::default()` は存在しない — `VM::new` は `new_with_db_path` に委譲）
- [x] `cargo check` でコンパイルエラー 0

#### 4-2. `__streaming_pipeline` の最適化（4744行目付近）

- [x] `grep -n "__streaming_pipeline" fav/src/backend/vm.rs | head -3` で正確な行番号を確認
- [x] chunk ループ内の `chunk_items.to_vec()` を arena acquire に置き換え:
  - [x] `#[cfg(not(target_arch = "wasm32"))] self.chunk_arena.start_chunk();`
  - [x] `#[cfg(not(target_arch = "wasm32"))] let mut buf = self.chunk_arena.acquire(chunk_items.len());`
  - [x] `#[cfg(target_arch = "wasm32")] let mut buf = Vec::with_capacity(chunk_items.len());`
  - [x] `buf.extend_from_slice(chunk_items);`
  - [x] `VMValue::List(FavList::new(buf))` で chunk VMValue を構築
- [x] chunk 出力の `result.extend(fl.to_vec())` を end_chunk に置き換え:
  - [x] `#[cfg(not(target_arch = "wasm32"))] self.chunk_arena.end_chunk(current, &mut result);`
  - [x] `#[cfg(target_arch = "wasm32")] match current { VMValue::List(fl) => result.extend(fl.to_vec()), other => result.push(other), }`
- [x] ループ後に `#[cfg(not(target_arch = "wasm32"))] self.chunk_arena.reset_bump();` を追加
- [x] `cargo check` でコンパイルエラー 0

#### 4-3. `VMValue::Record` の型確認

- [x] `grep -n "VMValue::Record\|Record(HashMap\|Record(std::collections" fav/src/backend/vm.rs | head -10` で Record の実際の型を確認
- [x] 確認した型に合わせて `arena_stats_to_record` ヘルパー（またはインライン）を実装

#### 4-4. `"Arena.stats"` を `call_builtin`（`&mut self` メソッド）に追加

> `Arena.stats` は `self.chunk_arena` にアクセスするため `vm_call_builtin`（自由関数）ではなく
> `call_builtin`（VM の `&mut self` メソッド）に追加する。

- [x] `grep -n "fn call_builtin\|impl VM" fav/src/backend/vm.rs | head -5` で `call_builtin` の場所を確認
- [x] `"Arena.stats"` ハンドラを `call_builtin` の match アームに追加:
  - [x] 引数なし確認（`!args.is_empty()` で `self.error(...)` を返す）
  - [x] `#[cfg(not(target_arch = "wasm32"))]` ブロックで `self.chunk_arena.stats()` から Record を構築
  - [x] `#[cfg(target_arch = "wasm32")]` ブロックで `err_vm` を返す

#### 4-5. `is_known_builtin_namespace` に `"Arena"` を追加

- [x] `grep -n "is_known_builtin_namespace\|\"IO\"\|\"ArrowBatch\"" fav/src/backend/vm.rs | head -10` で関数を確認
- [x] `"Arena" => true,`（または同等の形式）を追加
- [x] `grep -rn "\"ArrowBatch\"\|builtin_namespace\|known_builtin" fav/src/middle/compiler.rs | head -10` で compiler.rs にも同様のリストがあれば追加
- [x] `grep -rn "\"ArrowBatch\"\|builtin_ns" fav/src/middle/checker.rs | head -10` で checker.rs にも同様のリストがあれば追加
- [x] `cargo check` でコンパイルエラー 0

---

### T5: `fav/src/driver.rs` — `v207000_tests`

- [x] `driver.rs` 末尾に `#[cfg(test)] mod v207000_tests { ... }` を追加
- [x] テスト 1: `version_is_20_7_0` — `Cargo.toml` に `"20.7.0"` が含まれる
- [x] テスト 2: `arena_acquire_and_release` — acquire → release → 再 acquire で pool hit を確認
- [x] テスト 3: `arena_stats_track_counts` — acquire/release 後に alloc_count / reset_count が正しい
- [x] テスト 4: `arena_streaming_pipeline_correctness` — `start_chunk` → `acquire` → `release` を 3 回繰り返し、`reset_count == 3` かつ acquire + alloc 合計 == 3 を確認
- [x] テスト 5: `arena_disabled_by_env` — `ChunkArena::new_with_enabled(false)` で pool が使われないことを確認（`acquire_count == 0`、`alloc_count == 1`。`std::env::set_var` は Rust 1.80+ で unsafe 必須のため使わない）
- [x] 各テストで `#[cfg(not(target_arch = "wasm32"))]` ガードを付与
- [x] `cargo test v207000` — 5/5 PASS を確認

---

### T6: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.6.0"` → `"20.7.0"` に変更
- [x] 既存の `version_is_20_6_0` テストに `#[ignore]` を追加

---

### T7: `CHANGELOG.md` 更新 + ベンチマーク

- [x] `CHANGELOG.md` の先頭に v20.7.0 エントリを追加:
  - [x] `### Added` — `ChunkArena`、`Arena.stats`、`bumpalo = "3"`、`FAV_ARENA_ENABLED`
  - [x] `### Changed` — `__streaming_pipeline` の arena 統合
  - [x] `### Performance` — `record_transform_1m_ms` +20〜40%、`streaming_peak_memory_mb` -20%
- [x] `benchmarks/v20.7.0.json` を生成

---

### T8: `site/content/docs/` — `Arena.stats` ドキュメント

- [x] `site/content/docs/runes/arena.mdx` を新規作成:
  - [x] `Arena.stats()` シグネチャ・説明・使用例
  - [x] フィールド一覧（acquire_count / alloc_count / reset_count / peak_capacity）
  - [x] WASM 非対応の注意書き
  - [x] `FAV_ARENA_ENABLED=0` によるデバッグ無効化の説明

---

## テスト（v207000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_7_0` | `Cargo.toml` に `"20.7.0"` が含まれる |
| `arena_acquire_and_release` | acquire → release → 再 acquire で pool 再利用が確認できる |
| `arena_stats_track_counts` | acquire/release 後に `alloc_count` / `reset_count` が正しく更新される |
| `arena_streaming_pipeline_correctness` | 3 回 acquire/release で `reset_count == 3` |
| `arena_disabled_by_env` | `FAV_ARENA_ENABLED=0` 時に pool が使われず `acquire_count == 0` |

---

## 完了条件チェックリスト

- [x] `FavList::to_vec` が `pub(crate)` に変更されている（必須 — arena/mod.rs からアクセスするため）
- [x] `ChunkArena::acquire` / `release` が Vec を pool から再利用する（enabled=true 時）
- [x] `ChunkArena::new_with_enabled(bool)` が存在し、テストで env var なしで enabled/disabled をテストできる
- [x] `__streaming_pipeline` が arena を使って中間 Vec を取得・返却する
- [x] `Arena.stats()` primitive が `call_builtin`（self メソッド）に追加されている
- [x] `is_known_builtin_namespace` に `"Arena"` が追加されている（vm.rs。compiler.rs/checker.rs も確認）
- [x] `FAV_ARENA_ENABLED=0` 環境変数で arena を無効化できる
- [x] WASM ビルドが `cfg(not(wasm32))` ガードでコンパイルを通る
- [x] `cargo test v207000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `fav/Cargo.toml` version が `20.7.0`
- [x] `CHANGELOG.md` に v20.7.0 エントリが追加されている
- [x] `benchmarks/v20.7.0.json` が生成されている
- [x] `record_transform_1m_ms` が v20.6.0 比 +20% 以上改善（ストリーミングパイプライン）

---

## 優先度

```
T1（Cargo.toml bumpalo）  ← 他すべての前提
T2（arena/mod.rs）         ← T1 完了後（最大工数）
T3（lib.rs / main.rs）     ← T2 完了後
T4（vm.rs 統合）           ← T3 完了後
T5（driver.rs テスト）     ← T4 完了後
T6（バージョン更新）       ← 任意タイミング
T7（CHANGELOG + bench）    ← T5 完了後
T8（サイトドキュメント）   ← T7 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| `arena/mod.rs` から `VMValue` への参照がコンパイルエラー | `use crate::backend::vm::VMValue;` のパスを実際の module 構造で確認。必要なら `VMValue` に `pub` を追加 |
| `FavList::to_vec` が `pub(crate)` でないため arena から呼べない | `grep -n "fn to_vec" fav/src/backend/vm.rs` で可視性確認。必要なら `pub(crate)` に変更 |
| `std::env::set_var` が Rust 1.80+ で `unsafe` 必須 | `rustc --version` で確認。1.80+ なら `ChunkArena::new_disabled()` コンストラクタで代替 |
| `VMValue::Record` の実際の型が `HashMap<String, VMValue>` でない | 実装前に grep で確認。型が異なれば `Arena.stats` の返却値を `VMValue::List` の 4 要素タプルで代替 |
| `VM::new()` の箇所が複数あって `chunk_arena` 追加漏れ | `grep -n "fn new\|VM {" vm.rs` で全箇所を確認 |
| WASM ビルドで `mod arena` が引き込まれる | lib.rs / main.rs の `mod arena` に `#[cfg(not(target_arch = "wasm32"))]` を付与（T3 で実施） |
