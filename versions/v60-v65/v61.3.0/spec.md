# v61.3.0 Spec — パターンガード拡張（OR パターン各アームへの個別ガード）

Date: 2026-07-31
Status: COMPLETE

---

## 概要

OR パターンの各アームに独立したガードを付与できる構文を追加する。

```favnir
match row {
  ("active" if score > 90) | ("pending" if score > 50) => process(row)
  _ => skip(row)
}
```

従来の OR パターン `"a" | "b"` は後方互換で動作する（ガード = None）。

---

## vm.rs に関する補足

ロードマップには「vm.rs でアーム別ガード評価ロジックを実装」と記載されているが、
Favnir の OR パターンマッチングは `backend/codegen.rs` が生成するバイトコードで完結しており、
`vm.rs` に直接のパターン処理コードは存在しない（確認済み）。
本バージョンではガード評価を `codegen.rs` に実装し、`vm.rs` は変更しない。

---

## 実装スコープ（11 ファイル）

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/ast.rs` | 更新 | `Pattern::Or(Vec<Pattern>, Span)` → `Pattern::Or(Vec<(Pattern, Option<Expr>)>, Span)` |
| `fav/src/frontend/parser.rs` | 更新 | `parse_or_alternative()` 追加、`parse_match_arm()` 更新 |
| `fav/src/middle/checker.rs` | 更新 | check_pattern_bindings + collect_pattern_variants の 2 箇所 |
| `fav/src/middle/compiler.rs` | 更新 | pattern_binds + compile_pattern の 2 箇所 |
| `fav/src/fmt.rs` | 更新 | `Pattern::Or` フォーマット（guard あり: `(pat if expr)` 形式） |
| `fav/src/lint.rs` | 更新 | `pattern_lit_keys_all` + `collect_pattern_bound_names` の 2 箇所 |
| `fav/src/emit_python.rs` | 更新 | `Pattern::Or` Python エミット（first alt の guard を無視） |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | `Pattern::Or` の first alt 取得 |
| `fav/src/middle/ir.rs` | 更新 | `IRPattern::Or(Vec<IRPattern>)` → `IRPattern::Or(Vec<(IRPattern, Option<IRExpr>)>)` |
| `fav/src/backend/codegen.rs` | 更新 | OR パターン各アームの guard 評価（JumpIfFalse で次アームへ） |
| `fav/src/driver.rs` | 更新 | `remap_ir_pattern` の `IRPattern::Or` アーム |

新規ファイルなし。`Cargo.toml` バージョン変更なし（サブバージョン）。

---

## AST 変更

### `ast.rs`

```rust
// 変更前
Or(Vec<Pattern>, Span),

// 変更後（v61.3.0: per-arm guard 付き OR パターン）
Or(Vec<(Pattern, Option<Expr>)>, Span),
```

既存の `Pattern::Or(pats, span)` 分解は `Pattern::Or(arms, span)` に変わり、
各 arm は `(pat, Option<Expr>)` タプルとなる。

### `ir.rs`

```rust
// 変更前
Or(Vec<IRPattern>),

