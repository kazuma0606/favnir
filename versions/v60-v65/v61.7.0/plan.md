# v61.7.0 実装計画

## フェーズ

### Phase 1: ast.rs — `TypeExpr::Hole` 追加
- `TypeExpr::Hole(Span)` をバリアントとして追加
- `span()` メソッドに `TypeExpr::Hole(s) => s` を追加

### Phase 2: コンパイルエラー駆動で exhaustive match を修正
- `cargo build` を実行し、TypeExpr の exhaustive match エラーが出るファイルを特定
- 各ファイルに `TypeExpr::Hole(_) => ...` アームを追加（内容はファイルごとに適切な値）
  - fmt.rs: `"_".to_string()`
  - compiler.rs / ast_lower_checker.rs / lsp/references.rs / lineage.rs: `{}` (no-op)
  - emit_python.rs: `"Any".to_string()`
  - driver.rs: build エラーの内容に応じて対応

### Phase 3: parser.rs — `_` を Hole として解析
- 型解析関数（`parse_type` 系）で識別子 `"_"` を検出したら `TypeExpr::Hole(span)` を返す
- `List<_>` など型引数位置でも動作することを確認

### Phase 4: checker.rs — Hole を Unknown として解決
- `resolve_type_expr_with_self` に `TypeExpr::Hole(_) => Type::Unknown` を追加
- `resolve_type_expr_with_subst` に `TypeExpr::Hole(_) => Type::Unknown` を追加

### Phase 5: lint.rs — W040 追加
- `check_w040_type_holes` 関数を追加
- `check_lint` に呼び出しを追加（W039 の後）

### Phase 6: lsp/inlay_hints.rs — `collect_hole_hints` 追加
- `collect_hole_hints(source, type_at)` を追加
  - source を再パースして `TypeExpr::Hole` 位置を取得
  - InlayHint を生成（ラベル: "inferred"）
- `handle_inlay_hints` に呼び出しを追加

### Phase 7: driver.rs — テスト追加
- `v61700_tests` モジュールを `v61600_tests` の直前に挿入
- `type_hole_infers_correctly`: `fn f(x: Int) -> _ { x }` が型エラーなしで通ることを確認
- `type_hole_inlay_hint`: `_` が `TypeExpr::Hole` としてパースされることを確認

### Phase 8: ビルド・テスト
- `cargo build` でコンパイルエラー 0
- `cargo test v61700` で 2 件 PASS
- `cargo test -j 8` で 3373 tests passed, 0 failed

## 実装順序の根拠

- Phase 1 → Phase 2: AST 追加 → コンパイルエラー修正の順で進める（エラー駆動）
- Phase 3 は Phase 2 完了後（parser の変更が exhaustive match に影響しないため）
- Phase 4 は parser が動作してから（checker の動作確認をしたい）
- W040 (Phase 5) は parser 完了後（`TypeExpr::Hole` のパターンを実際に使うため）
- LSP hint (Phase 6) は最後（parser + checker 完了後にテスト可能）

## リスク

- `_` が既存の識別子（変数名等）として使われているケースとの衝突
  → `_` は型注釈位置のみ Hole として解釈（式位置は変更なし）
- `List<_>` の型引数位置で `parse_type` が呼ばれているか確認が必要
  → parser.rs で型引数パースコードを grep して確認（T0 で実施）
- `resolve_type_expr_with_subst` の第三フィールド数が spec と異なる可能性
  → checker.rs の実際のシグネチャを読んで確認（T0 で実施）
