# v16.5.0 Tasks — 型エイリアス（Type Alias）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [ ] A-1: `fav/Cargo.toml` の `version` を `"16.5.0"` に変更
- [ ] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — Lexer: `alias` キーワード追加（lexer.rs）

- [ ] B-1: `fav/src/frontend/lexer.rs` の `TokenKind` enum に `Alias` variant 追加
- [ ] B-2: `next_token` の識別子認識に `"alias" => TokenKind::Alias` を追加
- [ ] B-3: `cargo build` → コンパイルエラーなし確認

---

## Phase C — AST: `TopLevel::AliasDecl` 追加（ast.rs）

- [ ] C-1: `fav/src/ast.rs` の `TopLevel` enum に `AliasDecl { name: String, params: Vec<String>, ty: TypeExpr, span: Span }` 追加
- [ ] C-2: `cargo build` → exhaustive match エラーを確認（Phase G で対処）

---

## Phase D — Parser: `alias` 構文パース（parser.rs）

- [ ] D-1: `fav/src/frontend/parser.rs` の `parse_top_level` に `TokenKind::Alias => self.parse_alias_decl()` 分岐を追加
- [ ] D-2: `parse_alias_decl` メソッドを実装（name + optional `<T,U>` + `=` + TypeExpr）
- [ ] D-3: `cargo build` → コンパイルエラーなし確認

---

## Phase E — Checker: エイリアス収集・展開（checker.rs）

- [ ] E-1: `Checker` struct に `alias_env: HashMap<String, (Vec<String>, Type)>` フィールド追加
- [ ] E-2: `Checker::new` / `init_env` で `alias_env` を初期化
- [ ] E-3: `check_program` の先頭で全 `TopLevel::AliasDecl` を走査し `alias_env` に登録
- [ ] E-4: `resolve_alias(ty: Type) -> Type` メソッドを実装（Named → alias_env 参照、ジェネリック展開）
- [ ] E-5: `check_fn_def` のパラメータ型・戻り型に `resolve_alias` を適用
- [ ] E-6: 型比較箇所（`unify` または `check_type_compat`）で両辺に `resolve_alias` を適用
- [ ] E-7: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Compiler: `AliasDecl` をスキップ（compiler.rs）

- [ ] F-1: `fav/src/middle/compiler.rs` の `compile_top_level` に `TopLevel::AliasDecl { .. } => {}` 追加
- [ ] F-2: `cargo build` → コンパイルエラーなし確認

---

## Phase G — exhaustive match 対応（各ファイル）

- [ ] G-1: `cargo build` でエラーが出る全ファイルを確認
- [ ] G-2: `fav/src/driver.rs` の `TopLevel` match に `AliasDecl { .. } => { }` 追加
- [ ] G-3: `fav/src/lineage.rs` の `TopLevel` match に追加（存在する場合）
- [ ] G-4: `fav/src/lint.rs` の `TopLevel` match に追加（存在する場合）
- [ ] G-5: `fav/src/emit_python.rs` の `TopLevel` match に追加（存在する場合）
- [ ] G-6: `fav/src/fmt.rs` の `TopLevel` match に追加（存在する場合）
- [ ] G-7: `fav/src/middle/ast_lower_checker.rs` の `TopLevel` match に追加（フォールバック）
- [ ] G-8: その他 exhaustive match エラーが出るファイルに追加
- [ ] G-9: `cargo build` → コンパイルエラーなし確認

---

## Phase H — テスト追加（v165000_tests）

- [ ] H-1: `fav/src/driver.rs` に `v165000_tests` モジュール追加
- [ ] H-2: `version_is_16_5_0` テスト実装
- [ ] H-3: `alias_basic` テスト実装（`alias Email = String` を使った fn がコンパイル・実行される）
- [ ] H-4: `alias_interchangeable` テスト実装（`alias UserId = Int` — `Int` 引数として渡せる）
- [ ] H-5: `alias_generic` テスト実装（`alias Result2<T> = Result<T, String>` が動作する）
- [ ] H-6: `alias_in_signature` テスト実装（エイリアスを引数型・戻り型に使った fn が動作する）
- [ ] H-7: `cargo test v165000` → 5/5 PASS 確認

---

## Phase I — サイトドキュメント

- [ ] I-1: `site/content/docs/language/type-alias.mdx` 新規作成（構文・使用例・`type` との違い・制約）

---

## Phase J — テスト確認とコミット

- [ ] J-1: `cargo test v165000` → 5/5 PASS 最終確認
- [ ] J-2: `cargo test` → 全件 PASS（リグレッションなし）確認
- [ ] J-3: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.5.0"` | [ ] |
| `alias Email = String` が解析・型解決される | [ ] |
| エイリアス型は元の型と交換可能（型エラーなし） | [ ] |
| ジェネリックエイリアス `alias Result2<T> = Result<T, String>` が動作する | [ ] |
| エイリアスを引数型・戻り型に使った関数が正常に動作する | [ ] |
| `type Name(Inner)` との共存に問題がない | [ ] |
| `cargo test v165000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `site/content/docs/language/type-alias.mdx` が存在する | [ ] |
