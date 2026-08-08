# v61.4.0 Spec — record update 式（`{ base | field: new_value }`）

Date: 2026-07-31
Status: REVIEWED

---

## 概要

ETL で「既存レコードの一部フィールドだけ書き換えた新レコードを作る」操作を簡潔に記述できる構文を追加する。

```favnir
bind updated <- { row | status: "active", score: row.score + 10 }
bind enriched <- { order | total: order.price * order.qty, currency: "JPY" }
```

既存の RecordSpread `{ ...base, field: val }` と意味論的に等価だが、より ETL 的に読みやすいパイプスタイルの構文。

---

## vm.rs および codegen.rs に関する補足

`Expr::RecordUpdate` は `compiler.rs` で `IRExpr::RecordSpread` にデシュガーする。
`vm.rs` と `codegen.rs` は AST の `Expr` を直接処理しないため、変更不要。
既存の `MergeRecord` opcode を再利用する。

---

## 実装スコープ（9 ファイル）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `Expr::RecordUpdate` バリアント追加 |
| `fav/src/frontend/parser.rs` | 更新 | `{ ident \| field: val }` の解析（`LBrace` アームに条件追加） |
| `fav/src/middle/checker.rs` | 更新 | `check_expr` と `collect_free_vars_expr` の `RecordUpdate` アーム追加 |
| `fav/src/middle/compiler.rs` | 更新 | `compile_expr` と `collect_free_vars_expr` の `RecordUpdate` アーム追加 |
| `fav/src/fmt.rs` | 更新 | `Expr::RecordUpdate` フォーマット（`{ base \| field: val }` 形式） |
| `fav/src/lint.rs` | 更新 | lint の各 `Expr` walk 関数に `RecordUpdate` アーム追加 |
| `fav/src/emit_python.rs` | 更新 | `Expr::RecordUpdate` → Python `{**base, "field": val}` |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | `Expr::RecordUpdate` lower |
| `fav/src/driver.rs` | 更新 | `v61400_tests` モジュール追加 |

新規ファイルなし。`vm.rs`・`codegen.rs`・`ir.rs` は変更不要（デシュガー方式）。
`Cargo.toml` バージョン変更なし（サブバージョン）。

---

## AST 変更（`ast.rs`）

```rust
/// `{ base | field: val, ... }` — record update 式 (v61.4.0)
/// base の型の全フィールドを継承し、fields で指定したフィールドを上書きする。
RecordUpdate {
    base: Box<Expr>,
    fields: Vec<(String, Expr)>,
    span: Span,
},
```

`span()` メソッドの exhaustive match アームにも追加:
```rust
Expr::RecordUpdate { span, .. } => span,
```

---

## パーサー変更（`parser.rs`）

### 判別ロジック（`LBrace` アーム）

`{` の後に `ident` + `|`（Pipe）が続く場合、RecordUpdate として解析する。

```rust
TokenKind::LBrace => {
    if self.peek2() == Some(&TokenKind::DotDotDot) {
        // ... RecordSpread（既存）
    } else if matches!(self.peek2(), Some(TokenKind::Ident(_)))
        && self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::Pipe)
    {
        // v61.4.0: RecordUpdate: { ident | field: val, ... }
        let start = self.peek_span().clone();
        self.advance(); // consume '{'
        self.parse_record_update(start)
    } else {
        Ok(Expr::Block(Box::new(self.parse_block()?)))
    }
}
```

**設計注**: base は識別子で始まる式に限定（`{ get_row(id) | ... }` 等の複合式は v61.4.0 ではサポートしない）。
識別子チェックは `peek2() = Ident AND tokens[pos+2] = Pipe` で行う。

**`parse_expr()` と `|` の安全性**: `Pipe`（`|`）は `parse_expr` / `parse_logical_or` で中置演算子として使われない
（`||` は `PipePipe`、`|>` は `PipeGt`、`|` 単体は closure 開始 `|params|` のみ）。
したがって `parse_expr(base)` は `|` を消費せず、直後の `expect(Pipe)` は正しく機能する。

