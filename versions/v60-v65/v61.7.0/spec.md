# v61.7.0 — `_` 型プレースホルダー（部分型注釈・推論ヒント）

## 概要

型注釈位置に `_` を書くと型推論が自動的に型を埋める「型プレースホルダー」を追加する。
`TypeExpr::Hole` を AST に追加し、parser / checker / lint / LSP inlay hints を一貫して更新する。

---

## 動機

現状、型注釈は完全に明示する必要がある。`_` を使うと省略した型だけ推論させ、
LSP の inlay hint で「何が推論されたか」を確認できる。

```favnir
fn process(rows: List<_>) -> _ {
  rows |> List.filter(|r| r.active)
}
// `_` は型推論が埋める → inlay hint: List<Row> -> List<Row>
```

---

## スコープ

### 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `TypeExpr::Hole(Span)` バリアント追加、`span()` メソッド更新 |
| `fav/src/frontend/parser.rs` | 型注釈位置の `_` を `TypeExpr::Hole` として解析 |
| `fav/src/middle/checker.rs` | `resolve_type_expr_with_self` と `resolve_type_expr_with_subst` に `Hole` アーム追加 |
| `fav/src/lsp/inlay_hints.rs` | `collect_hole_hints` 追加、`handle_inlay_hints` に統合 |
| `fav/src/lint.rs` | W040 `type_hole_inferred` ルール追加 |
| 下記 exhaustive match ファイル群 | `TypeExpr::Hole` アーム追加（コンパイルエラー駆動） |

### exhaustive match 更新ファイル（cargo build エラー駆動で確定）

以下は TypeExpr を網羅 match しているファイル。`Hole` バリアント追加により
コンパイルエラーが発生するため、順次 `TypeExpr::Hole(_) => ...` アームを追加する。

- `fav/src/fmt.rs` — `TypeExpr::Hole(_) => "_".to_string()`
- `fav/src/middle/compiler.rs` — traverse 不要のため `TypeExpr::Hole(_) => {}`
- `fav/src/emit_python.rs` — `TypeExpr::Hole(_) => "Any".to_string()`（Python 型ヒントとして）
- `fav/src/middle/ast_lower_checker.rs` — `TypeExpr::Hole(_) => {}`
- `fav/src/lsp/references.rs` — `TypeExpr::Hole(_) => {}`
- `fav/src/lineage.rs` — `TypeExpr::Hole(_) => {}`
- `fav/src/driver.rs` — 必要なアームを追加（build エラーで確認）

---

## 実装詳細

### 1. ast.rs — `TypeExpr::Hole` 追加

```rust
/// `_` — type placeholder: inferred by the type checker (v61.7.0)
Hole(Span),
```

`span()` メソッドに追加:
```rust
TypeExpr::Hole(s) => s,
```

`display()` には `_ => "...".to_string()` の catch-all が既にあるため変更不要。

### 2. parser.rs — `_` を Hole として解析

型解析関数内で識別子 `"_"` を検出したら `TypeExpr::Hole` を返す。

```rust
// 型注釈位置で "_" が来たら Hole として返す
if ident == "_" {
    return Ok(TypeExpr::Hole(span));
}
```

型注釈が許可される場所:
- 関数パラメータ型（`fn f(x: _)`）
- 関数戻り型（`fn f() -> _`）
- ジェネリック型引数（`List<_>`）

### 3. checker.rs — Hole を Unknown として解決

`resolve_type_expr_with_self` に追加:
```rust
TypeExpr::Hole(_) => Type::Unknown,
```

`resolve_type_expr_with_subst` にも同様に追加:
```rust
TypeExpr::Hole(_) => Type::Unknown,
```

`Type::Unknown` は `is_compatible` で全型と互換であるため、Hole が型チェックを
ブロックしない。型推論が文脈から型を自動的に決定する。

### 4. lsp/inlay_hints.rs — `collect_hole_hints`

```rust
/// v61.7.0: `_` 型プレースホルダーの位置に inlay hint を追加。
/// 関数の戻り型または引数型に TypeExpr::Hole があれば「inferred」とヒント表示。
pub(crate) fn collect_hole_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    // source を再パースして TypeExpr::Hole を持つ位置を列挙し、
    // type_at から対応する型を参照して InlayHint を生成する。
    // type_at に該当 span がない場合はヒントを出力しない（Unknown 型を表示しても有益でないため）。
    let mut hints = Vec::new();
    // ...
    hints
}
```

`handle_inlay_hints` に呼び出しを追加:
```rust
// v61.7.0: `_` 型プレースホルダーのヒント
hints.extend(collect_hole_hints(&doc.source, &doc.type_at));
```

### 5. lint.rs — W040 `type_hole_inferred`

```rust
// ── W040: type hole `_` inferred (v61.7.0) ───────────────────────────────────
fn check_w040_type_holes(program: &Program, errors: &mut Vec<LintError>) {
    for fd in &program.functions {
        if let Some(ret_ty) = &fd.ret_ty {
            if matches!(ret_ty, TypeExpr::Hole(_)) {
                errors.push(LintError::new(
                    "W040",
                    format!("type hole `_` in return type of `{}` — consider making explicit", fd.name),
                    fd.span.clone(),
                ));
            }
        }
        for param in &fd.params {
            if matches!(param.ty, TypeExpr::Hole(_)) {
                errors.push(LintError::new(
                    "W040",
                    format!("type hole `_` in parameter `{}` of `{}` — consider making explicit", param.name, fd.name),
                    param.span.clone(),
                ));
            }
        }
    }
}
```

`check_lint` 関数に追加:
```rust
// v61.7.0: W040
check_w040_type_holes(program, &mut errors);
```

---

## 完了条件

- **Rust テスト 2 件**（ベース 3371 + 2 = 3373 tests passed, 0 failed）
  - `type_hole_infers_correctly` — `_` 戻り型で型エラーが発生しないことを確認
  - `type_hole_inlay_hint` — パーサーが `_` を `TypeExpr::Hole` として解析することを確認

---

## 注意事項

- `resolve_type_expr_with_self` は `&self`（immutable）のため `fresh_var`（`&mut self`）は呼べない。`Type::Unknown` を返すことで実質的に同じ効果を達成する
- `display()` には `_ => "..."` catch-all が既にあるため追加不要
- `span()` は exhaustive match のため `Hole(s) => s` を追加必須
- W040 は v61.7.0 で通常の `fav lint` に含める（`--strict` フラグによる有効化は v61.8.0 で実装）。ロードマップの旧記述「W039 は --strict 下でのみ有効化」は誤記（W039 は v61.2.0 実装済み）であり修正済み
- exhaustive match の更新漏れは `cargo build` のエラーで検出できる
