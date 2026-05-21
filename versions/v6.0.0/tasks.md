# Favnir v6.0.0 タスクリスト — セルフホスト

作成日: 2026-05-20

---

## Phase A: VM Primitive 追加

- [x] A-1: `IO.argv() -> List<String>` を vm.rs に実装
- [x] A-2: `IO.argv` の型シグネチャを checker.rs に追加
- [x] A-3: `List.take_while` を vm.rs に実装
- [x] A-4: `List.drop_while` を vm.rs に実装
- [x] A-5: `List.take_while` / `List.drop_while` の型シグネチャを checker.rs に追加
- [x] A-6: vm_stdlib_tests.rs に `test_io_argv` を追加
- [x] A-7: vm_stdlib_tests.rs に `test_list_take_while` を追加
- [x] A-8: vm_stdlib_tests.rs に `test_list_drop_while` を追加
- [x] A-9: `cargo test` 通過確認

---

## Phase B: レキサー (`fav/self/lexer.fav`)

- [x] B-1: `fav/self/` ディレクトリを作成
- [x] B-2: `Token` sum type を定義（全 47 トークン）
- [x] B-3: `is_digit` / `is_alpha` / `is_whitespace` ヘルパー関数を実装
- [x] B-4: `scan_number` 関数を実装（Int / Float リテラル）
- [x] B-5: `scan_string_chars` 関数を実装（文字列リテラル、エスケープ対応）
- [x] B-6: `keyword_token` / `single_char_token` 関数を実装（識別子・キーワード判定）
- [x] B-7: `scan_op` 関数を実装（単文字・2文字演算子）
- [x] B-8: `scan` 再帰関数を実装（空白・コメントスキップ含む）
- [x] B-9: `lex(src: String) -> Result<List<Token>, String>` を実装
- [x] B-10: `fav check fav/self/lexer.fav` エラーなし確認
- [x] B-11: `driver.rs` に `self_tests::self_hosted_lexer_type_checks` を追加
- [x] B-12: Rust レキサーとの出力比較テストを数ケース追加
- [x] B-13: `cargo test` 通過確認（985 tests）

---

## Phase C: パーサー (`fav/self/parser.fav`)

- [x] C-1: `Lit` / `BinOp` / `Pat` sum type を定義
- [x] C-2: `Expr` 再帰的 sum type を定義
- [x] C-3: `Stmt` / `MatchArm` / `TypeExpr` 型を定義
- [x] C-4: `FnDef` / `TypeDef` / `Item` / `Program` 型を定義
- [x] C-5: `ParseState` + `peek` / `advance` / `expect` ヘルパーを実装
- [x] C-6: `parse_expr` (Pratt パーサー) を実装
- [x] C-7: `parse_if` / `parse_match` を実装
- [x] C-8: `parse_block` (`bind x <- expr` 列) を実装
- [x] C-9: `parse_fn_def` を実装
- [x] C-10: `parse_type_def` を実装
- [x] C-11: `parse` トップレベル関数を実装
- [x] C-12: `fav check fav/self/parser.fav` エラーなし確認
- [x] C-13: 簡単な式のパーステストを追加（`1 + 2 * 3` など）
- [x] C-14: 関数定義のパーステストを追加
- [x] C-15: `cargo test` 通過確認（986 tests）

---

## Phase D: 型チェッカー (`fav/self/checker.fav`)

- [x] D-1: `Type` sum type を定義
- [x] D-2: `type_to_str` / `str_to_type` 変換関数を実装
- [x] D-3: `CheckState` 型を定義（`env: Map<String, String>`, `errors: List<String>`）
- [x] D-4: `infer_lit` 関数を実装（リテラルの型推論）
- [x] D-5: `infer_binop` 関数を実装（二項演算の型推論）
- [x] D-6: `infer_var` 関数を実装（変数参照の型解決）
- [x] D-7: `infer_call` 関数を実装（関数呼び出しの型推論）
- [x] D-8: `infer_if` 関数を実装（if/else の型推論）
- [x] D-9: `infer_match` 関数を実装（match 式の型推論）
- [x] D-10: `infer_expr` ディスパッチ関数を実装
- [x] D-11: `check_fn_def` 関数を実装（関数定義の型チェック）
- [x] D-12: `check` トップレベル関数を実装
- [x] D-13: `fav check fav/self/checker.fav` エラーなし確認
- [x] D-14: 基本型推論のテストを追加
- [x] D-15: `cargo test` 通過確認

