# v61.4.0 Plan — record update 式（`{ base | field: new_value }`）

Date: 2026-07-31
Status: REVIEWED

---

## 実装順序

AST 追加 → パーサー → コンパイルエラー駆動で全 Expr match 箇所を修正 → checker の型検証 → テスト

---

## Phase 1: AST 追加（`ast.rs`）

### P1: `Expr::RecordUpdate` バリアント追加

`RecordSpread` の直後に追加:
```rust
RecordUpdate {
    base: Box<Expr>,
    fields: Vec<(String, Expr)>,
    span: Span,
},
```

`span()` メソッドの match にも追加:
```rust
Expr::RecordUpdate { span, .. } => span,
```

この変更でコンパイルエラーが多数発生 → Phase 2 以降で解消。

---

## Phase 2: コンパイルエラー修正（各 Expr match 箇所）

`cargo build` でエラー一覧を確認後、exhaustive match が壊れた箇所を順次修正。
基本方針: `RecordSpread` アームを参考に `RecordUpdate` アームを同じ位置に追加。

修正対象（予想）:

| ファイル | 関数 | 修正内容 |
|---|---|---|
| `checker.rs` | `check_expr` | RecordUpdate 型チェック（P4 で詳細実装）|
| `checker.rs` | `collect_free_vars_expr` | **存在しない** — 変更不要 |
| `compiler.rs` | `compile_expr` | デシュガー → IRExpr::RecordSpread |
| `compiler.rs` | `collect_free_vars_expr` | base + fields を walk |
| `fmt.rs` | `expr` | `{ base \| field: val }` 形式 |
| `lint.rs` | 複数の walk 関数（**9箇所**） | base + fields を walk |
| `emit_python.rs` | `emit_expr` | `{**base, "field": val}` |
| `ast_lower_checker.rs` | `lower_expr` | ERecordSpread として lower |
| `driver.rs` | `remap_ir_expr` ※ | 既存 RecordSpread と同様 |

※ `driver.rs` の `remap_ir_expr` は IRExpr を対象とするため変更不要の可能性がある。ビルドエラーで確認。
`checker.rs` の `collect_free_vars_expr` は存在しないため変更不要。

---

## Phase 3: パーサー追加（`parser.rs`）

### P3-1: `parse_record_update` 新規追加

`parse_record_spread` の直後に追加:
```rust
fn parse_record_update(&mut self, start: Span) -> Result<Expr, ParseError>
```

### P3-2: `LBrace` アームに条件追加

```rust
TokenKind::LBrace => {
    if self.peek2() == Some(&TokenKind::DotDotDot) {
        // RecordSpread（既存）
    } else if matches!(self.peek2(), Some(TokenKind::Ident(_)))
        && self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::Pipe)
    {
        // v61.4.0: RecordUpdate
        let start = self.peek_span().clone();
        self.advance(); // consume '{'
        self.parse_record_update(start)
    } else {
        Ok(Expr::Block(Box::new(self.parse_block()?)))
    }
}
```

---

## Phase 4: checker の型検証強化（`checker.rs`）

### P4: `check_expr` の RecordUpdate アームを本実装

Phase 2 で追加したスタブ（`check_expr(base)` + `Type::Unknown`）を、
base 型の Named 型チェック + フィールド型検証付きの実装に置き換える。

**型検証実装方針**:
- `self.lookup_field_type(&base_ty, fname)` でフィールド型を取得（L4593 メソッド）
- `Type::Unknown` が返った場合（フィールド不明）: 型チェックをスキップ
- それ以外: standalone `unify(&val_ty, &expected_ty)` で互換チェック（L370 関数）
- `self.types_compatible` は存在しない。`self.unify` も存在しない。
- `self.type_defs` の値は `TypeBody`（`TypeDef` ではない）— `type_def.fields.get()` は使えない
- `collect_free_vars_expr` は checker.rs に存在しない（変更不要）
- 返り値は常に `base_ty`（base の型）

---

## Phase 5: テスト追加（`driver.rs`）

### P5: `v61400_tests` モジュール追加

`v61300_tests` の直前（上側）に挿入:
- `record_update_basic`: `{ row | status: "active" }` が型チェックを通過
- `record_update_type_check`: 複数フィールド更新が型チェックを通過

---

## Phase 6: テスト実行・確認

- `cargo test -j 8 -- --test-threads=8`
- `record_update_basic` pass
- `record_update_type_check` pass
- 総テスト数 **3363** tests passed, 0 failed
- 既存 RecordSpread テストが引き続き pass

---

## Phase 7: 事後処理

- `versions/current.md` を v61.4.0 / 3363 tests に更新
- `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.4.0 実績欄を更新
- このファイル（plan.md）と tasks.md を COMPLETE に更新

---

## リスク・注意事項

1. **`{ x | y }` 形式の誤検知**: ident + Pipe の検出は RecordUpdate として解釈するが、
   Favnir の `|` は中置演算子ではない（`||` = `PipePipe`、`|>` = `PipeGt`）ため
   `parse_expr()` が `|` を消費することはなく、`expect(Pipe)` は正しく機能する。
2. **checker.rs の実装**: `lookup_field_type` + standalone `unify` を使う。
   `types_compatible`・`self.unify` は存在しない。`type_defs` の値は `TypeBody`。
3. **lint.rs の `Expr::RecordSpread` は 9 箇所**: L251, L426, L589, L656, L862, L1009, L1337, L2336, L3042。
   全箇所に `RecordUpdate` アームを追加すること。
4. **`driver.rs` の `remap_ir_expr`**: `Expr::RecordUpdate` が AST レベルの関数の場合、
   `remap_ir_expr` は IRExpr を対象とするため変更不要の可能性がある。ビルドエラーで確認。
