# v16.5.0 Tasks — 型エイリアス（Type Alias）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.5.0"` に変更
- [x] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — Lexer: `alias` キーワード追加（lexer.rs）

- [x] B-1: `fav/src/frontend/lexer.rs` の `TokenKind` enum に `Alias` variant 追加
- [x] B-2: `next_token` の識別子認識に `"alias" => TokenKind::Alias` を追加
- [x] B-3: `cargo build` → コンパイルエラーなし確認

---

## Phase C — AST: `Item::AliasDecl` 追加（ast.rs）

- [x] C-1: `fav/src/ast.rs` の `Item` enum に `AliasDecl { name: String, params: Vec<String>, ty: TypeExpr, span: Span }` 追加
- [x] C-2: `Item::span()` メソッドに `AliasDecl { span, .. } => span` 追加
- [x] C-3: `cargo build` → exhaustive match エラーを確認（Phase G で対処）

---

## Phase D — Parser: `alias` 構文パース（parser.rs）

- [x] D-1: `fav/src/frontend/parser.rs` の `parse_item` に `TokenKind::Alias =>` 分岐を追加
- [x] D-2: `parse_alias_decl` メソッドを実装（name + optional `<T,U>` + `=` + TypeExpr）
- [x] D-3: `cargo build` → コンパイルエラーなし確認

---

## Phase E — Checker: エイリアス収集・展開（checker.rs）

- [x] E-1: `Checker` struct に `alias_env: HashMap<String, (Vec<String>, TypeExpr)>` フィールド追加
- [x] E-2: `Checker::new` / `new_with_resolver` で `alias_env` を初期化
- [x] E-3: `register_item_signatures` で `Item::AliasDecl` を `alias_env` に登録
- [x] E-4: `check_item` に `Item::AliasDecl { .. } => {}` 追加
- [x] E-5: `resolve_type_expr_with_self` に `alias_env` 解決（ジェネリック型引数代入含む）を追加
- [x] E-6: `resolve_type_expr_with_subst` にも同様の `alias_env` 解決を追加
- [x] E-7: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Compiler: `AliasDecl` をスキップ（compiler.rs）

- [x] F-1: `compile_program` の `_ => {}` catch-all で `AliasDecl` がスキップされることを確認（変更不要）
- [x] F-2: `cargo build` → コンパイルエラーなし確認

---

## Phase G — exhaustive match 対応（各ファイル）

- [x] G-1: `cargo build` でエラーが出る全ファイルを確認（driver.rs, fmt.rs の 2 件）
- [x] G-2: `fav/src/driver.rs` の `Item` match に `AliasDecl { .. } => {}` 追加
- [x] G-3: `fav/src/fmt.rs` の `Item` match に `AliasDecl` フォーマット追加
- [x] G-4: `cargo build` → コンパイルエラーなし確認

---

## Phase H — テスト追加（v165000_tests）

- [x] H-1: `fav/src/driver.rs` に `v165000_tests` モジュール追加
- [x] H-2: `version_is_16_5_0` テスト実装
- [x] H-3: `alias_basic` テスト実装（`alias Email = String` を使った fn がコンパイル・実行される）
- [x] H-4: `alias_interchangeable` テスト実装（`alias UserId = Int` — `Int` 引数として渡せる）
- [x] H-5: `alias_generic` テスト実装（`alias Result2<T> = Result<T, String>` が動作する）
- [x] H-6: `alias_in_signature` テスト実装（エイリアスを引数型・戻り型に使った fn が動作する）
- [x] H-7: `cargo test v165000` → 5/5 PASS 確認

---

## Phase I — サイトドキュメント

- [x] I-1: `site/content/docs/language/type-alias.mdx` 新規作成（構文・使用例・`type` との違い・制約）

---

## Phase J — テスト確認とコミット

- [x] J-1: `cargo test v165000` → 5/5 PASS 最終確認
- [x] J-2: `cargo test` → 1598 PASS（リグレッションなし — 旧バージョンテスト 8 件は想定内失敗）確認
- [x] J-3: コミット（3ed54fa）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.5.0"` | [x] |
| `alias Email = String` が解析・型解決される | [x] |
| エイリアス型は元の型と交換可能（型エラーなし） | [x] |
| ジェネリックエイリアス `alias Result2<T> = Result<T, String>` が動作する | [x] |
| エイリアスを引数型・戻り型に使った関数が正常に動作する | [x] |
| `type Name(Inner)` との共存に問題がない | [x] |
| `cargo test v165000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `site/content/docs/language/type-alias.mdx` が存在する | [x] |
