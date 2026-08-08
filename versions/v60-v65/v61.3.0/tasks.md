# v61.3.0 Tasks — パターンガード拡張（OR パターン各アームへの個別ガード）

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3358 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"61.0.0"` であることを確認
  - `grep '^version' fav/Cargo.toml` → `version = "61.0.0"`
  - （サブバージョン v61.x.x は Cargo.toml を更新しないため "61.0.0" のままであること）
- [x] `v61300_tests` がまだ存在しないことを確認
  - `grep -c 'v61300_tests' fav/src/driver.rs` = 0 件
- [x] `v61200_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v61200_tests' fav/src/driver.rs` ≥ 1 件
- [x] `Pattern::Or` の現在の型が `Vec<Pattern>` であることを確認
  - `grep -n 'Or(Vec<Pattern>' fav/src/ast.rs` ≥ 1 件
- [x] `IRPattern::Or` の現在の型が `Vec<IRPattern>` であることを確認
  - `grep -n 'Or(Vec<IRPattern>' fav/src/middle/ir.rs` ≥ 1 件
- [x] E0395 がまだ存在しないことを確認
  - `grep -c 'E0395' fav/src/error_catalog.rs` = 0 件

---

## T1: `ast.rs` — `Pattern::Or` 型変更

```rust
// 変更前
Or(Vec<Pattern>, Span),

// 変更後（v61.3.0: per-arm guard 付き OR パターン）
Or(Vec<(Pattern, Option<Expr>)>, Span),
```

- [x] `Pattern::Or` の型を変更した
- [x] `cargo build 2>&1 | grep error | head -30` でコンパイルエラー一覧を確認した

---

## T2: `ir.rs` — `IRPattern::Or` 型変更

```rust
// 変更前
Or(Vec<IRPattern>),

// 変更後
/// `(p1 if g1) | (p2 if g2)` — per-arm guarded or-pattern (v61.3.0)
Or(Vec<(IRPattern, Option<IRExpr>)>),
```

- [x] `IRPattern::Or` の型を変更した
- [x] `cargo build 2>&1 | grep error | head -30` でエラー数が増えていないことを確認した

---

## T3: タプル分解修正（コンパイルエラー駆動）

### T3-1: `checker.rs` — `collect_pattern_variants` の `Pattern::Or` アーム

```rust
// 変更後
Pattern::Or(arms, _) => {
    for (p, _) in arms { collect_pattern_variants(p, covered, has_catch_all); }
}
```

- [x] `collect_pattern_variants` の `Pattern::Or` アームを修正した

### T3-2: `compiler.rs` — `pattern_binds` の `Pattern::Or` アーム

```rust
// 変更後
Pattern::Or(arms, _) => {
    if let Some((first, _)) = arms.first() { pattern_binds(first, out); }
}
```

- [x] `pattern_binds` の `Pattern::Or` アームを修正した

### T3-3: `fmt.rs` — `Pattern::Or` フォーマット

```rust
// 変更後
Pattern::Or(arms, _) => {
    arms.iter().map(|(p, guard)| {
        let s = self.pattern(p);
        if let Some(g) = guard {
            format!("({} if {})", s, self.expr(g))
        } else {
            s
        }
    }).collect::<Vec<_>>().join(" | ")
}
```

- [x] `fmt.rs` の `Pattern::Or` フォーマットを修正した

### T3-4: `lint.rs` — `pattern_lit_keys_all` の `Pattern::Or` アーム

```rust
// 変更後
Pattern::Or(arms, _) => arms.iter().flat_map(|(p, _)| pattern_lit_keys_all(p)).collect(),
```

- [x] `pattern_lit_keys_all` の `Pattern::Or` アームを修正した

### T3-5: `lint.rs` — `collect_pattern_bound_names` の `Pattern::Or` アーム

```rust
// 変更後
Pattern::Or(arms, _) => arms.iter().flat_map(|(p, _)| collect_pattern_bound_names(p)).collect(),
```

- [x] `collect_pattern_bound_names` の `Pattern::Or` アームを修正した

### T3-6: `emit_python.rs` — `Pattern::Or` Python エミット

```rust
// 変更後
Pattern::Or(arms, _) => {
    if let Some((first, _)) = arms.first() {
        // guard は Python 側では無視（emit_python は型消去後の単純出力）
        // 既存の first パターン出力ロジックを使用
        ...
    }
}
```

