# v61.8.0 タスクリスト

Status: IN PROGRESS
Version: 61.8.0
Base tests: 3374
Target tests: 3376

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3374 tests passed, 0 failed を確認
- [x] `lint_program` のシグネチャを grep で確認（`pub fn lint_program(program: &Program) -> Vec<LintError>`）
- [x] `LintTomlConfig` の既存フィールドを `toml.rs` で確認（`warn_as_error`, `allow` の 2 フィールド）
- [x] `toml.rs` に `parse_lint_config` という独立関数があるか、または `parse_fav_toml` 内インライン処理かを grep で確認
- [x] `cmd_lint` のシグネチャを `driver.rs` で確認（引数リスト）
- [x] `cmd_check` 内に `lint_program` の直接呼び出しが存在するか grep で確認（存在しない場合は「新規追加」アプローチ）
- [x] `fav lint` フラグ解析ループを `main.rs` で確認（`--strict` の有無）
- [x] `v61700_tests` が `driver.rs` に存在することを grep で確認

---

## T1: lint.rs — `LintConfig` + `lint_program_with_config` 追加

- [x] `LintConfig { strict: bool, perf: bool }` 構造体を `lint_program` 直前に追加
- [x] `#[derive(Debug, Clone, Default)]` を付与
- [x] `lint_program_with_config(program: &Program, config: &LintConfig) -> Vec<LintError>` を追加
  - 内部で `lint_program(program)` を呼び出し
  - `config.strict` が true の場合、`e.code == "W040"` の message に ` [strict]` を付与
- [x] `cargo build` でエラーなし

---

## T2: toml.rs — `LintTomlConfig.strict` 追加

- [x] `LintTomlConfig` に `pub strict: Option<bool>` フィールドを追加（コメント `// v61.8.0`）
- [x] `parse_lint_config` / toml パース処理でキー `"strict"` を `bool` としてパース
- [x] `cargo build` でエラーなし

---

## T3: driver.rs — `cmd_lint` 更新 + `cmd_check` に lint 呼び出し追加

- [x] `cmd_lint` のシグネチャに `strict: bool` を追加
- [x] 内部で `use crate::lint::{LintConfig, lint_program_with_config};` を追加（または `crate::lint::` プレフィックス）
- [x] `cmd_lint` 内の `lint_program(&program)` を `lint_program_with_config(&program, &LintConfig { strict, perf: false })` に変更
- [x] `cmd_check` 内には現在 `lint_program` の直接呼び出しが**存在しない**（W006 処理のみ）
  - T0 で確認した `strict` ブロック（L4634 付近）の後に `lint_program_with_config` を**新規追加**する
  - `Program` は `Parser::parse_str(&source, path)` で再パースして取得（既存の `check_single_file` と同様のパターン）
  - strict=true の場合のみ W040 タグ付き lint 結果を表示する（false の場合は従来通り `lint_program` を使用でも可）
- [x] `cargo build` でエラーなし

---

## T4: main.rs — `fav lint --strict` フラグ追加

- [x] `fav lint` フラグ解析ループで `let mut strict = false;` を宣言
- [x] `"--strict" => { strict = true; i += 1; }` を追加
- [x] `cmd_lint(...)` 呼び出しに `strict` を追加
- [x] `cargo build` でエラーなし

---

## T5: driver.rs — `v61800_tests` 追加

- [x] `v61700_tests` の直前に `v61800_tests` モジュールを挿入
- [x] `check_strict_mode_w040_tagged` テスト追加
  - `fn f(x: Int) -> _ { x }` を `lint_program_with_config` + `LintConfig { strict: true, .. }` に渡す
  - W040 のメッセージに `"[strict]"` が含まれることを `assert!`
- [x] `fav_toml_lint_strict` テスト追加
  - `"[lint]\nstrict = true\n"` を含む toml 文字列をパース
  - `LintTomlConfig.strict == Some(true)` を確認

---

## T6: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v61800` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3376 tests passed, 0 failed を確認

---

## T7: ドキュメント更新

- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` v61.8.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v61.8.0（3376 tests）に更新、「次」を v61.9.0 に
- [x] `CHANGELOG.md` に v61.8.0 エントリを追加
- [x] `site/content/docs/tools/lint.mdx`（または既存ドキュメント）に `--strict` フラグと `[lint] strict = true` の説明を追記
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

- **[BUG][false positive] cmd_check での二重 lint 実行** — `check_single_file` は `lint_program` を呼ばないため二重出力は発生しない。指摘は `cmd_check_fix_src`（別関数）との混同
- **[BUG][false positive] W040 strict 専用化** — v61.7.0 仕様「W040 は通常 lint に含める」が意図的設計。strict モードで `[strict]` タグ付与のみが v61.8.0 の変更。Bug 扱いせず
- **[STYLE]** `cmd_lint` の `let strict = strict || ...` シャドーイング → `let strict_mode =` にリネーム
- **[STYLE]** `LintConfig::perf` dead_code 懸念 → `#[allow(dead_code)]` を付与
- **[STYLE]** `toml.rs` の `[lint]` トリガーコメントが不正確 → 正確な歴史的経緯に修正

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3376 passed, 0 failed（ベース 3374 + 2）
- 完了日: 2026-08-01
