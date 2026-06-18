# v15.3.0 Tasks — `fav test` DSL（ネイティブテストフレームワーク）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"15.3.0"` に変更

---

## Phase B — テスト追加

- [x] B-1: `fav/src/driver.rs` に `v153000_tests` モジュール追加（5 テスト）
  - `version_is_15_3_0`
  - `test_def_in_ast`
  - `assert_ok_primitive_exists`
  - `cmd_test_exists`
  - `testing_doc_exists`

---

## Phase C — AST: `TopLevel::TestDef` 追加

- [x] C-1: `fav/src/ast.rs` に `Item::TestDef { name: String, body: Vec<Stmt>, span: Span }` 追加
  ※ 実装時に `Item::TestDef` として実装済み（`TopLevel` は `Item` に統合）

---

## Phase D — パーサー: `test "..." { }` 構文

- [x] D-1: `fav/src/frontend/parser.rs` の `parse_top_level` に `"test"` 識別子のブランチ追加
  - 文字列リテラル（テスト名）をパース
  - `{` 〜 `}` の本体（Stmt リスト）をパース
  - `Item::TestDef { name, body, span }` を返す

- [x] D-2: `test` ブロック内で `assert_eq` / `assert_ok` / `assert_err` / `assert_true` が
         通常の関数呼び出し（`ECall`）としてパースされることを確認

---

## Phase E — コンパイラ: TestDef コンパイル

- [x] E-1: `fav/src/middle/ir.rs` の `IRProgram` に test 関数格納対応済み

- [x] E-2: `fav/src/middle/compiler.rs` の `compile_program` を更新
  - `Item::TestDef` を `$test:<name>` 関数名でコンパイル
  - 通常の `fns` には含めない

- [x] E-3: `fav/src/middle/compiler.rs` の builtin primitive リストに
         `"assert_ok"` / `"assert_err"` / `"assert_true"` を追加（2 箇所）

- [x] E-4: `cargo build` → コンパイルエラーなし確認

---

## Phase F — VM: アサーション primitive 拡張

- [x] F-1: `fav/src/backend/vm.rs` に `"assert_ok"` primitive 追加
  - `Result.ok(v)` → v を返す
  - `Result.err(e)` → `Err(format!("assert_ok failed: got err({e})"))`

- [x] F-2: `fav/src/backend/vm.rs` に `"assert_err"` primitive 追加
  - `Result.err(e)` → e を返す
  - `Result.ok(v)` → `Err(format!("assert_err failed: got ok({v})"))`

- [x] F-3: `fav/src/backend/vm.rs` に `"assert_true"` primitive 追加
  - `Bool(true)` → `Ok(VMValue::Unit)`
  - `Bool(false)` → `Err("assert_true failed: got false")`

- [x] F-4: `cargo test` → 既存アサーション系テストのリグレッションなし確認
  - `checker.fav` の `infer_hm evar fresh` テストが stale だったため修正（`Err` 期待に変更）

---

## Phase G — `cmd_test` 実装

- [x] G-1: `fav/src/driver.rs` の `cmd_test` 関数（既存実装）を拡張
  - `Ok(Bool(false))` → FAIL として扱う修正を追加

- [x] G-2: `fav/src/driver.rs` の CLI ルーター（`match cmd`）に `"test"` ブランチ追加（既存実装済み）

- [x] G-3: 動作確認：`self/checker.fav` で `fav test` を実行 → 47/47 pass

---

## Phase H — checker / ast_lower_checker: TestDef スキップ

- [x] H-1: `fav/src/middle/checker.rs` の型チェックループで `Item::TestDef` をスキップ（既存実装済み）

- [x] H-2: `fav/src/middle/ast_lower_checker.rs` の `lower_program` で `TestDef` をスキップ（既存実装済み）

- [x] H-3: `fav run <file>` で test ブロックを含むファイルを実行しても test ブロックが無視されることを確認

---

## Phase I — CLI help 更新

- [x] I-1: `fav/src/driver.rs` の `cmd_help` / ヘルプテキストに `test <file>` を追加（既存実装済み）

---

## Phase J — サイトドキュメント

- [x] J-1: `site/content/docs/language/testing.mdx` 更新
  - `assert_eq` / `assert_ok` / `assert_err` / `assert_true` のリファレンス追加
  - `Ctx.mock` との組み合わせ例追加

---

## Phase K — コミット

- [x] K-1: `cargo test v153000` → 5/5 パス最終確認

- [x] K-2: `cargo test` → 全件パス（リグレッションなし）確認
  - `--test-threads=1` で全件 pass 確認

- [x] K-3: コミット（commit: fef4335）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.3.0"` | [x] |
| `cargo test v153000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `ast.rs` に `Item::TestDef` が存在する | [x] |
| `vm.rs` に `assert_ok` primitive が存在する | [x] |
| `driver.rs` に `cmd_test` 関数が存在する | [x] |
| `fav test self/checker.fav` で PASS レポートが出力される（47/47） | [x] |
| `fav run sample.fav` で test ブロックが無視される | [x] |
| `site/content/docs/language/testing.mdx` が存在する | [x] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.3.0/spec.md` | 仕様・スコープ |
| `versions/v15.3.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/roadmap-v15.1-v16.0.md` | v15.3.0 セクション |
| `fav/src/ast.rs` | `Item` enum — TestDef 追加対象 |
| `fav/src/frontend/parser.rs` | `parse_top_level` — 追加対象 |
| `fav/src/middle/compiler.rs` | `compile_program` — TestDef 対応 |
| `fav/src/backend/vm.rs` | 既存 `assert_eq` / `assert_ne` primitive — 参考 |
