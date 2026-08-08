# v63.2.0 タスクリスト

Status: COMPLETE
Version: 63.2.0
Base tests: 3408
Target tests: 3410

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3408 tests passed, 0 failed を確認
- [x] `fav/src/cache.rs` が存在し `IncrementalCache` / `stage_hash` が実装されていることを確認
- [x] `driver.rs` に `v63100_tests` が存在することを確認（挿入位置確認）
- [x] `driver.rs` に `cmd_incremental_cache_status` が存在することを確認

---

## T1: `driver.rs` — `cmd_run_with_cache` 追加

- [x] `cmd_incremental_cache_status` の直後（`cmd_build_aot_validate` の直前）に追加:
  ```rust
  pub fn cmd_run_with_cache(src: &str, cache_dir: &str) -> String { ... }
  ```
- [x] `cargo build` でエラーなし

---

## T2: `driver.rs` — `v63200_tests` 追加

- [x] `v63100_tests` の直前（ファイル先頭方向）に以下を挿入:
  （注意: モジュールトップの `use crate::cache::{IncrementalCache, stage_hash};` と
  `use tempfile::TempDir;` は `watch_incremental_recompile` で使用。
  `watch_notify_integration` は関数内で `use notify::{...}` と `use std::sync::mpsc;` を直接 import）
  ```rust
  // -- v63200_tests (v63.2.0) -- fav watch 改善・IncrementalCache 統合 --
  #[cfg(test)]
  mod v63200_tests { ... }
  ```
- [x] `cargo build` でエラーなし（テスト挿入後のインクリメンタル確認）

---

## T3: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0（全ステップ完了後の最終確認）
- [x] `cargo test v63200` で 2 件 PASS
  - `watch_incremental_recompile` PASS
  - `watch_notify_integration` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3410 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v63.2.0 エントリを追加
- [x] `versions/roadmap/roadmap-v63.1-v64.0.md` v63.2.0 セクションに実績追記
- [x] `versions/current.md` の「進行中」を v63.2.0（3410 tests）に更新
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応（spec-reviewer）

- [MED-1] ロードマップとのスコープ縮小根拠の説明不足 → spec.md の「既存実装の確認」に `roadmap-v63.1-v64.0.md` の `**既存機能の扱い**` 注記と整合する旨を明記
- [MED-2] 仕様コメントの戻り値 `"cache hit: <file> ..."` と実装不一致 → `"cache hit: (skipped recompile)"` に統一
- [MED-3] tasks.md T2 の import 注意コメントが `watch_notify_integration` 側で誤解を招く → 用途別に分けて明記
- [LOW] plan.md Step 4 の自己参照を「最終ステップとして tasks.md を更新」と明示

## コードレビュー指摘対応（code-reviewer）

- [HIGH] `watch_notify_integration` で `dir` が `w` より先に drop される（OS ウォッチャーが削除済みパスを監視したまま残る）→ 明示的 `drop(w)` を `dir` drop の前に追加
- [MED] `cmd_run_with_cache` がパースのみキャッシュし型チェックをキャッシュしない設計が未文書化 → コメントと `TODO(v63.x)` を追加
- [LOW] 関数内 `use crate::cache::...` がプロジェクトスタイルと不一致 → `crate::cache::stage_hash(...)` / `crate::cache::IncrementalCache::new(...)` のフルパス記法に統一

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3410 passed, 0 failed
- 主要実装: `driver.rs`（`cmd_run_with_cache` + `v63200_tests`）
- 完了日: 2026-08-02