// 変更後
/// `(p1 if g1) | (p2 if g2)` — per-arm guarded or-pattern (v61.3.0)
Or(Vec<(IRPattern, Option<IRExpr>)>),
```

---

## パーサー変更（`parser.rs`）

### 新規: `parse_or_alternative`

1 つの OR アームを `(Pattern, Option<Expr>)` として返す。

**設計注**: `(` を先に consume してから `if` or `,` を判定する方針を採用する。
既存の `parse_pattern()` はタプルを自己完結で解析できるが、`(pat if guard)` 形式を自然に区別するために
`parse_or_alternative` で `(` を先読みして分岐する設計とした。
タプルパターンのフィールド命名規則（`_0`, `_1`, ...）は既存の Tuple パターン実装と統一する必要がある。
**実装時確認事項**: `parser.rs` の既存タプルパターン実装を確認し、`PatternField::Alias("_0", ...)` スキームが正しいかを `cargo build` でコンパイルエラーがないことで検証する。

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
            // re-enter tuple parsing using already-consumed '(' and first pat
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

### 変更: `parse_match_arm`

OR パターンのループで `parse_or_alternative` を使用する。

```rust
// 変更前
let first_pat = self.parse_pattern()?;
let pattern = if self.peek() == &TokenKind::Pipe {
    let or_start = first_pat.span().clone();
    let mut pats = vec![first_pat];
    while self.peek() == &TokenKind::Pipe {
        self.advance();
        pats.push(self.parse_pattern()?);
    }
    Pattern::Or(pats, self.span_from(&or_start))
} else {
    first_pat
};

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

---

## チェッカー変更（`checker.rs`）

### `check_pattern_bindings` — `Pattern::Or` アーム（L4212 付近）

```rust
// 変更前
Pattern::Or(pats, _) => {
    let mut bound = HashSet::new();
    for pat in pats {
        if let Pattern::Bind(name, _) = pat { ... }
        self.check_pattern_bindings(pat, ty);
    }
}

// 変更後
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

**注**: `E0395` は新規エラーコード（`error_catalog.rs` の E03xx 系最大値は E0384。E0395 は既存コードと衝突しないため新規追加）。`error_catalog.rs` の既存エントリ末尾（`];` 直前）に追加する。

### `collect_pattern_variants` — `Pattern::Or` アーム（L10412 付近）

```rust
// 変更前
Pattern::Or(pats, _) => {
    for p in pats { collect_pattern_variants(p, covered, has_catch_all); }
}

// 変更後
Pattern::Or(arms, _) => {
    for (p, _) in arms { collect_pattern_variants(p, covered, has_catch_all); }
}
```

---

## コンパイラ変更（`compiler.rs`）

### `pattern_binds`（L1872 付近）

```rust
// 変更前
Pattern::Or(pats, _) => {
    if let Some(first) = pats.first() { pattern_binds(first, out); }
}

// 変更後
Pattern::Or(arms, _) => {
    if let Some((first, _)) = arms.first() { pattern_binds(first, out); }
}
```

### `compile_pattern`（L2764 付近）

```rust
// 変更前
Pattern::Or(pats, _) => {
    IRPattern::Or(pats.iter().map(|p| compile_pattern(p, ctx)).collect())
}

// 変更後
// v61.3.0: ガード式も IR にコンパイルして伝播
Pattern::Or(arms, _) => {
    IRPattern::Or(arms.iter().map(|(p, guard)| {
        let ir_pat = compile_pattern(p, ctx);
        let ir_guard = guard.as_ref().map(|g| compile_expr(g, ctx));
        (ir_pat, ir_guard)
    }).collect())
}
```

---

## その他ファイルの変更（タプル分解のみ）

### `fmt.rs`（L838 付近）

```rust
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

### `lint.rs` — `pattern_lit_keys_all`（L3087 付近）

```rust
Pattern::Or(arms, _) => arms.iter().flat_map(|(p, _)| pattern_lit_keys_all(p)).collect(),
```

### `lint.rs` — `collect_pattern_bound_names`（L3143 付近）

```rust
Pattern::Or(arms, _) => arms.iter().flat_map(|(p, _)| collect_pattern_bound_names(p)).collect(),
```

### `emit_python.rs`（L1092 付近）

```rust
Pattern::Or(arms, _) => {
    if let Some((first, _)) = arms.first() {
        // guard は Python 側では無視（emit_python は型消去後の単純出力）
        ...
    }
}
```

### `ast_lower_checker.rs`（L121 付近）

```rust
ast::Pattern::Or(arms, _) => {
    if let Some((first, _)) = arms.first() {
        ast_lower_pattern(first, ctx)
    } else { ... }
}
```

---

## コードゲン変更（`codegen.rs`）

**前提確認**: Favnir VM の `JumpIfFalse` opcode はスタック top の Bool 値を **pop（消費）** してからジャンプ判定を行う（`vm.rs` で確認済み）。ガード評価後に追加の Pop は不要。

`IRPattern::Or(arms)` のループで、各アームの guard を評価する。

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

---

## `driver.rs` — `remap_ir_pattern`

```rust
// 変更前
IRPattern::Or(pats) => {
    IRPattern::Or(pats.iter().map(remap_ir_pattern).collect())
}

// 変更後
IRPattern::Or(arms) => {
    // guard の IRExpr は remap 不要（シンボルテーブルに依存しない）
    IRPattern::Or(arms.iter().map(|(p, g)| (remap_ir_pattern(p), g.clone())).collect())
}
```

---

## `error_catalog.rs` への E0395 追加

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

---

## テスト仕様（`v61300_tests` 2 件）

### `guard_or_pattern_per_arm`

```rust
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
```

### `guard_or_pattern_fallthrough`

```rust
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
```

---

## 完了条件

- `guard_or_pattern_per_arm` pass
- `guard_or_pattern_fallthrough` pass
- 総テスト数: **3360** tests passed, 0 failed（ベース 3358 + 2）
- 既存の OR パターンテスト（`v61100_tests::pattern_or_type_check_arms_same` 等）が引き続き pass
- `fmt.rs` でガード付き OR パターンが `(pat if expr) | (pat if expr)` 形式でフォーマットされる
- ガードなし OR パターンのフォーマット出力が変化しないこと（後方互換確認）
- `E0395` が `error_catalog.rs` に登録されている
- ガード付き OR + ワイルドカード組み合わせで exhaustiveness 警告が出ないこと（`_` が catch-all として認識される）
- ガード式がパターン変数（`y > 90` の `y` 等）を参照できること（`check_pattern_bindings` が先にバインディングをスコープ追加するため）
- CHANGELOG は v62.0 でまとめて記載のため本バージョンでの個別更新不要

---

## ベーステスト数の注意点

ロードマップ記載「ベース 3357 + 2 = 3359」は v61.2.0 code-reviewer 対応前の想定値。
実際の v61.2.0 テスト数: **3358**（W039 positive test 追加で +1）
実際のテスト数目標: **3358 + 2 = 3360** tests passed, 0 failed

---

## テスト数推移（参照用）

| バージョン | テスト数 | 備考 |
|---|---|---|
| v61.1.0 | 3355 | OR パターン強化 |
| v61.2.0 | 3358 | as-pattern 拡張（code-reviewer +1 で +3） |
| v61.3.0 | **3360** | パターンガード拡張 |
