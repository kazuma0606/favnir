# v61.4.0 Tasks — record update 式（`{ base | field: new_value }`）

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3361 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"61.0.0"` であることを確認
  - `grep '^version' fav/Cargo.toml` → `version = "61.0.0"`
- [x] `v61400_tests` がまだ存在しないことを確認
  - `grep -c 'v61400_tests' fav/src/driver.rs` = 0 件
- [x] `v61300_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v61300_tests' fav/src/driver.rs` ≥ 1 件
- [x] `Expr::RecordUpdate` がまだ存在しないことを確認
  - `grep -c 'RecordUpdate' fav/src/ast.rs` = 0 件
- [x] `RecordSpread` の現在の行番号を確認（挿入位置の把握）
  - `grep -n 'RecordSpread' fav/src/ast.rs`
- [x] `collect_free_vars_expr` が checker.rs に存在しないことを確認
  - `grep -c 'collect_free_vars_expr' fav/src/middle/checker.rs` = 0 件
- [x] lint.rs の `Expr::RecordSpread` が 9 箇所あることを確認
  - `grep -c 'Expr::RecordSpread' fav/src/lint.rs` = 9 件

---

## T1: `ast.rs` — `Expr::RecordUpdate` バリアント追加

`RecordSpread(Box<Expr>, Vec<(String, Expr)>, Span)` の直後に追加:

```rust
/// `{ base | field: val, ... }` — record update 式 (v61.4.0)
/// base の型の全フィールドを継承し、fields で指定したフィールドを上書きする。
RecordUpdate {
    base: Box<Expr>,
    fields: Vec<(String, Expr)>,
    span: Span,
},
```

`span()` メソッドの `Expr::RecordSpread(_, _, s) => s` の直後に追加:
```rust
Expr::RecordUpdate { span, .. } => span,
```

- [x] `Expr::RecordUpdate` バリアントを追加した
- [x] `span()` メソッドに `RecordUpdate` アームを追加した
- [x] `cargo build 2>&1 | grep "^error" | head -20` でエラー一覧を確認した

---

## T2: コンパイルエラー修正（タプル分解・exhaustive match 対応）

`cargo build` のエラーを基に、各ファイルの `Expr::RecordSpread` アームの直後に
`Expr::RecordUpdate` アームを追加する。T4（checker 本実装）前の段階では
スタブ（`check_expr(base); Type::Unknown` 等）で通過させてよい。

### T2-1: `checker.rs` — `check_expr` スタブ追加

```rust
// v61.4.0: record update 式（スタブ — T4 で本実装）
Expr::RecordUpdate { base, fields, .. } => {
    let base_ty = self.check_expr(base);
    for (_, v) in fields { self.check_expr(v); }
    base_ty
}
```

- [x] `check_expr` に `RecordUpdate` スタブを追加した

### T2-2: `checker.rs` — `collect_free_vars_expr`（存在しない）

`grep -c 'collect_free_vars_expr' fav/src/middle/checker.rs` = 0 件。変更不要。

- [x] `collect_free_vars_expr` は checker.rs に存在しないことを確認済み（スキップ）

### T2-3: `compiler.rs` — `compile_expr` デシュガー追加

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

- [x] `compile_expr` に `RecordUpdate` デシュガーを追加した

### T2-4: `compiler.rs` — `collect_free_vars_expr` 追加

```rust
Expr::RecordUpdate { base, fields, .. } => {
    collect_free_vars_expr(base, bound, free);
    for (_, v) in fields {
        collect_free_vars_expr(v, bound, free);
    }
}
```

- [x] `compiler.rs` の `collect_free_vars_expr` に `RecordUpdate` アームを追加した

### T2-5: `fmt.rs` — `RecordUpdate` フォーマット追加

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

- [x] `fmt.rs` に `RecordUpdate` フォーマットを追加した

### T2-6: `lint.rs` — 各 walk 関数に `RecordUpdate` アーム追加

`Expr::RecordSpread(base, updates, _)` がある行数を `grep -n 'Expr::RecordSpread' fav/src/lint.rs` で確認し、
各箇所の直後に `RecordUpdate` アームを追加する。

各箇所のパターン:
```rust
Expr::RecordUpdate { base, fields, .. } => {
    // RecordSpread と同様に base と各フィールド値を walk する
    // 対象関数の RecordSpread アームを参照して同じ処理を適用
}
```

- [x] lint.rs の全 `Expr::RecordSpread` アームの直後に `RecordUpdate` アームを追加した（計 **9 箇所**）
- [x] 追加した件数を確認した（`grep -c 'RecordUpdate' fav/src/lint.rs` = 9 件）

