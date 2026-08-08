# v67.0.0 タスクリスト

Status: COMPLETE
Version: 67.0.0
Base tests: 3493
Target tests: 3497
Actual tests: 3497

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3493 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（本バージョンで `"67.0.0"` に更新する）
- [x] `driver.rs` に `v66900_tests` が存在することを確認（`v67000_tests` の挿入位置）
- [x] `driver.rs` に `v67000_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66900_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `ai_stage_layer_all_stable`, `ai_rune_docs_complete`
- [x] `versions/current.md` の「進行中バージョン」が `v66.9.0` であることを確認

---

## T1: ファイル更新（4件）

- [x] `fav/Cargo.toml` の version を `"66.0.0"` → `"67.0.0"` に変更
- [x] `MILESTONE.md` の `## v66.0.0` エントリの直前に v67.0.0 エントリを挿入
  - [x] `"AI-Native Stage Layer"` を含む（`milestone_has_ai_native_stage` テストにマッチ）
  - [x] v66.1〜v66.9 の達成内容を記述
  - [x] テスト数 3497 を記載
- [x] `README.md` に v67.0.0 宣言を追加
  - [x] `"AI-Native"` または `"v67.0"` を含む（`readme_mentions_ai_native` テストにマッチ）
- [x] `CHANGELOG.md` の `## [v66.0.0]` エントリの直前に v67.0.0 エントリを挿入
  - [x] `"v67.0.0"` を含む（`changelog_has_v67_0_0` テストにマッチ）
  - [x] v66.1〜v66.9 の成果を一括記載（保留していた CHANGELOG 分）

---

## T2: `driver.rs` — `v67000_tests` 追加・旧テスト修正

- [x] `v66000_tests::cargo_toml_version_is_66_0_0` のアサートを修正する
  - Cargo.toml version を `"67.0.0"` に変更すると、`"version = \"66.0.0\""` チェックが FAIL するため
  - 修正方針: `toml.contains("version = \"67.0.0\"")` に更新（または当該アサートをコメントアウト）
- [x] `// -- v66900_tests (v66.9.0)` コメントの直前に `v67000_tests` を挿入
  - [x] `cargo_toml_version_is_67_0_0`: `include_str!("../Cargo.toml")` に `version = "67.0.0"` を含む
  - [x] `changelog_has_v67_0_0`: `include_str!("../../CHANGELOG.md")` に `"v67.0.0"` を含む
  - [x] `milestone_has_ai_native_stage`: `include_str!("../../MILESTONE.md")` に `"AI-Native Stage Layer"` を含む
  - [x] `readme_mentions_ai_native`: `include_str!("../../README.md")` に `"AI-Native"` または `"v67.0"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト（cargo clean 前）

- [x] `cargo test --bin fav v67000_tests` で 4 件 PASS
  - [x] `cargo_toml_version_is_67_0_0` PASS
  - [x] `changelog_has_v67_0_0` PASS
  - [x] `milestone_has_ai_native_stage` PASS
  - [x] `readme_mentions_ai_native` PASS

---

## T4: `cargo clean` ★クリーンアップ

- [x] `cargo clean` 実行（`fav/` ディレクトリで）
- [x] `fav/tmp/hello.fav` を正しい内容で復元する（存在有無にかかわらず必ず実施）
  - 復元内容: `fn add(a: Int, b: Int) -> Int { a + b }` + 改行 + `fn main() -> Bool { add(1, 2) == 3 }`
- [x] `cargo test -j 8 -- --test-threads=8` で 3497 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3497 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` のバージョン一覧表で v67.0.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.0.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

---

## コードレビュー指摘と対応

- [MED] spec-reviewer: driver.rs の旧 cargo_toml_version テストが Cargo.toml version 変更後に FAIL する → 全17件を sed で一括置換（66.0.0 → 67.0.0）
- [MED] code-reviewer: 旧バージョンテストのエラーメッセージ文字列に `"66.0.0"` が残存 → sed で一括修正（"should have version 66.0.0" 等 → "67.0.0"）、3497 tests 再確認済み
- [MED] spec-reviewer: hello.fav 復元が条件付き記述だった → 必須操作に修正
- [LOW] spec-reviewer: Actual tests フィールド未記載・plan Step4 実行ディレクトリ未明示・spec cargo clean 説明不正確 → 各修正済み
