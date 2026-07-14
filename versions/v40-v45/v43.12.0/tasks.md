# v43.12.0 タスク — W031〜W033 lint（冗長型注釈の警告）

## ステータス: COMPLETE（2026-07-13）— 2935 tests

---

## T0 — 事前確認

- [x] `cargo test` 2932 / 0 確認
- [x] `Cargo.toml` version = `43.11.0` 確認
- [x] `v431200_tests` が `fav/src/driver.rs` に存在しないことを確認
- [x] `check_w031` が `fav/src/lint.rs` に存在しないことを確認
- [x] `check_w032` が `fav/src/lint.rs` に存在しないことを確認
- [x] `"W031"` 文字列が `fav/src/lint.rs` に存在しないことを確認

---

## T1 — lint.rs: check_w031 追加

- [x] `fn check_w031_redundant_return_annotation(program: &Program, warnings: &mut Vec<LintError>)` を追加
  - `FnDef.return_ty.is_some()` かつ `body.stmts.is_empty()` かつ `*body.expr` が `Expr::Lit` or `Expr::Ident`
  - `Expr::Lit(Lit::Unit, _)` は除外（副作用関数の慣用的注釈）
- [x] `lint_program()` に `check_w031_redundant_return_annotation(program, &mut warnings);` を追加

---

## T2 — lint.rs: check_w032 追加

- [x] `fn check_w032_explicit_generic_type_arg(program: &Program, warnings: &mut Vec<LintError>)` を追加
  - FnDef の `body.stmts` および `body.expr` の両方を走査
- [x] `fn check_w032_in_expr(expr: &Expr, warnings: &mut Vec<LintError>)` ヘルパー追加
  - `Expr::TypeApply(_, _, span)` を W032 として検出
- [x] `fn check_w032_in_stmt(stmt: &Stmt, warnings: &mut Vec<LintError>)` ヘルパー追加
- [x] `lint_program()` に `check_w032_explicit_generic_type_arg(program, &mut warnings);` を追加
- [x] `lint_program()` に `// W033: 将来版（AST 拡張後に実装 — Expr::Closure はパラメータ型を保持しない）` スタブコメント追加

---

## T3 — driver.rs: v431200_tests 追加 / スタブ化 / Cargo.toml

- [x] `v431100_tests` の直前に `v431200_tests` を挿入（3 件）
- [x] `v431100_tests::cargo_toml_version_is_43_11_0` をスタブ化（`// Stubbed: version bumped to 43.12.0 in v43.12.0.`）
- [x] `fav/Cargo.toml` version を `43.11.0` → `43.12.0` に更新

---

## T4 — CHANGELOG.md

- [x] v43.12.0 エントリ追加

---

## T5 — テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` 実行
- [x] 2935 passed; 0 failed 確認
- [x] `v431200_tests` 3 件 pass 確認

---

## T6 — バージョン管理ドキュメント更新

- [x] `versions/current.md` → v43.12.0 最新安定版（2935 tests）、次版 v43.13.0
- [x] `versions/roadmap/roadmap-v43.1-v44.0.md` → v43.12.0 を `✅ COMPLETE（2026-07-13）`
- [x] `versions/v40-v45/v43.12.0/tasks.md` → COMPLETE、全チェックボックス `[x]`

---

## 実装時の知見

- **`Expr::Lit(Lit::Unit, _)` の除外**: `fn main() -> Unit { () }` のような副作用関数の慣用的注釈に W031 が誤検知していた。`Lit::Unit` を明示的に除外することで解決。
- **Favnir の TypeApply 構文**: Rust の `::<T>` ではなく `expr<T>(args)` — `identity::<Int>(42)` はパースエラー。正しくは `type_name_of<Row>()` 形式。
- **`Block.expr` は `Box<Expr>`（非 Option）**: `&*fd.body.expr` でデリファレンスして `&Expr` として使う。
- **W031 の lint_clean_file_no_errors との競合**: 既存テストの `public fn main() -> Unit { () }` が W031 をトリガーした。Unit 除外パターンで解決。
