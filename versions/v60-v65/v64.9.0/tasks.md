# v64.9.0 タスクリスト

Status: COMPLETE
Version: 64.9.0
Base tests: 3447
Target tests: 3449

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3447 tests passed, 0 failed を確認
- [x] `driver.rs` に `v64900_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64800_tests` が存在することを確認（`v64900_tests` の挿入位置）
- [x] `site/content/docs/performance/performance1-overview.mdx` が存在することを確認
- [x] `performance1-overview.mdx` に `"Performance 1.0 Overview"` / `"Quick Start"` / `"Performance Certification Checklist"` / `"Benchmark Results"` が含まれることを確認
- [x] CI 上で clippy クリーンであることを確認（CI ログまたは `cargo clippy 2>&1 | grep error` で確認、0 errors）

---

## T1: `driver.rs` — `v64900_tests` 追加

- [x] `// -- v64800_tests` コメント行の直前に `v64900_tests` を挿入
  - [x] `use super::*;` を含む
  - [x] `scale_all_v64_features_stable`: `cmd_build_ci` / `cmd_profile_flamegraph_aot` / `cmd_build_wasm` を呼び、各エラープレフィックスで始まらないことを確認
  - [x] `performance1_overview_doc_complete`: `include_str!` で MDX を読み込み、4 セクション（`"Performance 1.0 Overview"` / `"Quick Start"` / `"Performance Certification Checklist"` / `"Benchmark Results"`）の存在を確認
- [x] `cargo build` でエラーなし

---

## T2: ビルド・テスト

- [x] `cargo test --bin fav v64900_tests` で 2 件 PASS
  - [x] `scale_all_v64_features_stable` PASS
  - [x] `performance1_overview_doc_complete` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3449 tests passed, 0 failed を確認

---

## T3: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.9.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.9.0 セクションに実績追記（3449 tests）
- [x] `versions/current.md` の「進行中」を v64.9.0（3449 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）

## コードレビュー対応（code-reviewer 指摘）

- [MED] `cmd_build_wasm` のアサーションに陽性確認なし → `wasm_result.contains("Compiling (target: wasm32)")` を追加
- [MED] `current.md` 「次に切る版」が v64.9.0 のまま → v65.0.0 に更新
- [LOW] `cmd_profile_flamegraph_aot` の陽性確認なし → `aot_result.contains("Generated:")` を追加
- [LOW] `performance1_overview_doc_complete` が v64800_tests と部分重複 → 設計上の問題なし、変更不要

## コードレビュー対応（spec-reviewer 指摘）

- [MED] spec.md に Cranelift 環境依存リスク記述がなかった → 技術ノートに注記追記
- [MED] アサーション `"Performance 1.0"` と実ファイルヘッダのずれ → `"Performance 1.0 Overview"` に変更（spec/plan/tasks）
- [MED] lint/clippy 除外根拠不足 → spec.md 非スコープに「CI 代替」根拠を詳述、tasks.md T0 に確認項目追加
