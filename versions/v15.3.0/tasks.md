# v15.3.0 Tasks — `fav test` DSL（ネイティブテストフレームワーク）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"15.3.0"` に変更

---

## Phase B — テスト追加

- [ ] B-1: `fav/src/driver.rs` に `v153000_tests` モジュール追加（5 テスト）
  - `version_is_15_3_0`
  - `test_def_in_ast`
  - `assert_ok_primitive_exists`
  - `cmd_test_exists`
  - `testing_doc_exists`

---

## Phase C — AST: `TopLevel::TestDef` 追加

- [ ] C-1: `fav/src/ast.rs` に `TopLevel::TestDef { name: String, body: Vec<Stmt>, span: Span }` 追加

---

## Phase D — パーサー: `test "..." { }` 構文

- [ ] D-1: `fav/src/frontend/parser.rs` の `parse_top_level` に `"test"` 識別子のブランチ追加
  - 文字列リテラル（テスト名）をパース
  - `{` 〜 `}` の本体（Stmt リスト）をパース
  - `TopLevel::TestDef { name, body, span }` を返す

- [ ] D-2: `test` ブロック内で `assert_eq` / `assert_ok` / `assert_err` / `assert_true` が
         通常の関数呼び出し（`ECall`）としてパースされることを確認

---

## Phase E — コンパイラ: TestDef コンパイル

- [ ] E-1: `fav/src/middle/ir.rs` の `IRProgram` に `test_fns: Vec<IRFnDef>` フィールド追加

- [ ] E-2: `fav/src/middle/compiler.rs` の `compile_program` を更新
  - `TopLevel::TestDef` を `test_fns` にコンパイル
  - テスト関数名: `__test__<name_with_spaces_replaced_by_underscores>`
  - 通常の `fns` には含めない

- [ ] E-3: `fav/src/middle/compiler.rs` の builtin primitive リストに
         `"assert_ok"` / `"assert_err"` / `"assert_true"` を追加

- [ ] E-4: `cargo build` → コンパイルエラーなし確認
  - `IRProgram` の `test_fns` フィールドを参照している箇所の `.. Default::default()` または
    `test_fns: vec![]` で初期化漏れを修正

---

## Phase F — VM: アサーション primitive 拡張

- [ ] F-1: `fav/src/backend/vm.rs` に `"assert_ok"` primitive 追加
  - `Result.ok(v)` → v を返す
  - `Result.err(e)` → `Err(format!("assert_ok failed: got err({e})"))`

- [ ] F-2: `fav/src/backend/vm.rs` に `"assert_err"` primitive 追加
  - `Result.err(e)` → e を返す
  - `Result.ok(v)` → `Err(format!("assert_err failed: got ok({v})"))`

- [ ] F-3: `fav/src/backend/vm.rs` に `"assert_true"` primitive 追加
  - `Bool(true)` → `Ok(VMValue::Unit)`
  - `Bool(false)` → `Err("assert_true failed: got false")`

- [ ] F-4: `cargo test` → 既存アサーション系テストのリグレッションなし確認

---

## Phase G — `cmd_test` 実装

- [ ] G-1: `fav/src/driver.rs` に `cmd_test(path: &str)` 関数追加
  - ファイルをパース → `TopLevel::TestDef` を収集
  - テストなしの場合 "no tests found" と表示して終了
  - `running N tests` ヘッダー出力
  - 各 TestDef を `test_fns` 経由でコンパイル・VM 実行
  - `test <name> ... ok` / `test <name> ... FAILED` を出力
  - 失敗一覧（`failures:` セクション）を出力
  - `test result: ok/FAILED. N passed; M failed` サマリー出力
  - FAIL が 1 件以上の場合 `process::exit(1)`

- [ ] G-2: `fav/src/driver.rs` の CLI ルーター（`match cmd`）に `"test"` ブランチ追加
  - `fav test <file>` → `cmd_test(file)`
  - 引数なしの場合はエラーメッセージ表示

- [ ] G-3: 動作確認：サンプル `test` ブロックを含む `.fav` ファイルで `fav test` を実行

---

## Phase H — checker / ast_lower_checker: TestDef スキップ

- [ ] H-1: `fav/src/middle/checker.rs` の型チェックループで `TopLevel::TestDef` をスキップ
  - `fav check <file>` で test ブロックを含むファイルを処理してもエラーにならないこと

- [ ] H-2: `fav/src/middle/ast_lower_checker.rs` の `lower_program` で `TestDef` をスキップ

- [ ] H-3: `fav run <file>` で test ブロックを含むファイルを実行しても test ブロックが無視されることを確認

---

## Phase I — CLI help 更新

- [ ] I-1: `fav/src/driver.rs` の `cmd_help` / ヘルプテキストに `test <file>` を追加

---

## Phase J — サイトドキュメント

- [ ] J-1: `site/content/docs/language/testing.mdx` 新規作成
  - `test "..." { }` 構文説明
  - `assert_eq` / `assert_ok` / `assert_err` / `assert_true` のリファレンス
  - `fav test <file>` の実行例と期待出力
  - `Ctx.mock` との組み合わせ例

---

## Phase K — コミット

- [ ] K-1: `cargo test v153000` → 5/5 パス最終確認

- [ ] K-2: `cargo test` → 全件パス（リグレッションなし）確認

- [ ] K-3: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.3.0"` | [ ] |
| `cargo test v153000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `ast.rs` に `TopLevel::TestDef` が存在する | [ ] |
| `vm.rs` に `assert_ok` primitive が存在する | [ ] |
| `driver.rs` に `cmd_test` 関数が存在する | [ ] |
| `fav test sample.fav` で PASS/FAIL レポートが出力される | [ ] |
| `fav run sample.fav` で test ブロックが無視される | [ ] |
| `site/content/docs/language/testing.mdx` が存在する | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.3.0/spec.md` | 仕様・スコープ |
| `versions/v15.3.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/roadmap-v15.1-v16.0.md` | v15.3.0 セクション |
| `fav/src/ast.rs` | `TopLevel` enum — 追加対象 |
| `fav/src/frontend/parser.rs` | `parse_top_level` — 追加対象 |
| `fav/src/middle/compiler.rs` | `compile_program` — TestDef 対応 |
| `fav/src/backend/vm.rs` | 既存 `assert_eq` / `assert_ne` primitive — 参考 |
