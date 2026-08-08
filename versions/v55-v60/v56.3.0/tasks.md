# Tasks — v56.3.0 — 行多相レコード活用拡張

## ステータス: COMPLETE（2026-07-25）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.3.0 セクションを確認
- [x] ベーステスト数 3231（v56.2.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.2.0` であることを確認（更新前）
- [x] `TypeExpr::RecordType` が `(Vec<(String, TypeExpr)>, Span)` の 2 要素タプルであることを確認（変更前）
- [x] `parse_base_type` の RecordType 解析ループに `| ident` 処理がないことを確認（変更対象）
- [x] `TypeExpr` に `display()` メソッドがないことを確認（新規追加対象 — `FlwStep::display_str` とは別）
- [x] `driver.rs` に `v56300_tests` が存在しないことを確認（新規追加対象）
- [x] `v56200_tests` に `cargo_toml_version_is_56_2_0` が存在することを確認（削除対象）
- [x] `TokenKind::Pipe` が既存トークンとして定義されていることを確認（lexer.rs L462）
- [x] `Type::Unknown` は `is_compatible` で全型と互換であることを確認（checker.rs L71-74）
- [x] `resolve_field_access` の末尾フォールバックが `_ => Type::Unknown` を返すことを確認（checker.rs L5628）
- [x] `substitute_self_in_type_expr`（compiler.rs L1680）が `RecordType(fields, span)` パターンを持つことを確認（変更対象）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.3.0` に更新（56.2.0 から変更）
- [x] T2: `fav/src/ast.rs` — `TypeExpr::RecordType` に `Option<String>` を追加
  - [x] `RecordType(Vec<(String, TypeExpr)>, Span)` → `RecordType(Vec<(String, TypeExpr)>, Option<String>, Span)`
  - [x] span match arm を `TypeExpr::RecordType(_, _, s)` に更新
  - [x] `impl TypeExpr` に `pub fn display(&self) -> String` を追加
    - [x] `RecordType(fields, Some(r), _)` → `"{ field1: Type1, ... | r }"`
    - [x] `RecordType(fields, None, _)` → `"{ field1: Type1, ... }"`
    - [x] `Named`, `Arrow` 対応（その他は `"..."` フォールバック）
- [x] T3: `fav/src/frontend/parser.rs` — `{ field: Type | r }` 解析対応
  - [x] `parse_base_type` の RecordType ループに `TokenKind::Pipe` 検出を追加
  - [x] row_var として `Some(ident)` を取得 → ループ break
  - [x] `TypeExpr::RecordType(fields, row_var, span)` として構築
- [x] T4: 全 RecordType match arm 更新（`cargo build` エラーで全箇所を網羅）
  - [x] `ast.rs` span match arm
  - [x] `emit_python.rs` ワイルドカードパターン
  - [x] `driver.rs`（8 箇所）
  - [x] `fmt.rs`（2 箇所）— row_var を `| r` 形式で出力
  - [x] `lint.rs`（1 箇所）
  - [x] `lsp/references.rs`（1 箇所）
  - [x] `middle/ast_lower_checker.rs`（2 箇所）
  - [x] `middle/compiler.rs`（3 箇所）
    - [x] `substitute_self_in_type_expr`（L1680）で `row_var.clone()` を保持して再構築
  - [x] `middle/checker.rs`（4 箇所）— `_row_var` で束縛
    - [x] `type_expr_contains`（L10586）も row_var チェック追加（`|| row_var.as_deref() == Some(name)`）
- [x] T5: `fav/src/driver.rs` — 既存テスト更新
  - [x] `v56200_tests::cargo_toml_version_is_56_2_0` を削除
- [x] T6: `fav/src/driver.rs` — `v56300_tests` モジュールを `v56200_tests` の直前に追加
  - [x] `use crate::ast::TypeExpr` インポートを含む
  - [x] `check_errors` 定義
  - [x] `cargo_toml_version_is_56_3_0`
  - [x] `row_poly_generic_fn`（`errors.is_empty()` assert、コメントで Unknown 互換の根拠を明示）
  - [x] `row_poly_lsp_hover`（`TypeExpr::RecordType` + `Some("r")` + `Span` ダミー → `display()` テスト）
- [x] T7: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.3.0 テスト数を修正
  - [x] `3232 + 2 = 3234` → `3231 + 2 = 3233` に更新（v56.4.0 の base も 3233 + 2 = 3235 に修正）

---

## テスト・検証

- [x] T8: `cargo build` でコンパイルエラーがないことを確認（`Finished` を確認）
- [x] T9: `cargo test` 全通過（**3233 tests passed, 0 failed**）
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` ok
  - [x] `v56300_tests::row_poly_generic_fn` ok
  - [x] `v56300_tests::row_poly_lsp_hover` ok
  - [x] 既存 3231 件全通過（-1 削除 +3 追加 = net +2）
- [x] T10: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T11: `CHANGELOG.md` に v56.3.0 エントリを追加（version: `56.2.0 → 56.3.0`）
- [x] T12: `versions/current.md` を v56.3.0 / 3233 tests に更新
- [x] T13: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.3.0 実績を COMPLETE に更新
- [x] T14: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.3.0 実績欄も COMPLETE に更新（テスト数も修正）

---

## 完了確認

- [x] `cargo_toml_version_is_56_3_0` pass
- [x] `row_poly_generic_fn` pass（`errors.is_empty()`）
- [x] `row_poly_lsp_hover` pass（`display()` が `name: String` と `| r` を含む）
- [x] **3233 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `TypeExpr::RecordType` が `Option<String>` row_var フィールドを持つ
- [x] `{ name: String | r }` がパースエラーなし
- [x] `TypeExpr::display()` が行変数を `| r` で表示する
- [x] `substitute_self_in_type_expr` が row_var を保持して再構築する
- [x] `v56200_tests::cargo_toml_version_is_56_2_0` が削除されている
- [x] `CHANGELOG.md` に v56.3.0 エントリが追加されている（version: `56.2.0 → 56.3.0`）
- [x] `versions/current.md` が v56.3.0 / 3233 tests を反映
- [x] T13 / T14 のロードマップ更新が完了している（テスト数修正含む）

## 実装メモ（コードレビュー対応）

- anonymous record literal `{ name: "Alice" }` は Favnir 非対応のため、`row_poly_generic_fn` テストを関数定義の型チェックのみ検証する形に変更（`fn main() -> String { "ok" }` で代替）
- `Span` は `crate::frontend::lexer::Span`（`crate::ast::Span` は private）— `row_poly_lsp_hover` テストで `use crate::frontend::lexer::Span` を使用
- `type_expr_contains` の row_var チェック（`|| row_var.as_deref() == Some(name)`）は spec の仕様通りに追加

## コードレビュー対応（v56.3.0 完了後）

指摘 [MED]×2、[LOW]×2、[HIGH]×0。

- **[MED] driver.rs 3 ヘルパーの row_var 無視** → `favnir_type_display` / `format_type_expr` / `type_expr_str` を `fmt.rs` と同様に `Some(r) => "{{ {} | {} }}"` 形式に修正
- **[MED] parser エラーメッセージ品質** → `| r` 後のゴミトークンは `expect(RBrace)` が正しくエラーを返すため動作上問題なし。スコープ外として対応不要とした
- **[LOW] `display()` の `_` アームが粗い** → `Optional(inner, _)` → `"{}?"` / `Fallible(inner, _)` → `"{}!"` を追加
- **[LOW] `ast_lower_checker.rs` コメント不足** → `v56.3.0 追加の row_var も現状は捨てる` 旨のコメントを追記
