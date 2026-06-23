# v24.4.0 実装計画 — v1.0 後方互換性ポリシー確定

## 前提確認

```bash
grep -n "version = " fav/Cargo.toml
# → "24.3.0" であること

grep -n "mod v244000_tests" fav/src/driver.rs | head -3
# → 0 件であること

grep -n "deprecated: bool\|deprecated_ann\|W020\|check_w020" fav/src/ast.rs fav/src/frontend/parser.rs fav/src/lint.rs | head -5
# → 全 0 件であること
```

---

## T0: `fav/src/ast.rs` — `FnDef.deprecated` 追加

`FnDef` 構造体に `deprecated: bool` フィールドを追加する。

```rust
pub struct FnDef {
    pub visibility: Option<Visibility>,
    pub is_async: bool,
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_ty: Option<TypeExpr>,
    pub effects: Vec<Effect>,
    pub body: Block,
    pub span: Span,
    pub api_annotation: Option<ApiAnnotation>,
    /// v24.4.0: `#[deprecated]` アノテーション付き関数
    pub deprecated: bool,
}
```

**FnDef を構築している箇所すべてに `deprecated: false` を追加する。**

探し方（全ソースファイルを対象にすること）:
```bash
grep -rn "FnDef {" fav/src/ | head -30
```

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0（FnDef 構築漏れがないこと）

---

## T1: `fav/src/frontend/parser.rs` — `parse_deprecated_annotation()` 追加

### T1-1: `parse_deprecated_annotation` 関数

```rust
/// v24.4.0: `#[deprecated]` を認識して bool を返す。
fn parse_deprecated_annotation(&mut self) -> Result<bool, ParseError> {
    if self.peek() == &TokenKind::Hash
        && matches!(self.tokens.get(self.pos + 1), Some(t) if t.kind == TokenKind::LBracket)
        && matches!(self.tokens.get(self.pos + 2), Some(t) if t.kind == TokenKind::Ident && t.text == "deprecated")
        && matches!(self.tokens.get(self.pos + 3), Some(t) if t.kind == TokenKind::RBracket)
    {
        self.advance(); // #
        self.advance(); // [
        self.advance(); // deprecated
        self.advance(); // ]
        Ok(true)
    } else {
        Ok(false)
    }
}
```

### T1-2: `parse_item()` に組み込み

`parse_item()` の先頭（既存アノテーション解析の直前）に追加:

```rust
// v24.4.0: parse optional #[deprecated] annotation before fn
let deprecated_ann = self.parse_deprecated_annotation()?;
```

`fn` を parse した後に `fd.deprecated = deprecated_ann;` を設定:

```rust
TokenKind::Fn => {
    let mut fd = self.parse_fn_def(vis, false)?;
    fd.deprecated = deprecated_ann;
    fd.api_annotation = api_annotation;
    return Ok(Item::FnDef(fd));
}
// async fn アームにも同様に追加
```

> **注意**: `impl` ブロック内の `fn`（`parse_impl_def` 内の `parse_fn_def`）は `parse_item()` を通らないため `deprecated` は `false` のまま。`impl` 内 deprecated は v24.7+ で対応予定。

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T2: `fav/src/lint.rs` — W020 追加

### T2-1: `collect_deprecated_calls_in_expr` / `collect_deprecated_calls_in_block`

```rust
fn collect_deprecated_calls_in_expr(
    expr: &Expr,
    deprecated: &std::collections::HashSet<String>,
    errors: &mut Vec<LintError>,
) {
    match expr {
        // Expr::Apply(func, args, span) — 関数呼び出し（FnCall ではない）
        Expr::Apply(func, args, span) => {
            if let Expr::Ident(name, _) = func.as_ref() {
                if deprecated.contains(name) {
                    errors.push(LintError::new(
                        "W020",
                        format!("call to deprecated function `{name}`"),
                        span.clone(),
                    ));
                }
            }
            collect_deprecated_calls_in_expr(func, deprecated, errors);
            for a in args {
                collect_deprecated_calls_in_expr(a, deprecated, errors);
            }
        }
        // Expr::If(cond, then, else_, span)
        Expr::If(cond, then, else_, _) => {
            collect_deprecated_calls_in_expr(cond, deprecated, errors);
            collect_deprecated_calls_in_block(then, deprecated, errors);
            if let Some(e) = else_ {
                collect_deprecated_calls_in_block(e, deprecated, errors);
            }
        }
        // Expr::Match(subject, arms, span) — MatchArm.body は Expr（Block ではない）
        Expr::Match(subject, arms, _) => {
            collect_deprecated_calls_in_expr(subject, deprecated, errors);
            for arm in arms {
                collect_deprecated_calls_in_expr(&arm.body, deprecated, errors);
            }
        }
        Expr::Block(b) => collect_deprecated_calls_in_block(b, deprecated, errors),
        _ => {}
    }
}

