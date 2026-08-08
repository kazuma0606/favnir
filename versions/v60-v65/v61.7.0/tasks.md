# v61.7.0 タスクリスト

Status: COMPLETE
Version: 61.7.0
Base tests: 3371
Target tests: 3373

---

## T0: 事前確認

- [x] `cargo test` でベース 3371 tests passed, 0 failed を確認
- [x] `parser.rs` の型解析エントリポイント（`parse_type` / `parse_type_expr` 等）を grep で確認
- [x] `_` が型引数位置（`List<_>` 等）でも `parse_type` が呼ばれるか確認
- [x] `resolve_type_expr_with_subst` のシグネチャ（L3941 付近）を確認
- [x] `TypeExpr` を網羅 match しているファイルを `cargo build` で事前特定
- [x] `v61600_tests` が driver.rs に存在することを grep で確認

---

## T1: ast.rs — `TypeExpr::Hole` 追加

- [x] `TypeExpr::Hole(Span)` をバリアントとして追加（`ConstInt` の後）
- [x] `span()` メソッドに `TypeExpr::Hole(s) => s` を追加

---

## T2: exhaustive match 修正（cargo build 駆動）

- [x] `cargo build` を実行してコンパイルエラーファイルを列挙
- [x] `fmt.rs` — `TypeExpr::Hole(_) => "_".to_string()` 相当を追加
- [x] `middle/compiler.rs` — `TypeExpr::Hole(_) => {}` 相当を追加
- [x] `emit_python.rs` — `TypeExpr::Hole(_) => "Any".to_string()` 相当を追加
- [x] `middle/ast_lower_checker.rs` — `TypeExpr::Hole(_) => {}` 相当を追加
- [x] `lsp/references.rs` — `TypeExpr::Hole(_) => {}` 相当を追加
- [x] `lineage.rs` — `TypeExpr::Hole(_) => {}` 相当を追加
- [x] `driver.rs` — build エラーの内容に応じてアームを追加
- [x] 再度 `cargo build` でエラー 0 を確認

---

## T3: parser.rs — `_` を Hole として解析

- [x] 型解析エントリポイントで識別子 `"_"` を `TypeExpr::Hole(span)` に変換する処理を追加
- [x] `List<_>` 等の型引数位置でも Hole が生成されることを手動確認

---

## T4: checker.rs — Hole を Unknown として解決

- [x] `resolve_type_expr_with_self` に `TypeExpr::Hole(_) => Type::Unknown` を追加
- [x] `resolve_type_expr_with_subst` に `TypeExpr::Hole(_) => Type::Unknown` を追加

---

## T5: lint.rs — W040 追加

- [x] `check_w040_type_holes(program, errors)` 関数を追加（関数の戻り型・引数型の Hole を検出）
- [x] `check_lint` 関数の W039 呼び出しの直後に `check_w040_type_holes` を追加

---

## T6: lsp/inlay_hints.rs — `collect_hole_hints` 追加

- [x] `collect_hole_hints(source, type_at) -> Vec<InlayHint>` を追加
  - source を Parser::parse_str で再パースし、`TypeExpr::Hole` を持つ fn を探す
  - Hole のある位置に `InlayHint { label: "inferred".into(), kind: 1 }` を追加
- [x] `handle_inlay_hints` の末尾に呼び出しを追加（`// v61.7.0:` コメント付き）

---

## T7: driver.rs — `v61700_tests` 追加

- [x] `v61600_tests` の直前に `v61700_tests` モジュールを挿入
- [x] `type_hole_infers_correctly` テスト追加
  - `fn f(x: Int) -> _ { x }` が型エラーなしで通ることを確認
- [x] `type_hole_inlay_hint` テスト追加
  - `fn f(x: Int) -> _ { x }` のパース結果が `TypeExpr::Hole` を含むことを確認

---

## T8: ビルド・テスト

- [x] `cargo build` でコンパイルエラー 0
- [x] `cargo test v61700` で 2 件 PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3373 tests passed, 0 failed を確認

---

## T9: ドキュメント更新

- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` v61.7.0 セクションに実績を追記
- [x] `versions/current.md` の「進行中」を v61.7.0（3373 tests）に更新、「次」を v61.8.0 に
- [x] `CHANGELOG.md` に v61.7.0 エントリを追加
- [x] site MDX（型プレースホルダー機能説明）— v61.7.0 では対象外（v62.0 Language Polish 宣言時に追加予定）
- [x] tasks.md を COMPLETE に更新（本ファイル）

---

## コードレビュー指摘対応

- `_` は `TokenKind::Ident("_")` ではなく `TokenKind::Underscore` として lexer が出力する → parser を修正
- `collect_hole_hints` の hint label は `ty.display()` で統一（`Display` 未実装のため `format!("{}", ty)` は使えない）
- `type_hole_inlay_hint` → テスト名を `type_hole_parsed_as_hole` に変更（実際の検証内容に合わせて）

### code-reviewer 指摘対応（実装後レビュー）

- **[BUG][MED]** W040 が `TrfDef` の `input_ty` / `output_ty` / params を未検出 → `check_w040_type_holes` に `Item::TrfDef` アーム追加
- **[BUG][LOW]** `ast.rs::TypeExpr::display()` が Hole を `"..."` にフォールスルー → `TypeExpr::Hole(_) => "_"` を明示追加
- **[BUG][LOW]** `collect_hole_hints` の column 計算が `span.start` ベース（1文字ずれ）→ `span.end` ベースに修正
- **[BUG][LOW]** W040 発火テスト欠如 → `type_hole_w040_fires` テスト追加（3373 → 3374）

---

## 完了サマリー

- Status: COMPLETE
- Tests: 3374 passed, 0 failed（ベース 3371 + 3）
- 完了日: 2026-08-01