- [x] `emit_python.rs` の `Pattern::Or` アームを修正した（`first` → `first.0` 等）

### T3-7: `ast_lower_checker.rs` — `Pattern::Or` の first alt 取得

```rust
// 変更後
ast::Pattern::Or(arms, _) => {
    if let Some((first, _)) = arms.first() {
        ast_lower_pattern(first, ctx)
    } else { ... }
}
```

- [x] `ast_lower_checker.rs` の `Pattern::Or` アームを修正した

### T3-8: `driver.rs` — `remap_ir_pattern` の `IRPattern::Or` アーム

```rust
// 変更後
IRPattern::Or(arms) => {
    // guard の IRExpr は remap 不要（シンボルテーブルに依存しない）
    IRPattern::Or(arms.iter().map(|(p, g)| (remap_ir_pattern(p), g.clone())).collect())
}
```

- [x] `driver.rs` の `remap_ir_pattern` の `IRPattern::Or` アームを修正した

### T3-9: コンパイル確認

- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T4: `parser.rs` — `parse_or_alternative` 追加 + `parse_match_arm` 更新

### T4-1: `parse_or_alternative` 新規追加

`parse_match_arm` または `parse_pattern` の付近に追加:

```rust
/// OR パターンの 1 アームを解析する。
/// `(pat if guard)` 形式 → (pat, Some(guard))
/// `pat` 形式（ガードなし）→ (pat, None)
fn parse_or_alternative(&mut self) -> Result<(Pattern, Option<Expr>), ParseError> {
    if self.peek() == &TokenKind::LParen {
        let start = self.peek_span().clone();
        self.advance(); // consume '('
        let pat = self.parse_pattern()?;
        if self.peek() == &TokenKind::If {
            // (pat if guard) — guarded OR alternative (v61.3.0)
            self.advance(); // consume 'if'
            let guard = self.parse_expr()?;
            self.expect(&TokenKind::RParen)?;
            Ok((pat, Some(guard)))
        } else if self.peek() == &TokenKind::Comma {
            // tuple pattern: fall back to tuple handling
            let mut fields = vec![PatternField::Alias("_0".to_string(), Box::new(pat), start.clone())];
            let mut i = 1usize;
            while self.peek() == &TokenKind::Comma {
                self.advance();
                if self.peek() == &TokenKind::RParen { break; }
                fields.push(PatternField::Alias(
                    format!("_{}", i),
                    Box::new(self.parse_pattern()?),
                    self.span_from(&start),
                ));
                i += 1;
            }
            self.expect(&TokenKind::RParen)?;
            Ok((Pattern::Record(fields, self.span_from(&start)), None))
        } else {
            // grouping (pat)
            self.expect(&TokenKind::RParen)?;
            Ok((pat, None))
        }
    } else {
        let pat = self.parse_pattern()?;
        Ok((pat, None))
    }
}
```

- [x] `parse_or_alternative` を追加した
- [x] `cargo build` でコンパイルエラーがないことを確認した

### T4-2: `parse_match_arm` 更新

```rust
// 変更後
let first_alt = self.parse_or_alternative()?;
let pattern = if self.peek() == &TokenKind::Pipe {
    let or_start = first_alt.0.span().clone();
    let mut alts = vec![first_alt];
    while self.peek() == &TokenKind::Pipe {
        self.advance();
        alts.push(self.parse_or_alternative()?);
    }
    Pattern::Or(alts, self.span_from(&or_start))
} else if first_alt.1.is_some() {
    // Single alt with guard (unusual but valid): wrap in Or
    let or_start = first_alt.0.span().clone();
    Pattern::Or(vec![first_alt], self.span_from(&or_start))
} else {
    first_alt.0
};
```

- [x] `parse_match_arm` を更新した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T5: `checker.rs` — `check_pattern_bindings` の `Pattern::Or` アーム更新