fn collect_deprecated_calls_in_block(
    block: &Block,
    deprecated: &std::collections::HashSet<String>,
    errors: &mut Vec<LintError>,
) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b)  => collect_deprecated_calls_in_expr(&b.expr, deprecated, errors),
            Stmt::Chain(c) => collect_deprecated_calls_in_expr(&c.expr, deprecated, errors),
            Stmt::Expr(e)  => collect_deprecated_calls_in_expr(e, deprecated, errors),
            Stmt::Yield(y) => collect_deprecated_calls_in_expr(&y.expr, deprecated, errors),
            _ => {}
        }
    }
    collect_deprecated_calls_in_expr(&block.expr, deprecated, errors);
}
```

### T2-2: `check_w020_deprecated_call`

既存の `check_w01x_*` と同じ **2 引数シグネチャ** (`program: &Program, errors: &mut Vec<LintError>`) を使う。

```rust
// ── W020: deprecated_call (v24.4.0) ────────────────────────────────────────────
pub fn check_w020_deprecated_call(program: &Program, errors: &mut Vec<LintError>) {
    use std::collections::HashSet;
    let deprecated: HashSet<String> = program.items.iter()
        .filter_map(|item| {
            if let Item::FnDef(fd) = item {
                if fd.deprecated { Some(fd.name.clone()) } else { None }
            } else {
                None
            }
        })
        .collect();
    if deprecated.is_empty() {
        return;
    }
    for item in &program.items {
        match item {
            Item::FnDef(fd) => {
                collect_deprecated_calls_in_block(&fd.body, &deprecated, errors);
            }
            Item::TrfDef(td) => {
                collect_deprecated_calls_in_block(&td.body, &deprecated, errors);
            }
            _ => {}
        }
    }
}
```

### T2-3: `lint_program()` に組み込み

`check_w019_string_concat_chain` の直後に追加:

```rust
// v24.4.0: W020
check_w020_deprecated_call(program, &mut errors);
```

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T3: `fav/src/driver.rs` — v244000_tests 追加

### T3-1: `v243000_tests::version_is_24_3_0` を削除

```rust
#[test]
fn version_is_24_3_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(
        cargo.contains("version = \"24.3.0\""),
        "Cargo.toml should have version 24.3.0"
    );
}
```

この関数ごと削除する。

### T3-2: `v244000_tests` モジュールを `v243000_tests` の直後に追加

```rust
// ── v244000_tests (v24.4.0) — v1.0 後方互換性ポリシー確定 ────────────────────
#[cfg(test)]
mod v244000_tests {
    use super::*;

    #[test]
    fn version_is_24_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"24.4.0\""),
            "Cargo.toml should have version 24.4.0"
        );
    }

    #[test]
    fn deprecated_fn_annotation_parsed() {
        let src = "#[deprecated]\nfn old_func(x: Int) -> Int { x + 1 }";
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        let fn_def = prog.items.iter().find_map(|i| {
            if let crate::ast::Item::FnDef(fd) = i { Some(fd) } else { None }
        }).expect("FnDef not found");
        assert!(fn_def.deprecated, "fn should be marked deprecated");
    }

    #[test]
    fn deprecated_call_emits_w020() {
        let src = r#"
            #[deprecated]
            fn old_func(x: Int) -> Int { x + 1 }

            fn new_func(x: Int) -> Int { old_func(x) }
        "#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        let mut warnings = Vec::new();
        crate::lint::check_w020_deprecated_call(&prog, &mut warnings);
        assert_eq!(warnings.len(), 1, "expected 1 W020 warning, got: {warnings:?}");
        assert!(
            warnings[0].message.contains("old_func"),
            "warning should mention old_func: {:?}",
            warnings[0]
        );
    }

    #[test]
    fn stability_md_has_policy() {
        let md = include_str!("../../STABILITY.md");
        assert!(md.contains("v1.x"), "STABILITY.md should mention v1.x policy");
        assert!(md.contains("v2.0"), "STABILITY.md should mention v2.0 policy");
    }

    #[test]
    fn changelog_has_v24_4_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(
            cl.contains("[v24.4.0]"),
            "CHANGELOG.md should have [v24.4.0] entry"
        );
    }
}
```

- [ ] `cargo test v244000 --bin fav` — 5/5 PASS を確認
- [ ] `cargo test --bin fav` — リグレッションなし（1948 件合格）を確認

---

## T4: `STABILITY.md` 作成

トップレベル（`C:\Users\yoshi\favnir\STABILITY.md`）に作成。

```markdown
# Favnir Stability Policy

