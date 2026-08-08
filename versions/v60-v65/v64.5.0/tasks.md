# v64.5.0 タスクリスト

Status: COMPLETE
Version: 64.5.0
Base tests: 3439
Target tests: 3441

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3439 tests passed, 0 failed を確認
- [x] `driver.rs` に `v64500_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64400_tests` が存在することを確認（`v64500_tests` の挿入位置）
- [x] `site/content/docs/runtime/benchmarks.mdx` が存在しないことを確認（新規作成）
- [x] `benchmarks/compare/run_comparison.sh` が存在しないことを確認（新規作成）
- [x] `include_str!` パス確認: `../../site/content/docs/runtime/benchmarks.mdx` → `C:\Users\yoshi\favnir\site\content\docs\runtime\benchmarks.mdx`
- [x] `include_str!` パス確認: `../../benchmarks/compare/run_comparison.sh` → `C:\Users\yoshi\favnir\benchmarks\compare\run_comparison.sh`

---

## T1: `site/content/docs/runtime/benchmarks.mdx` 作成

- [x] frontmatter（`title` / `description`）を追加
- [x] `"Benchmark"` を含むベンチマーク説明を追加
- [x] `"pandas"` を含む比較結果を追加
- [x] 比較テーブル（Favnir AOT / pandas / Apache Beam / dbt）を掲載

---

## T2: `benchmarks/compare/run_comparison.sh` 作成

- [x] `C:\Users\yoshi\favnir\benchmarks\compare\run_comparison.sh` を作成
- [x] `"benchmark"` および `"run_comparison"` を含む内容にする
- [x] shebang（`#!/usr/bin/env bash`）を先頭に追加

---

## T3: `driver.rs` — `v64500_tests` 追加

- [x] `// -- v64400_tests (v64.4.0) -- flamegraph AOT --` コメント行の直前に `v64500_tests` を挿入
  - [x] `docs_benchmarks_page_exists`（非空・`"Benchmark"/"benchmark"` 含む・`"pandas"` 含む）
  - [x] `benchmark_compare_script_exists`（非空・`"run_comparison"/"benchmark"` 含む）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v64500_tests` で 2 件 PASS
  - [x] `docs_benchmarks_page_exists` PASS
  - [x] `benchmark_compare_script_exists` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3441 tests passed, 0 failed を確認

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.5.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.5.0 セクションに実績追記（3441 tests）
- [x] `versions/current.md` の「進行中」を v64.5.0（3441 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）
