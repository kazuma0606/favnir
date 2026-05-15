# Favnir v0.8.0 タスク一覧

更新日: 2026-04-30（Phase 1-5 全完了）

> [ ] 未完了 / [x] 完了
>
> **ゴール**: fav test / fmt / lint / explain 強化 + eval.rs 廃止
> **前提**: v0.7.2（三相アーキテクチャ）完了済み
> **現状**: Phase 0-5 全完了・251 テスト通過

---

## Phase 0-A: バージョン文字列統一 ✅

- [x] `Cargo.toml`: `version = "0.1.0"` → `"0.8.0"`
- [x] `src/main.rs` HELP テキスト: `"v0.6.0"` → `"v0.8.0"`
- [x] `cargo build` が通ること

---

## Phase 0-B: コンパイラ警告解消 ✅

### 削除（呼び出し元なし確認済み）
- [x] `middle/checker.rs`: `fn compose_effects` を削除
- [x] `middle/checker.rs`: `fn merge_effect` を削除
- [x] `middle/checker.rs`: `fn instantiate` を削除
- [x] `middle/checker.rs`: Checker 構造体の `type_params` フィールドを削除
- [x] `middle/checker.rs`: Checker 構造体の `subst` フィールドを削除

### `#[allow(dead_code)]` 追加
- [x] `ast.rs` 先頭に `#![allow(dead_code)]` を追加
  - 対象: TypeExpr の Span フィールド群、Field.span、EmitUnion、NamespaceDecl/UseDecl
- [x] `cargo build` で警告ゼロを確認

---

## Phase 0-C: value.rs 切り出し ✅

- [x] `src/value.rs` 新規作成
- [x] `eval.rs` の `pub enum Value { ... }` と全 `impl Value` を `value.rs` に移動
- [x] `src/main.rs`: `mod value;` を追加
- [x] `backend/vm.rs`: `use crate::value::Value`
- [x] `driver.rs`: Value の参照パスを確認・更新
- [x] `cargo test` 全通過確認

---

## Phase 0-D: eval.rs の廃止（段階的） ✅

### Step 1: fav run を VM に切り替え ✅
- [x] `driver.rs`: `cmd_run` から `Interpreter::run_with_db` の呼び出しを削除
- [x] `driver.rs`: `build_artifact → vm.exec_main` のフローに変更
- [x] `driver.rs`: `--db` フラグを VM に渡す
- [x] `cargo test` 全通過確認

### Step 2: eval.rs の枯らし・削除 ✅
- [x] `src/eval.rs` を削除
- [x] `src/main.rs`: `mod eval;` を削除
- [x] eval.rs テストを VM ベースで移植
  - `backend/vm_legacy_coverage_tests.rs`（10 テスト）
  - `backend/vm_stdlib_tests.rs`（50 テスト）
- [x] `cargo test` 全通過確認（235 テスト通過）

---

## Phase 1: fav test ✅

### 1-1: 構文追加 ✅
- [x] `frontend/lexer.rs`: `TokenKind::Test` 追加（`"test"` キーワード）
- [x] `ast.rs`: `struct TestDef { name: String, body: Block, span: Span }` 追加
- [x] `ast.rs`: `Item::TestDef(TestDef)` を追加
- [x] `frontend/parser.rs`: `parse_test_item()` 実装

### 1-2: 型検査 ✅
- [x] `middle/checker.rs`: `check_test_def()` 実装
- [x] `assert` / `assert_eq` / `assert_ne` を型 env に登録

### 1-3: VM 組み込みアサーション ✅
- [x] `backend/vm.rs`: `"assert"`, `"assert_eq"`, `"assert_ne"` 追加

### 1-4: CLI ✅
- [x] `driver.rs`: `cmd_test(file, filter, fail_fast)` 実装
- [x] `src/main.rs`: `"test"` コマンドをディスパッチに追加

### 1-5: examples と動作確認 ✅
- [x] `examples/test_sample.fav` — 10 tests
- [x] `examples/math.test.fav` — 8 tests (*.test.fav 形式)
- [x] `fav test examples/test_sample.fav` が通ること（10 passed）
- [x] `fav test examples/math.test.fav` が通ること（8 passed）

