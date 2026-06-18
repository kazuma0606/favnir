# v16.6.0 Tasks — モジュールシステム強化（Namespace Alias）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.6.0"` に変更
- [x] A-2: `cargo build` → コンパイルエラーなし確認

---

## Phase B — Lexer: `as` キーワード追加（lexer.rs）

- [x] B-1: `fav/src/frontend/lexer.rs` の `TokenKind` enum に `As` variant 追加
- [x] B-2: `next_token` の識別子認識に `"as" => TokenKind::As` を追加
- [x] B-3: `cargo build` → コンパイルエラーなし確認

---

## Phase C — AST: `Item::UseAlias` 追加（ast.rs）

- [x] C-1: `fav/src/ast.rs` の `Item` enum に `UseAlias { original: String, alias: String, span: Span }` 追加
- [x] C-2: `Item::span()` に `UseAlias { span, .. } => span` 追加
- [x] C-3: `cargo build` → exhaustive match エラーを確認（Phase G で対処）

---

## Phase D — Parser: `use X as Y` パース（parser.rs）

- [x] D-1: `fav/src/frontend/parser.rs` の `TokenKind::Use` 分岐で、`use IDENT as IDENT` パターンを検出
- [x] D-2: `as` が続く場合に `Item::UseAlias { original, alias, span }` を返す分岐を追加
- [x] D-3: `as` が続かない場合は既存の `RuneUse` パスへフォールスルー
- [x] D-4: `is_rune_use_pattern()` で `use X as Y` を rune-use パスから除外（`TokenKind::As` チェック追加）
- [x] D-5: `parse_import_decl` で `import "path" as alias` の `as` を `TokenKind::As` で受付（旧 `peek_ident_text("as")` 修正）

---

## Phase E — Compiler: `namespace_aliases` + エイリアス解決（compiler.rs）

- [x] E-1: `CompileCtx` struct に `namespace_aliases: HashMap<String, String>` フィールド追加
- [x] E-2: `CompileCtx::new` / 初期化箇所で `namespace_aliases: HashMap::new()` 追加
- [x] E-3: `compile_program` のグローバル登録フェーズに `Item::UseAlias` の処理を追加（`namespace_aliases.insert`）
- [x] E-4: `compile_expr` の `Expr::FieldAccess` 処理で、`obj` が `Ident` の場合に `namespace_aliases` でエイリアス解決してから既存ロジックを実行
- [x] E-5: `cargo build` → コンパイルエラーなし確認

---

## Phase F — Checker: `namespace_aliases` + exhaustive match（checker.rs）

- [x] F-1: `Checker` struct に `namespace_aliases: HashMap<String, String>` フィールド追加
- [x] F-2: `Checker::new` / `new_with_resolver` で `namespace_aliases: HashMap::new()` 追加
- [x] F-3: `register_item_signatures` に `Item::UseAlias` の処理を追加（`namespace_aliases.insert`）
- [x] F-4: `check_item` に `Item::UseAlias { .. } => {}` 追加
- [x] F-5: `check_builtin_apply` で `namespace_aliases` を参照してエイリアス解決
- [x] F-6: `cargo build` → コンパイルエラーなし確認

---

## Phase G — exhaustive match 対応（各ファイル）

- [x] G-1: `cargo build` でエラーが出る全ファイルを確認
- [x] G-2: `fav/src/driver.rs` の `Item` match に `UseAlias { .. } => {}` 追加
- [x] G-3: `fav/src/fmt.rs` の `Item` match に `UseAlias` フォーマット追加（`"use {original} as {alias}"`）
- [x] G-4: その他 exhaustive match エラーが出るファイルに追加
- [x] G-5: `cargo build` → コンパイルエラーなし確認

---

## Phase H — テスト追加（v166000_tests）

- [x] H-1: `fav/src/driver.rs` に `v166000_tests` モジュール追加
- [x] H-2: `version_is_16_6_0` テスト実装
- [x] H-3: `namespace_alias_string` テスト実装（`use String as S` → `S.concat` が動作）
- [x] H-4: `namespace_alias_list` テスト実装（`use List as L` → `L.length` が動作）
- [x] H-5: `namespace_alias_math` テスト実装（`use Math as M` → `M.abs(-42)` = 42）
- [x] H-6: `namespace_alias_multi` テスト実装（`use String as S; use List as L` — 複数共存）
- [x] H-7: `cargo test v166000` → 5/5 PASS 確認

---

## Phase I — サイトドキュメント

- [x] I-1: `site/content/docs/language/modules.mdx` 新規作成（`use` 構文・エイリアス・プロジェクトインポート）

---

## Phase J — テスト確認とコミット

- [x] J-1: `cargo test v166000` → 5/5 PASS 最終確認
- [x] J-2: `cargo test` → 全件 PASS（リグレッションなし）確認
- [x] J-3: コミット

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.6.0"` | [x] |
| `as` キーワードがレキサーで認識される | [x] |
| `Item::UseAlias` が AST に追加されている | [x] |
| `use String as S` → `S.concat` が `String.concat` として動作する | [x] |
| `use List as L` → `L.length` が `List.length` として動作する | [x] |
| `use Math as M` → `M.abs` が `Math.abs` として動作する | [x] |
| 複数エイリアスが共存して動作する | [x] |
| `cargo test v166000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `site/content/docs/language/modules.mdx` が存在する | [x] |
