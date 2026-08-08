# v64.3.0 タスクリスト

Status: COMPLETE
Version: 64.3.0
Base tests: 3435
Target tests: 3437

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3435 tests passed, 0 failed を確認
- [x] `site/content/docs/runtime/performance.mdx` が存在しないことを確認（新規作成）
- [x] `site/content/docs/runtime/aot.mdx` が存在することを確認（同ディレクトリへの配置確認）
- [x] `driver.rs` に `v64300_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64200_tests` が存在することを確認（`v64300_tests` の挿入位置）

---

## T1: `site/content/docs/runtime/performance.mdx` 作成

- [x] frontmatter（title / description）を追加
- [x] `## AOT Compilation` セクション — `fav build --link` / `--ci` / `fav bench` の使い方
- [x] `## Incremental Compilation` セクション — キャッシュヒット説明
- [x] `## Parallel Execution` セクション — `par` / `--opt-stats`
- [x] `## DAG Optimization` セクション — dead stage 削除 / pure stage fusion
- [x] `## Backpressure Control` セクション — `[backpressure]` toml 設定例
- [x] `## Bottleneck Identification Workflow` セクション — 4ステップ手順
- [x] `## Regression Detection` セクション — `[bench]` / `regression_threshold_pct`
- [x] `"AOT"` / `"fav bench"` / `"fav profile"` がファイル内に含まれることを確認

---

## T2: `driver.rs` — `v64300_tests` 追加

- [x] `v64200_tests` の直前に `v64300_tests` を挿入
  - `docs_performance_guide_exists`（非空・`"Performance"` 含む）
  - `docs_performance_has_aot_section`（`"AOT"` / `"fav bench"` / `"fav profile"` 含む）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v64300_tests` で 2 件 PASS
  - `docs_performance_guide_exists` PASS
  - `docs_performance_has_aot_section` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3437 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.3.0 エントリを追加（v64.2.0 エントリを参照してフォーマットを統一）
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.3.0 セクションに実績追記（完了条件テスト数を 3437 に修正）
- [x] `versions/current.md` の「進行中」を v64.3.0（3437 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）
