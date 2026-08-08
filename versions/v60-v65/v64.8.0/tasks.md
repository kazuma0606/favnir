# v64.8.0 タスクリスト

Status: COMPLETE
Version: 64.8.0
Base tests: 3445
Target tests: 3447

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3445 tests passed, 0 failed を確認
- [x] `site/content/docs/performance/performance1-overview.mdx` が存在しないことを確認（新規作成）
- [x] `site/content/docs/performance/` ディレクトリが存在することを確認（既存）
- [x] `driver.rs` に `v64800_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64700_tests` が存在することを確認（`v64800_tests` の挿入位置）

---

## T1: MDX 作成

- [x] `site/content/docs/performance/performance1-overview.mdx` を新規作成
  - [x] frontmatter（title / description）を含む
  - [x] `"Performance 1.0"` を含む（`docs_performance1_overview_exists` 通過条件）
  - [x] Quick Start セクションに `fav build` / `fav bench` / `fav profile` / `fav lint` を含む（`docs_performance1_has_quickstart` 通過条件）
  - [x] Performance Certification Checklist を含む
  - [x] ベンチマーク比較表（Favnir AOT vs pandas / Apache Beam / dbt）を含む

---

## T2: `driver.rs` — `v64800_tests` 追加

- [x] `// -- v64700_tests` コメント行の直前に `v64800_tests` を挿入
  - [x] `docs_performance1_overview_exists`（`"Performance 1.0"` を含む）
  - [x] `docs_performance1_has_quickstart`（`"fav build"` / `"fav bench"` / `"fav profile"` / `"fav lint"` を含む）
  - [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v64800_tests` で 2 件 PASS
  - [x] `docs_performance1_overview_exists` PASS
  - [x] `docs_performance1_has_quickstart` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3447 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.8.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.8.0 セクションに実績追記（3447 tests）
- [x] `versions/current.md` の「進行中」を v64.8.0（3447 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）

**非スコープ（T4 では実施しない）**: `site/` navigation/sidebar への `performance1-overview.mdx` 追記は v64.9 以降

## コードレビュー対応

- [HIGH] `current.md` の「次に切る版」が v64.4.0 のまま → v64.9.0 に更新
- [LOW] MDX 相対リンク先が存在しないパス → `./aot`/`./performance`/`./benchmarks` を `../runtime/aot` 等に修正（`./profiling` はそのまま）