```rust
// v61.3.0: per-arm guard 対応。ガード式の型が Bool であることを検証。
Pattern::Or(arms, _) => {
    let mut bound: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (pat, guard) in arms {
        if let Pattern::Bind(name, _) = pat {
            if !bound.insert(name.clone()) { continue; }
        }
        self.check_pattern_bindings(pat, ty);
        // v61.3.0: ガード式の型を検証（Bool であること）
        if let Some(guard_expr) = guard {
            let guard_ty = self.infer_expr(guard_expr);
            if !matches!(guard_ty, Type::Bool | Type::Unknown) {
                let span = guard_expr.span();
                self.type_error(
                    "E0395",
                    &format!("or-pattern guard must be Bool, got {:?}", guard_ty),
                    span,
                );
            }
        }
    }
}
```

- [x] `check_pattern_bindings` の `Pattern::Or` アームを更新した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T6: `compiler.rs` — `compile_pattern` の `Pattern::Or` アーム更新

```rust
// v61.3.0: ガード式も IR にコンパイルして伝播
Pattern::Or(arms, _) => {
    IRPattern::Or(arms.iter().map(|(p, guard)| {
        let ir_pat = compile_pattern(p, ctx);
        let ir_guard = guard.as_ref().map(|g| compile_expr(g, ctx));
        (ir_pat, ir_guard)
    }).collect())
}
```

- [x] `compile_pattern` の `Pattern::Or` アームを更新した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T7: `codegen.rs` — ガード評価ロジック追加

`IRPattern::Or(arms)` のマッチアームで、ガード評価（`JumpIfFalse`）を追加する。

```rust
IRPattern::Or(arms) => {
    let mut or_success_jumps: Vec<usize> = Vec::new();
    for (i, (pat, guard)) in arms.iter().enumerate() {
        let is_last = i == arms.len() - 1;
        cg.emit_opcode(Opcode::Dup);
        let mut inner_fail: Vec<(usize, usize)> = Vec::new();
        let inner_depth = emit_pattern_test(pat, &mut inner_fail, cg, depth + 1);
        // SUCCESS PATH: pop extras back to depth
        for _ in (depth + 1)..=inner_depth {
            cg.emit_opcode(Opcode::Pop);
        }
        // v61.3.0: ガード評価（guard が Some の場合、Bool が false なら次アームへ）
        if let Some(guard_expr) = guard {
            emit_expr(guard_expr, cg);
            inner_fail.push((cg.emit_jump(Opcode::JumpIfFalse), 0));
        }
        if is_last {
            for item in inner_fail { fail_jumps.push(item); }
        } else {
            or_success_jumps.push(cg.emit_jump(Opcode::Jump));
            for (fail_jump, excess) in inner_fail {
                cg.patch_jump(fail_jump);
                for _ in 0..excess.saturating_sub(depth) {
                    cg.emit_opcode(Opcode::Pop);
                }
            }
        }
    }
    for j in or_success_jumps { cg.patch_jump(j); }
    depth
}
```

- [x] `codegen.rs` の `IRPattern::Or` アームにガード評価ロジックを追加した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T8: `error_catalog.rs` — E0395 追加

既存 ERROR_CATALOG 配列の末尾（`];` の直前）に追加
（E03xx 系最大値は E0384。E0394 は存在しないため「直後」ではなく末尾追記）:

```rust
ErrorEntry {
    code: "E0395",
    title: "or-pattern guard must be Bool",
    category: "type",
    description: "The guard expression in a per-arm OR pattern must have type Bool.",
    example: "(x if x + 1) | _ => ...",
    fix: "Ensure the guard expression evaluates to Bool.",
    long_description: None,
    suggestion: Some("Use a comparison or logical expression (e.g., `x > 0`) as the guard."),
},
```

- [x] `error_catalog.rs` に E0395 エントリを追加した
- [x] `cargo build` でコンパイルエラーがないことを確認した

---

## T9: `driver.rs` — `v61300_tests` モジュール追加

`v61200_tests` の直前（上側）に挿入する。

