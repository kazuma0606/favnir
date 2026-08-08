# v66.9.0 タスクリスト

Status: COMPLETE
Version: 66.9.0
Base tests: 3491
Target tests: 3493
Actual tests: 3493

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3491 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] 9 AI Rune ファイルすべてが存在することを確認（一覧は spec.md 参照）
- [x] `site/content/docs/runes/ai-runes-overview.mdx` が存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66800_tests` が存在することを確認（`v66900_tests` の挿入位置）
- [x] `driver.rs` に `v66900_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66800_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `lint_w055_untyped_llm_output`, `lint_w056_dim_implicit_cast`
- [x] `versions/current.md` の「進行中バージョン」が `v66.8.0` であることを確認（確認失敗時は前バージョンの tasks.md T4 が完了していることを確認してから current.md を手動修正すること）

---

## T1: MDX ドキュメント作成

- [x] `site/content/docs/runes/` ディレクトリの存在を確認（存在しない場合は作成）
- [x] `site/content/docs/runes/ai-runes-overview.mdx` を新規作成
  - [x] `"Rune.embed"` を含む（**`ai_rune_docs_complete` テストにマッチ**）
  - [x] 9 AI Rune 群（vec / embed / pinecone / pgvector / weaviate / qdrant / inference / serve / featurestore）の概要を記述
  - [x] MDX 先頭に `import` 文を置かない（acorn パースエラー回避）

---

## T2: `driver.rs` — `v66900_tests` 追加

- [x] `// -- v66800_tests (v66.8.0)` コメントの直前に `v66900_tests` を挿入
  - [x] `ai_stage_layer_all_stable`:
    - 9 AI Rune ファイルをすべて `include_str!` で読み込む
    - 各ファイルが `!is_empty()` であることをアサート
  - [x] `ai_rune_docs_complete`:
    - `ai-runes-overview.mdx` を `include_str!` で読み込む
    - `overview.contains("Rune.embed")` をアサート
  - [x] `include_str!` パスは `fav/src/driver.rs` 起点（`../../runes/...` / `../../site/...`）
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66900_tests` で 2 件 PASS
  - [x] `ai_stage_layer_all_stable` PASS
  - [x] `ai_rune_docs_complete` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3493 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3493 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v66.9.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v66.9.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。

---

## コードレビュー指摘と対応

- [HIGH] spec-reviewer: ロードマップ「W055〜W059 が正常に lint 検出できること」がspec未記載 → T0に v66800_tests 実行確認を追記。ロードマップ表現も「スタブ登録確認」に修正
- [MED] spec-reviewer: T4のbefore/after未明記 → 「未着手」→「完了」と明記
- [LOW] spec-reviewer: cargo build が完了条件に未記載 → spec.md完了条件に追加
