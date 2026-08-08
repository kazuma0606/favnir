# v63.1.0 タスクリスト

Status: COMPLETE
Version: 63.1.0
Base tests: 3406
Target tests: 3408

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3406 tests passed, 0 failed を確認
  （ロードマップ記載 3396 より +10 — v62.8.0/v62.9.0/v63.0.0 の実績値）
- [x] `fav/src/cache.rs` が **存在しない** ことを確認
- [x] `driver.rs` に `v63000_tests` が存在することを確認（挿入位置確認）
- [x] `fav/Cargo.toml` の現行バージョンが `63.0.0` であることを確認

---

## T1: `fav/src/cache.rs` — `IncrementalCache` 新規作成

- [x] `fav/src/cache.rs` を新規作成:
  - `StageEntry { stage_name: String, source_hash: String, type_sig: String }` （`#[derive(serde::Serialize, serde::Deserialize)]`）
  - `IncrementalCache { root: PathBuf }` struct
  - `impl IncrementalCache`:
    - `pub fn new(root: &Path) -> Self`（`create_dir_all` + `.ok()` で失敗を握り潰す）
    - `pub fn is_hit(&self, stage_name: &str, source_hash: &str) -> bool`
    - `pub fn store(&self, stage_name: &str, source_hash: &str, type_sig: &str)`
    - `pub fn invalidate(&self, stage_name: &str)`
    - `fn load_entry(&self, stage_name: &str) -> Option<StageEntry>`
    - `fn entry_path(&self, stage_name: &str) -> PathBuf`（`{stage_name}.json`）
  - `pub fn stage_hash(src: &[u8]) -> String`（`sha2::Sha256` で SHA-256）
- [x] `cargo build` は T2 後に実施（lib.rs 登録前はスキップ）

---

## T2: `fav/src/lib.rs` + `fav/src/main.rs` — モジュール登録

- [x] `lib.rs`: `pub mod incremental;` の直後に `#[cfg(not(target_arch = "wasm32"))] pub mod cache;` を追加
- [x] `main.rs`: `mod incremental;` の直後に `mod cache;` を追加
  （driver.rs は main.rs の crate 内のため main.rs への追加が必要）
- [x] `cargo build` でエラーなし（T1 + T2 合わせて初めてビルド検証）

---

## T3: `driver.rs` — `cmd_incremental_cache_status` 追加

- [x] `cmd_build_aot_validate` の直前に以下を追加:
  ```rust
  pub fn cmd_incremental_cache_status(cache_dir: &str) -> String { ... }
  ```
- [x] `cargo build` でエラーなし

---

## T4: `driver.rs` — `v63100_tests` 追加

- [x] `v63000_tests` の直前（ファイル先頭方向）に以下を挿入:
  （注意: `use crate::cache::{IncrementalCache, stage_hash};` と `use tempfile::TempDir;` が必要）
  ```rust
  // -- v63100_tests (v63.1.0) -- 差分コンパイルキャッシュ --
  #[cfg(test)]
  mod v63100_tests { ... }
  ```
- [x] `cargo build` でエラーなし（テスト挿入後のインクリメンタル確認）

---

## T5: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test v63100` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3408 tests passed, 0 failed を確認

---

## T6: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.1.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.1.0 セクションに実績追記・テスト数を `3398` → `3408` に補正
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` テスト数推移表のベース行（v63.0.0）を `3396` → `3406` に補正（spec-reviewer 対応で実施済み）
- [x] `versions/roadmap/roadmap-v60.1-v65.0.md` テスト数推移表の v63.1〜/v64.0 行を確認・更新（spec-reviewer 対応で実施済み）
- [x] `versions/current.md` の「進行中」を v63.1.0（3408 tests）に更新
- [x] `site/` MDX 追加は非スコープであることを確認（v63.2.0 以降）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応（spec-reviewer）

- [HIGH-1] `StageCache` vs `IncrementalCache` 命名不一致 → `IncrementalCache` に統一（spec/plan/tasks 修正）
- [HIGH-2] `cmd_run` 統合のスコープ乖離 → 非スコープ欄に「意図的な後送り」として明記
- [HIGH-3] ロードマップテスト数 +10 ずれ → roadmap-v63.1-v64.0.md 全バージョン補正
- [MED-1] `cmd_incremental_cache_status` テストなし → 非スコープ明記（v63.2.0 以降）
- [MED-2] plan.md Step 1 cargo build タイミング → Step 2 後に移動
- [MED-3] T4/T5 cargo build 役割曖昧 → 目的コメント明記
- [LOW-1] WASM ガード → lib.rs / main.rs を cfg ガード付きで追加
- [LOW-2] plan.md Step 6 粒度 → 5 項目に展開
- [LOW-3] site MDX 非スコープ未明記 → tasks.md T6 に追加
- 実装時発見: `driver.rs` は `main.rs` crate 内（`fav` bin crate）のため `lib.rs` だけでなく `main.rs` にも `mod cache;` が必要

## コードレビュー指摘対応（code-reviewer）

- [HIGH-1] `main.rs` の `mod cache;` に WASM ガード欠落 → `#[cfg(not(target_arch = "wasm32"))] mod cache;` に修正
- [HIGH-2] `entry_path` のパストラバーサルリスク → ステージ名を `[a-zA-Z0-9_-]` のみ許可するサニタイズ処理を追加
- [MED-1] `cmd_incremental_cache_status` の `read_dir` 結果がソート未実施 → `entries.sort()` 追加
- [MED-2] `IncrementalCache::new` の `create_dir_all` 失敗が無言 → `eprintln!` 警告に変更
- [MED-3] `v63100_tests` の `hash_v1 != hash_v2` 前提条件が未検証 → `assert_ne!(hash_v1, hash_v2, ...)` 追加
- [LOW-1] `stage_hash` のゼロパディング明示化 → `{:x}` → `{b:02x}` per-byte collect に変更
- [LOW-2] `main.rs` の `mod incremental;` も WASM ガードなし → 既存コードのため今バージョンでは対象外（将来対応）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3408 passed, 0 failed
- 主要実装: `fav/src/cache.rs`（`IncrementalCache` / `StageEntry` / `stage_hash`）/ `lib.rs` + `main.rs` モジュール登録 / `driver.rs`（`cmd_incremental_cache_status` + `v63100_tests`）
- 完了日: 2026-08-02