### `parse_record_update` 新規追加

`{` を消費した後に呼ばれる。

```rust
/// `{ base | field: val, ... }` — record update 式パーサー (v61.4.0)
/// `{` を消費済みの状態で呼ばれる。
fn parse_record_update(&mut self, start: Span) -> Result<Expr, ParseError> {
    let base = self.parse_expr()?;
    self.expect(&TokenKind::Pipe)?; // consume '|'
    let mut fields: Vec<(String, Expr)> = Vec::new();
    while self.peek() != &TokenKind::RBrace {
        let (field_name, _) = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let val = self.parse_expr()?;
        fields.push((field_name, val));
        if self.peek() == &TokenKind::Comma {
            self.advance(); // trailing comma 許容
        }
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(Expr::RecordUpdate {
        base: Box::new(base),
        fields,
        span: self.span_from(&start),
    })
}
```

---

## チェッカー変更（`checker.rs`）

### `check_expr` — `Expr::RecordUpdate` アーム

`lookup_field_type` メソッド（L4593）と standalone の `unify` 関数（L370）を使う。
`self.types_compatible` は存在しないため使用しない。`self.type_defs` の値は `TypeBody`（`TypeDef` ではない）。

```rust
// v61.4.0: record update 式
Expr::RecordUpdate { base, fields, span } => {
    let base_ty = self.check_expr(base);
    for (fname, fexpr) in fields {
        let val_ty = self.check_expr(fexpr);
        let expected_ty = self.lookup_field_type(&base_ty, fname);
        if !matches!(expected_ty, Type::Unknown) {
            if unify(&val_ty, &expected_ty).is_err() {
                self.type_error(
                    "E0102",
                    &format!(
                        "record update: field `{}` expects {:?}, got {:?}",
                        fname, expected_ty, val_ty
                    ),
                    span,
                );
            }
        }
    }
    base_ty
}
```

**注**: `lookup_field_type` は `Type::Named` の場合のみフィールド型を返し、それ以外は `Type::Unknown` を返す。
`unify` は `pub fn unify(t1: &Type, t2: &Type) -> Result<Subst, String>` — `self.unify` ではない。

### `collect_free_vars_expr` — checker.rs には存在しない

`grep -n 'collect_free_vars_expr' fav/src/middle/checker.rs` で 0 件。変更不要。

### `infer_called_fns` / `infer_effects_for_program` の `Expr::RecordUpdate` アーム

lint.rs 等の各 `Expr` walk 関数と同様にパターンを追加。

---

## コンパイラ変更（`compiler.rs`）

### `compile_expr` — `Expr::RecordUpdate` アーム（デシュガー方式）

```rust
// v61.4.0: { base | field: val } → IRExpr::RecordSpread（デシュガー）
Expr::RecordUpdate { base, fields, .. } => {
    let base_ir = compile_expr(base, ctx);
    let updates_ir: Vec<(String, IRExpr)> = fields
        .iter()
        .map(|(k, v)| (k.clone(), compile_expr(v, ctx)))
        .collect();
    IRExpr::RecordSpread(Box::new(base_ir), updates_ir, Type::Unknown)
}
```

### `collect_free_vars_expr` — `Expr::RecordUpdate` アーム（compiler.rs 内）

`RecordSpread` と同様:
```rust
Expr::RecordUpdate { base, fields, .. } => {
    collect_free_vars_expr(base, bound, free);
    for (_, v) in fields {
        collect_free_vars_expr(v, bound, free);
    }
}
```

---

## その他ファイルの変更

### `fmt.rs` — `Expr::RecordUpdate` フォーマット

```rust
Expr::RecordUpdate { base, fields, .. } => {
    let field_strs: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("{}: {}", k, self.expr(v)))
        .collect();
    if field_strs.is_empty() {
        format!("{{ {} | }}", self.expr(base))
    } else {
        format!("{{ {} | {} }}", self.expr(base), field_strs.join(", "))
    }
}
```

### `lint.rs` — 各 walk 関数に `Expr::RecordUpdate` アーム追加

lint.rs には `Expr` を walk する関数が複数あり、全て `RecordSpread` と同様のパターンを追加する:

```rust
Expr::RecordUpdate { base, fields, .. } => {
    // base を walk し、各フィールド値を walk する
    check_XXX(base, ...);
    for (_, v) in fields { check_XXX(v, ...); }
}
```

対象関数（`grep -n 'Expr::RecordSpread' fav/src/lint.rs` で確認した箇所と同数）:
- L251 付近の walk 関数
- L426 付近の walk 関数
- L589 付近の walk 関数
- L656 付近の walk 関数
- L862 付近の walk 関数
- L1009 付近の walk 関数
- L1337 付近の walk 関数
- L2336 付近の walk 関数
- L3042 付近の walk 関数

合計 **9 箇所**（5 ではない）。

### `emit_python.rs` — `Expr::RecordUpdate`

```rust
Expr::RecordUpdate { base, fields, .. } => {
    // { base | field: val } → {**base, "field": val}（RecordSpread と同様）
    let b = self.emit_expr(base);
    let mut parts = vec![format!("**{}", b)];
    for (k, v) in fields {
        let val = self.emit_expr(v);
        parts.push(format!("\"{}\": {}", k, val));
    }
    format!("{{{}}}", parts.join(", "))
}
```

### `ast_lower_checker.rs` — `Expr::RecordUpdate`

```rust
ast::Expr::RecordUpdate { base, fields, .. } => {
    v2("ERecordSpread", lower_expr(base), lower_field_list(fields))
}
```

---

## テスト仕様（`v61400_tests` 2 件）

テストモジュールの先頭:
```rust
#[cfg(test)]
mod v61400_tests {
    use super::*;
```

### `record_update_basic`

```rust
    /// { row | field: val } が parse + type-check を通過することを確認
    #[test]
    fn record_update_basic() {
    let src = concat!(
        "type Row = { status: String score: Int }\n",
        "fn update_row(row: Row) -> Row {\n",
        "  { row | status: \"active\" }\n",
        "}\n",
    );
    let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
    let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
    assert!(
        errors.is_empty(),
        "record update should type-check without errors; errors: {:?}",
        errors
    );
}
```

### `record_update_type_check`

```rust
/// 複数フィールドを同時に更新しても型チェックを通過することを確認
#[test]
fn record_update_type_check() {
    let src = concat!(
        "type Order = { total: Float currency: String }\n",
        "fn enrich(order: Order, qty: Float) -> Order {\n",
        "  { order | total: order.total * qty, currency: \"JPY\" }\n",
        "}\n",
    );
    let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
    let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
    assert!(
        errors.is_empty(),
        "multi-field record update should type-check without errors; errors: {:?}",
        errors
    );
}
```

---

## 完了条件

- `record_update_basic` pass
- `record_update_type_check` pass
- 総テスト数: **3363** tests passed, 0 failed（ベース 3361 + 2）
- 既存の RecordSpread テストが引き続き pass
- `fav fmt` で `{ row | status: "active" }` が正しくフォーマットされる
- CHANGELOG は v62.0 でまとめて記載のため本バージョンでの個別更新不要

---

## ベーステスト数の注意点

ロードマップ記載「ベース 3361 + 2 = 3363」は v61.3.0 code-reviewer 対応後の実績値を使用。
実際の v61.3.0 テスト数: **3361**（E0395 negative test 追加で +3）
実際のテスト数目標: **3361 + 2 = 3363** tests passed, 0 failed

---

## テスト数推移（参照用）

| バージョン | テスト数 | 備考 |
|---|---|---|
| v61.2.0 | 3358 | as-pattern 拡張（code-reviewer +1 で +3） |
| v61.3.0 | 3361 | OR パターンガード拡張（code-reviewer +1 で +3） |
| v61.4.0 | **3363** | record update 式 |