```rust
// -- v61300_tests (v61.3.0) -- パターンガード拡張 --
#[cfg(test)]
mod v61300_tests {
    use super::*;

    /// OR パターンの各アームに個別ガードを付与してもエラーにならないことを確認
    #[test]
    fn guard_or_pattern_per_arm() {
        let src = concat!(
            "fn classify(x: Int) -> String {\n",
            "  match x {\n",
            "    (y if y > 90) | (y if y > 50) => \"high\"\n",
            "    _ => \"low\"\n",
            "  }\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "per-arm guards in OR pattern should type-check without errors; errors: {:?}",
            errors
        );
    }

    /// 3 アーム OR パターン（各アームに個別ガード）が型チェックを通過することを確認。
    /// ガードなしのフォールスルー（`_`）と組み合わせたケース。
    #[test]
    fn guard_or_pattern_fallthrough() {
        let src = concat!(
            "fn route(status: String) -> String {\n",
            "  match status {\n",
            "    (\"active\" if true) | (\"pending\" if false) | _ => \"matched\"\n",
            "  }\n",
            "}\n",
        );
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let (errors, _) = crate::middle::checker::Checker::check_program(&prog);
        assert!(
            errors.is_empty(),
            "3-arm guarded OR with wildcard fallthrough should type-check; errors: {:?}",
            errors
        );
    }
}
```

- [x] `v61300_tests` モジュールを `v61200_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `guard_or_pattern_per_arm` テストが含まれている
- [x] `guard_or_pattern_fallthrough` テストが含まれている

---

## T10: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v61300_tests::guard_or_pattern_per_arm` pass
- [x] `v61300_tests::guard_or_pattern_fallthrough` pass
- [x] 既存 `v61100_tests` の OR パターンテストが引き続き pass
- [x] 総テスト数 **3360** tests passed, 0 failed を確認

---

## T11: 事後処理

- [x] `versions/current.md` を v61.3.0 / 3360 tests に更新
- [x] `versions/roadmap/roadmap-v61.1-v62.0.md` の v61.3.0 実績欄を更新
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v62.0 でまとめて記載）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

### 実装中の修正（1件）

- **[BUG] `self.infer_expr` が存在しない**:
  `Checker` の式型チェックメソッドは `infer_expr` ではなく `check_expr`。
  spec の擬似コードの名前誤りを修正。

### code-reviewer 指摘と対応（7件）

- **[HIGH] `continue` が2アーム目以降のガード型チェックをスキップするバグ**:
  `skip_bindings` フラグを導入し、ガード型チェックは `skip_bindings` に関わらず常に実行するよう修正。
  → `checker.rs` の `check_pattern_bindings` を修正。
- **[HIGH] `collect_free_vars_expr` が OR アームガード内の自由変数を収集しない**:
  `Expr::Match` ハンドラに OR アームガードの自由変数収集ロジックを追加。
  各 OR アームの `(or_pat, Some(guard))` に対し `inner_bound` を作成して収集。
  → `compiler.rs` を修正。
- **[MED] `remap_ir_pattern` の OR guard が global index をリマップしない**:
  `remap_ir_pattern` シグネチャに `global_idx_map` を追加し、guard の IRExpr も `remap_ir_expr` でリマップ。
  → `driver.rs` を修正。
- **[MED] codegen.rs の Bind スロット汚染（設計意図未記載）**:
  コメントを追加し、OR アームが同 slot を再利用する設計意図を明記。
- **[MED] emit_python.rs がガードを無視（Known limitation 未記載）**:
  Known limitation コメントを追加し、fav2py ユーザーへの注意を明記。
- **[MED] E0395 negative test 不足**:
  `guard_or_pattern_e0395_non_bool_guard` テストを追加（`y + 1` が Int のとき E0395 発火確認）。
  テスト追加により 3360 → 3361 tests に増加。
- **[LOW] `ast_lower_checker.rs` が全アームを lower しない（既存限界）**:
  Known limitation コメントを追加して設計意図を明記。

### spec-reviewer 指摘対応（実装前に修正済み）

- **[HIGH] ロードマップ `vm.rs` 記述**: `codegen.rs` に実装（vm.rs にパターン処理コードなし）
- **[HIGH] テスト数 3359 → 3360**: ロードマップ・推移表を一括補正
- **[HIGH] `parse_or_alternative` タプル再構築**: 既存 `parse_pattern` の `(p1, p2)` ロジックと同一スキームを確認し採用
- **[HIGH] JumpIfFalse consume 仕様**: spec に明記（JumpIfFalse は Bool を pop して評価）
- **[MED] E0394 前提誤り**: ERROR_CATALOG 末尾追記に変更（E0384 が実際の最大値）
- **[MED] long_description 必須**: `explain_error_all_codes_have_long_desc` テスト失敗で発覚、追加して修正
- その他 MED/LOW 指摘: spec.md 完了条件に追記

---

Status: COMPLETE
