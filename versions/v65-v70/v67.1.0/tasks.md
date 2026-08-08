# v67.1.0 タスクリスト

Status: COMPLETE
Version: 67.1.0
Base tests: 3497
Target tests: 3499
Actual tests: 3499

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3497 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/debug.rs` が存在しないことを確認（新規作成対象）
- [x] `site/content/docs/tools/debug.mdx` が存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v67000_tests` が存在することを確認（`v67100_tests` の挿入位置）
- [x] `driver.rs` に `v67100_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67000_tests` で 4 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `cargo_toml_version_is_67_0_0`, `changelog_has_v67_0_0`, `milestone_has_ai_native_stage`, `readme_mentions_ai_native`
- [x] `versions/current.md` の「進行中バージョン」が `v67.0.0` であることを確認

---

## T1: `fav/src/debug.rs` 作成 + `main.rs` への `mod` 宣言追加

- [x] `fav/src/debug.rs` を新規作成
  - [x] `"step"` を含む（`debug_step_execution` テストにマッチ）
  - [x] `"inspect"` を含む（`debug_step_execution` テストにマッチ）
  - [x] `"breakpoint"` を含む（`debug_breakpoint_stage` テストにマッチ）
  - [x] `pub fn cmd_debug(src: &str, _args: &[String]) -> String` を実装
- [x] `fav/src/main.rs` に `mod debug;` を追加（未追加だと debug.rs が型チェックされない）
- [x] `cargo build` でエラーなし（debug.rs が型チェックされた状態で）

---

## T2: `site/content/docs/tools/debug.mdx` 作成

- [x] `site/content/docs/tools/debug.mdx` を新規作成
  - [x] MDX 先頭に `import` 文を置かない（acorn パースエラー回避）
  - [x] `fav debug pipeline.fav` の使用例を記述
  - [x] `step` / `breakpoint` / `inspect` コマンドの説明を記述

---

## T3: `driver.rs` — `v67100_tests` 追加

- [x] `// -- v67000_tests (v67.0.0)` コメントの直前に `v67100_tests` を挿入
  - [x] `debug_step_execution`: `include_str!("debug.rs")` に `"step"` と `"inspect"` を含む
  - [x] `debug_breakpoint_stage`: `include_str!("debug.rs")` に `"breakpoint"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T4: ビルド・テスト

- [x] `cargo test --bin fav v67100_tests` で 2 件 PASS
  - [x] `debug_step_execution` PASS
  - [x] `debug_breakpoint_stage` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3499 tests passed, 0 failed を確認

---

## T5: ドキュメント・ステータス更新

> T4 のテスト全通過（3499 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` のバージョン一覧表で v67.1.0 の「状態」列を「未着手」→「完了」に変更し、変更後に当該行が「完了」になっていることを目視確認
- [x] `versions/current.md` の「進行中バージョン」を v67.1.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v67.1〜v67.9 では CHANGELOG.md を更新しない。v68.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: v67.1〜v67.9 では version を変更しない。v68.0.0 宣言時に `"68.0.0"` に更新する。

---

## コードレビュー指摘と対応

- [HIGH] spec-reviewer: `mod debug;` なしで debug.rs がコンパイルされない → spec/plan/tasks に main.rs への追加手順を明記、実装時に追加済み
- [HIGH] code-reviewer: main.rs に `Some("debug")` ディスパッチアームが欠落 → `Some("suggest")` の直前に追加、3499 tests 再確認済み
- [MED] spec-reviewer: plan.md に T5 相当のステップ欠落 → Step 5 追加
- [MED] spec-reviewer: MDX コンテンツの最低要件未定義 → step/breakpoint/inspect 記述要件を明示
