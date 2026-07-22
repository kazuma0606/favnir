# Tasks: v50.8.0 — ドキュメントサイト DX 3.0 記事

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3105 passed, 0 failed を確認（ベース確認）
- [x] `site/content/docs/tools/diagnostics.mdx` が存在しないことを確認（新規作成対象）
- [x] `site/content/docs/tools/trace-watch.mdx` が存在しないことを確認（新規作成対象）
- [x] `include_str!("../../site/content/docs/tools/diagnostics.mdx")` パスが正しいことを確認（`fav/src/driver.rs` 起点: `../../site/...`）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — MDX ドキュメント作成

- [x] `site/content/docs/tools/dap.mdx` を参照し、`trace-watch.mdx` の DAP 比較セクションと矛盾しないことを確認
- [x] `site/content/docs/tools/diagnostics.mdx` 新規作成
  - [x] H1: `# 診断出力 (Diagnostics)` を含む
  - [x] `suggestion` フィールドの説明を含む
  - [x] `fav check --json` の出力例を含む
  - [x] `fav explain --error <code>` の使い方を含む
  - [x] `fav explain --error --list` の説明を含む
  - [x] テスト用キーワード（`diagnostics`, `fav explain`, `suggestion` のいずれか）を含む
  - [x] 300 文字以上
- [x] `site/content/docs/tools/trace-watch.mdx` 新規作成
  - [x] H1: `# トレース & ウォッチ (Trace & Watch)` を含む
  - [x] `fav run --trace` の出力例（`[trace] stage=NAME  out=VALUE`）を含む
  - [x] `fav run --debug`（DAP）との違いを含む（比較テーブル）
  - [x] `--watch` の説明（将来対応予定である旨を明示）を含む
  - [x] テスト用キーワード（`--trace`, `[trace]`, `trace-watch` のいずれか）を含む
  - [x] 300 文字以上

## T2 — `driver.rs` — `v508000_tests` 追加

- [x] `v508000_tests` モジュールを `v507000_tests` の直前に追加（3 件）:
  - [x] `cargo_toml_version_is_50_8_0`: version = "50.8.0" を assert
  - [x] `docs_diagnostics_page_exists`: `content.len() >= 300` かつ診断キーワードが含まれることを assert
  - [x] `docs_trace_watch_page_exists`: `content.len() >= 300` かつ trace キーワードが含まれることを assert
- [x] `v507000_tests::cargo_toml_version_is_50_7_0` を削除（`run_trace_structured_output` / `run_watch_tracks_variable` は保持）

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.8.0"`
- [x] `cargo test` 3107 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.8.0 エントリ追加
- [x] `versions/current.md` を v50.8.0（3107 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.8.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