---

## Phase E: バイトコードコンパイラ (`fav/self/codegen.fav`)

- [x] E-1: `Opcode` sum type を定義（全オペコード）
- [x] E-2: `ConstVal` sum type を定義（定数プール用）
- [x] E-3: `CodegenCtx` 型を定義（定数プール、ローカル変数マップ、命令列）
- [x] E-4: `encode_opcode` 関数を実装（`Int.shl` / `Int.band` を使用）
- [x] E-5: `add_const` 関数を実装（定数をプールに追加）
- [x] E-6: `compile_lit` 関数を実装
- [x] E-7: `compile_binop` 関数を実装
- [x] E-8: `compile_if` 関数を実装（`OpJumpIfFalse` パッチバック）
- [x] E-9: `compile_expr` ディスパッチ関数を実装
- [x] E-10: `compile_fn_def` 関数を実装
- [x] E-11: `compile` トップレベル関数を実装（`List<Int>` を返す）
- [x] E-12: `fav check fav/self/codegen.fav` エラーなし確認
- [x] E-13: `1 + 2` のバイトコード出力テストを追加
- [x] E-14: `cargo test` 通過確認

---

## Phase F: 統合ドライバ (`fav/self/compiler.fav`)

- [x] F-1: 各フェーズの関数を `compiler.fav` にインライン化（または rune import で結合）
- [x] F-2: `compile_file(path: String) -> Result<List<Int>, String> !Io` を実装
- [x] F-3: `main` エントリポイントを実装（`IO.argv` でファイルパス受け取り）
- [x] F-4: `fav check fav/self/compiler.fav` エラーなし確認
- [x] F-5: `fav run fav/self/compiler.fav -- fav/tmp/test1.fav` が動作確認
- [x] F-6: end-to-end テスト（`fav run` で Favnir 製コンパイラを実行）を追加
- [x] F-7: `cargo test` 通過確認

---

## Phase G: ブートストラップ検証

- [x] G-1: `fav build fav/self/compiler.fav -o /tmp/compiler_s1.fvc` が成功
- [x] G-2: `fav run /tmp/compiler_s1.fvc -- fav/tmp/hello.fav` が正常終了
- [x] G-3: Stage 1 バイトコードで `compiler.fav` 自体をコンパイル（Stage 2 生成）
- [x] G-4: Stage 1 == Stage 2 の一致確認（diff でゼロ差分）
- [x] G-5: `fav/src/driver/self_tests.rs` にブートストラップテストを追加

---

## Phase H: ドキュメント

- [x] H-1: `site/content/docs/language/testing.mdx` を新規作成
  - `test "description" { ... }` 構文の説明
  - `fav test` / `fav test <file>` の使い方
  - `--filter` / `--fail-fast` / `--no-capture` / `--coverage` オプション
  - `bench "description" { ... }` 構文と `fav bench` の説明
  - テスト出力フォーマット（PASS / FAIL）の例
- [x] H-2: `site/lib/docs.ts` の `categoryOrder` に `'テスト'` カテゴリを追加（必要であれば）
- [x] H-3: `site/content/docs/language/testing.mdx` の frontmatter に `category: "言語仕様"` / `order` を設定

---

## Phase I: まとめ

- [x] I-1: `cargo test` 全件通過
- [x] I-2: `versions/v6.0.0/tasks.md` にチェックを入れる
- [x] I-3: `MEMORY.md` を更新
- [x] I-4: `feat: self-hosting — Favnir compiler written in Favnir (v6.0.0)` でコミット
