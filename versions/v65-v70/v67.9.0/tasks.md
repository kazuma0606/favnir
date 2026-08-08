# v67.9.0 タスクリスト

Status: COMPLETE
Version: 67.9.0
Base tests: 3513
Target tests: 3515

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3513 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- [x] `fav/src/debug.rs` / `fav/src/viz.rs` / `fav/src/suggest.rs` / `fav/src/simulate.rs` が存在することを確認
- [x] `site/content/docs/tools/developer-intelligence.mdx` が存在しないことを確認（新規作成）
- [x] `driver.rs` に `v67800_tests` が存在することを確認（`v67900_tests` の挿入位置）
- [x] `driver.rs` に `v67900_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v67800_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `doc_math_latex_rendered`, `doc_math_example_compiles`
- [x] `versions/current.md` の「進行中バージョン」が `v67.8.0` であることを確認

---

## T1: `site/content/docs/tools/developer-intelligence.mdx` 新規作成

- [x] ファイルを新規作成
  - [x] `"fav debug"` を含む（`debug_viz_suggest_docs_complete` テストが要求）
  - [x] `fav viz` / `fav suggest` / `fav simulate` / `Rune.proptest` / `fav profile --interactive` / `fav doc --math` を紹介
  - [x] 各機能に bash コードサンプルを含める

---

## T2: `driver.rs` — `v67900_tests` 追加

- [x] 挿入前に `grep "v67800_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認
- [x] `// -- v67800_tests (v67.8.0) -- Math-Aware Doc Generation --` の直前に `v67900_tests` を挿入
  - [x] `dev_intelligence_all_stable`:
    - `include_str!("debug.rs")` に `"cmd_debug"` を含む
    - `include_str!("viz.rs")` に `"cmd_viz"` を含む
    - `include_str!("suggest.rs")` に `"cmd_suggest"` を含む
    - `include_str!("simulate.rs")` に `"cmd_simulate"` を含む
  - [x] `debug_viz_suggest_docs_complete`:
    - `include_str!("../../site/content/docs/tools/developer-intelligence.mdx")` に `"fav debug"` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（driver.rs テスト挿入後）

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v67900_tests` で 2 件 PASS
  - [x] `dev_intelligence_all_stable` PASS
  - [x] `debug_viz_suggest_docs_complete` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3515 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

> T3 のテスト全通過（3515 tests passed）を確認してから実施すること。

- [x] `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.9.0「状態」列を「未着手」→「完了」に変更
- [x] `versions/current.md` の「進行中バージョン」を v67.9.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

> **sub-version ポリシー**: v67.x では Cargo.toml / CHANGELOG.md は変更しない。v68.0.0 宣言時に一括更新する。

---

## 設計上の意図的省略

- v67.1〜v67.8 のソースコード変更なし（コードフリーズ版）
- `Rune.proptest` 型チェック確認: v67.6.0 で完了済みのため本バージョン対象外

## コードレビュー指摘と対応

| 深刻度 | 内容 | 対応 |
|--------|------|------|
| [MED] | `developer-intelligence.mdx` にフロントマターがない | 既存 `tools/` MDX がフロントマターなし統一のため対応不要 |
| [LOW] | `contains("cmd_debug")` 等の文字列アサーションが弱い（コメント内でも真になりうる） | `"pub fn cmd_XXX"` 形式に変更してより堅牢なアサーションに修正 |