### T2-7: `emit_python.rs` — `RecordUpdate` Python エミット追加

```rust
Expr::RecordUpdate { base, fields, .. } => {
    // { base | field: val } → {**base, "field": val}
    let b = self.emit_expr(base);
    let mut parts = vec![format!("**{}", b)];
    for (k, v) in fields {
        let val = self.emit_expr(v);
        parts.push(format!("\"{}\": {}", k, val));
    }
    format!("{{{}}}", parts.join(", "))
}
```

- [x] `emit_python.rs` に `RecordUpdate` アームを追加した

### T2-8: `ast_lower_checker.rs` — `RecordUpdate` lower 追加

```rust
ast::Expr::RecordUpdate { base, fields, .. } => {
    v2("ERecordSpread", lower_expr(base), lower_field_list(fields))
}
```

- [x] `ast_lower_checker.rs` に `RecordUpdate` アームを追加した

### T2-9: コンパイル確認

- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T3: `parser.rs` — `parse_record_update` 追加 + `LBrace` アーム更新

### T3-1: `parse_record_update` 新規追加（`parse_record_spread` の直後）

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

- [x] `parse_record_update` を追加した
- [x] `cargo build` でコンパイルエラーがないことを確認した

### T3-2: `LBrace` アームに RecordUpdate 条件を追加

```rust
TokenKind::LBrace => {
    if self.peek2() == Some(&TokenKind::DotDotDot) {
        // RecordSpread（既存）
        let start_spread = self.peek_span().clone();
        self.advance(); // consume '{'
        self.parse_record_spread(start_spread)
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

- [x] `LBrace` アームに RecordUpdate 条件を追加した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T4: `checker.rs` — `check_expr` の RecordUpdate アームを本実装

T2-1 で追加したスタブを本実装に置き換える。

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

**実装時確認事項**:
- `self.lookup_field_type(&base_ty, fname)` を使う（L4593）。`Type::Named` 以外では `Type::Unknown` を返す。
- `unify` は standalone 関数（L370）。`self.unify` / `self.types_compatible` は存在しない。
- `self.type_defs` の値は `TypeBody`（`TypeDef` ではない）。`type_def.fields.get()` は使えない。

- [x] `check_expr` の `RecordUpdate` アームを本実装に置き換えた
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T5: `driver.rs` — `v61400_tests` モジュール追加

`v61300_tests` の直前（上側）に挿入する。

```rust
// -- v61400_tests (v61.4.0) -- record update 式 --
#[cfg(test)]
mod v61400_tests {
    use super::*;

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
}
```

- [x] `v61400_tests` モジュールを `v61300_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `record_update_basic` テストが含まれている
- [x] `record_update_type_check` テストが含まれている

---

## T6: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v61400_tests::record_update_basic` pass
- [x] `v61400_tests::record_update_type_check` pass
- [x] 既存 RecordSpread テストが引き続き pass
- [x] 総テスト数 **3363** tests passed, 0 failed を確認

---

## T7: 事後処理

- [x] `versions/current.md` を v61.4.0 / 3363 tests に更新
- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.4.0 実績欄を更新
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v62.0 でまとめて記載）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

code-reviewer 指摘（実装後）:
- [HIGH] E0102 エラーコード衝突 → E0396（型不一致）/ E0397（不存在フィールド）を新設して対応
- [HIGH] パーサーの base Ident 限定仕様をコメントで明文化
- [MED] 存在しないフィールドへの更新が E0397 を発火するよう checker 修正
- [MED] 空 fields（`{ row | }`）をパーサーでエラーに
- [MED] base_ty 返却の意図を checker にコメント追加
- [LOW] ネガティブテスト 2 件追加（E0396 / E0397）→ 合計テスト数 3365

spec-reviewer 指摘（実装前）:
- [HIGH-2] TypeBody API 修正: `lookup_field_type` + standalone `unify` を使用 → 対応済
- [HIGH-3] lint.rs 9 箇所（5 ではない）→ 9 箇所全部追加
- [HIGH-4] parse_expr の `|` 安全性確認 → `|` は中置演算子でないため安全（ドキュメント追記）
- [MED-5] checker.rs の `collect_free_vars_expr` 不存在確認 → スキップ
- [MED-7] `types_compatible` → `unify` に統一
- [MED-8] テストに `use super::*` 追加
- lineage.rs にも 5 箇所の exhaustive match 追加（tasks.md 外だが自動対応）
- driver.rs / lsp/references.rs にも追加が必要だった（ビルドエラー駆動）

---

Status: COMPLETE