## v1.x 後方互換性保証

v1.x（マイナーバージョン）では破壊的変更を行わない。
パッチバージョン（v1.x.y+1）はバグ修正のみ。

## v2.0 破壊的変更ポリシー

破壊的変更は **2 年前から** `#[deprecated]` アノテーションで事前警告する。
`#[deprecated]` 付き API は `fav lint` で W020 警告が表示される。
削除予定は CHANGELOG.md と DEPRECATIONS.md に記録する。

## SemVer 準拠

Favnir は Semantic Versioning 2.0.0 に完全準拠する。
（major.minor.patch — major 変更時のみ後方互換性破壊を許可）

## `--legacy` フラグ

`--legacy` フラグは v2.0 まで維持する（v1.x では削除しない）。
v1.x ユーザーは `--legacy` で旧挙動にアクセスできる。
```

---

## T5: Cargo.toml + CHANGELOG + benchmarks

> **注意**: T3-1 の `version_is_24_3_0` 削除完了後に Cargo.toml を更新すること（T5-1）。

### T5-1: `fav/Cargo.toml` バージョン更新

```
version = "24.3.0" → "24.4.0"
```

### T5-2: `CHANGELOG.md` 先頭に v24.4.0 エントリ追加

```markdown
## [v24.4.0] — 2026-06-23 — v1.0 後方互換性ポリシー確定

### Added
- `#[deprecated]` アノテーション — `fn` 定義に付与することで廃止予定を宣言できる
- W020 `deprecated_call` lint ルール — `#[deprecated]` 付き関数の呼び出しを検出
- `STABILITY.md` — v1.x 後方互換ポリシー・v2.0 破壊的変更ポリシー・SemVer 準拠宣言

### Notes
- `#[deprecated]` は `fn` にのみ対応（`trf`・`flw` は v24.7+ 予定）
- `impl` ブロック内 `fn` への `#[deprecated]` は v24.7+ 予定
- `--legacy` フラグは v2.0 まで維持（ STABILITY.md 参照）
```

### T5-3: `benchmarks/v24.4.0.json` 作成

```json
{
  "version": "24.4.0",
  "date": "2026-06-23",
  "test_count": 1948,
  "feature": "v1.0 後方互換性ポリシー確定",
  "metrics": {
    "test_count": 1948,
    "duration_ms": 16700
  }
}
```

> **注意**: `duration_ms: 16700` は推定値。実装完了後に `cargo test --bin fav` の実測値（秒数 × 1000）に置き換えること。

---

## 実装順序

```
T0（ast.rs: FnDef.deprecated フィールド追加）
T1（parser.rs: parse_deprecated_annotation 追加 + parse_item 組み込み）
T2（lint.rs: check_w020_deprecated_call 追加 + lint_program 組み込み）
cargo check → エラー 0 確認
T3-1（version_is_24_3_0 削除）← T5-1 より前に必須
T3-2（v244000_tests 追加）
cargo test v244000 → 5/5 PASS 確認
T4（STABILITY.md 作成）
T5-1（version 更新）← T3-1 完了後
T5-2〜3（CHANGELOG / v24.4.0.json）
cargo test --bin fav → リグレッションなし確認（1948 件）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `FnDef` 構築箇所に `deprecated: false` を追加し忘れる | `cargo check` でコンパイルエラー | エラーが出た全箇所に `deprecated: false` を追加 |
| `parse_deprecated_annotation` の token 先読みインデックスが他のアノテーションとバッティング | `deprecated_fn_annotation_parsed` テスト失敗 | `parse_deprecated_annotation` は最初に呼ぶ（他アノテーションより前） |
| `check_w020_deprecated_call` の関数シグネチャが `lint_program` と不一致 | `cargo check` エラー | `program: &Program` → `&mut Vec<LintError>` のシグネチャを確認 |
| `Expr::FnCall` のバリアント名が実際と異なる | `cargo check` エラー | `grep -n "FnCall\|fn_call" src/ast.rs` で確認 |