**追加**: first-pass `next_fn_idx` reservation bug 修正（closure fn_idx がテスト関数自身を指す問題）

---

## Phase 2: fav fmt ✅

### 2-1: フォーマッタ本体（MVP） ✅
- [x] `src/fmt.rs` 新規作成
- [x] `fn format_program(prog: &Program) -> String`
- [x] fn / trf / flw / type / cap / impl / test 全定義対応
- [x] block インデント管理（4 スペース）
- [x] match アームを各行に整形
- [x] トップレベル定義間に空行 1 行

### 2-2: CLI ✅
- [x] `driver.rs`: `cmd_fmt(file, check)` 実装
- [x] `src/main.rs`: `"fmt"` コマンドをディスパッチに追加

### 2-3: テスト ✅
- [x] `fmt::tests` — 7 idempotency tests
- [x] `fav fmt examples/hello.fav --check` が差分なしで通ること
- [x] `cargo test` 全通過確認（242 テスト）

---

## Phase 3: エラーメッセージ改善 ✅

### 3-1: 診断ヘルパー ✅
- [x] `driver.rs`: `format_diagnostic(source, error) -> String` 実装
- [x] `^^^` アンダーライン生成
- [x] `error[EXXX]:` プレフィックス形式（元々対応済み）

### 3-2: エラー出力の更新 ✅
- [x] `cmd_check`: `format_diagnostic` で出力
- [x] `load_and_check_program`: `format_diagnostic` で出力（fav run のエラーも含む）

### 3-3: テスト ✅
- [x] `fav check` のエラー出力に `^^^` が表示されること（確認済み）
- [x] `cargo test` 全通過確認（242 テスト）

---

## Phase 4: fav explain 強化 ✅

### 4-1: IR ベース DEPS 収集 ✅
- [x] `middle/ir.rs`: `fn collect_deps(fn_def, globals) -> Vec<String>` 追加
- [x] Builtin FieldAccess → `"IO.println"` 形式
- [x] Fn/VariantCtor Global → fn 名

### 4-2: cmd_explain の更新 ✅
- [x] explain 時に `compile_program` を呼んで IRProgram を取得
- [x] DEPS 列を出力テーブルに追加

### 4-4: テスト ✅
- [x] `fav explain examples/hello.fav` に DEPS 列が表示されること（確認済み）
- [x] `cargo test` 全通過確認（242 テスト）

---

## Phase 5: fav lint（MVP） ✅

### 5-1: lint.rs 本体 ✅
- [x] `src/lint.rs` 新規作成
- [x] `struct LintError { code, message, span }`
- [x] `fn lint_program(program) -> Vec<LintError>`
- [x] L003: `fn` 名がスネークケースでない
- [x] L004: `type` 名がパスカルケースでない
- [x] L002: 未使用の `bind` 束縛
- [x] L001: `pub fn` が `_infer` 戻り値型（省略推論型）→ LintError

### 5-2: CLI ✅
- [x] `driver.rs`: `cmd_lint(file, warn_only)` 実装
  - [x] `--warn-only`: exit 0 のまま警告を表示
- [x] `src/main.rs`: `"lint"` コマンドをディスパッチに追加

### 5-3: テスト ✅
- [x] `lint::tests` — 9 テスト（L002/L003/L004 検出 + クリーンファイル）
- [x] `fav lint examples/hello.fav` が ok（exit 0）
- [x] `cargo test` 全通過確認（251 テスト）

---

## 全体完了条件

- [x] `cargo build` で警告ゼロ
- [x] `cargo test` 全テスト通過（251 テスト）
- [x] `fav test examples/test_sample.fav` が動く
- [x] `fav fmt examples/hello.fav --check` が差分なし
- [x] `fav lint examples/hello.fav` が ok
- [x] エラー出力に `^^^` アンダーラインが表示される
- [x] `Cargo.toml` バージョンが `"0.8.0"`
- [ ] roadmap.md の v0.8.0 を完了マーク（次ステップ）
