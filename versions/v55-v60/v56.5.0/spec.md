# Spec — v56.5.0 — OR パターン + パターンガード強化

## 概要

match 式の OR パターン（`Pat1 | Pat2`）とパターンガード（`if cond`）を組み合わせた
動作の検証・テストを行い、**W037（到達不能パターン）** lint ルールを `lint.rs` に追加する。

`Pattern::Or` および `MatchArm.guard` 自体は v17.2.0 時点で実装済みであるため、
本バージョンでは AST・パーサー・チェッカー・codegen/VM の変更は行わない。
新規追加は **W037 lint ルール + W037 unit test + OR パターン型チェック回帰テスト 2 件** のみ。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.5.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.5.0 行
- ベーステスト数: **3235**（v56.4.0 完了時点の実績値）
- 目標テスト数: **3237**（+2）
  - ロードマップ記載の「3236 + 2 = 3238」は v56.4.0 の当初見込みベースの値。
    実際の v56.4.0 は 3235 tests を達成したため、本バージョンの正確な目標は **3237**。

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `Pattern::Or(Vec<Pattern>, Span)` AST ノード | v17.2.0 | 実装済み（`ast.rs` L298） |
| `parse_match_arm` の OR パターン解析 | v17.2.0 | 実装済み（`parser.rs` L3569-3578） |
| `MatchArm.guard: Option<Box<Expr>>` | v0.5.0 | 実装済み（`ast.rs` L337） |
| guard 構文 `if expr` / `where expr` | v17.2.0 | 実装済み（`parser.rs` L3582-3589） |
| `checker.rs` `check_pattern_bindings` OR アーム | v17.2.0 | 実装済み（`checker.rs` L4206-4210） |
| `collect_pattern_variants` OR 展開 | v17.2.0 | 実装済み（`checker.rs` L10386-10390） |
| `codegen.rs` `IRPattern::Or` | v17.2.0 | 実装済み（`codegen.rs` L608） |
| **W037 到達不能パターン lint** | — | **未実装（本バージョンで追加）** |

**ロードマップとの差異**: ロードマップは「`PatternOr` AST ノード — 新規追加」と記載しているが、
`Pattern::Or` は v17.2.0 時点で実装済みである。本バージョンでは AST 変更は不要。
ロードマップの該当記述は実態に合わせて修正する（T12/T13 参照）。

---

## スコープの明確化

### `Pattern::Or` を新規追加しない根拠

ロードマップには「`PatternOr` AST ノード — 新規追加」と記載されているが、
`Pattern::Or(Vec<Pattern>, Span)` は v17.2.0 時点で実装済みである（`ast.rs` L298）。
パーサー・チェッカー・codegen・VM も対応済みのため、本バージョンでの再実装は不要。
`v56500_tests` の 2 件は**既存動作の回帰テスト**として機能する。

### W037 実装方針（シンプル・確実）

全 match 式に対して以下の到達不能パターンを検出する:

1. **ワイルドカード早期終端**: `_` または bind パターンが**ガードなし**で最後のアーム以外に現れた場合、
   直後の 1 アームのみに W037 を発行して終了する（それ以降のアームは報告しない）。
   注: `_ if cond -> ...` はガード付きのため catch-all ではなく W037 の対象外。
2. **リテラル重複**: 同一の文字列・整数・浮動小数点・bool リテラルパターンが
   同一 match 内に 2 回以上現れる場合 → W037

OR パターン内部の重複（`Ok(_) | Ok(_)` 等）は v56.5.0 スコープ外とする。
完全な到達不能解析（型情報を使った網羅性チェック）は将来スプリントとする。

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.5.0"
```

---

### 2. `fav/src/lint.rs` — W037 `check_unreachable_patterns` 追加

W コードは `lint.rs` にのみ実装する（`error_catalog.rs` は E コード専用のため W037 は登録しない）。
W036 セクション（`check_deprecated_rune_calls` の末尾付近）の後に追加する。

```rust
// ── W037: unreachable_pattern (v56.5.0) ──────────────────────────────────────

/// W037: match 式内の到達不能パターンを検出する。
///
/// 検出ケース:
/// 1. ワイルドカード (`_`) またはバインドパターン（変数名）がガードなしで
///    非末尾のアームに存在し、直後のアームが隠される場合（直後の 1 件のみ報告）。
/// 2. 同一 match 式内に同じリテラルパターンが 2 回以上現れる場合。
///
/// Known limitation: OR パターン内部の重複や型情報が必要なケースは対象外。
/// Known limitation: catch-all 後の 2 件目以降の到達不能アームは報告しない（最初の 1 件のみ）。
pub fn check_unreachable_patterns(program: &Program) -> Vec<LintError> {
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd) => check_block_for_unreachable(&fd.body, &mut errors),
            Item::StageDef(sd) => {
                if let Some(body) = &sd.body {
                    check_expr_for_unreachable(body, &mut errors);
                }
            }
            _ => {}
        }
    }
    errors
}

fn check_block_for_unreachable(block: &Block, errors: &mut Vec<LintError>) {
    for stmt in &block.stmts {
        check_stmt_for_unreachable(stmt, errors);
    }
    if let Some(tail) = &block.tail {
        check_expr_for_unreachable(tail, errors);
    }
}

fn check_stmt_for_unreachable(stmt: &Stmt, errors: &mut Vec<LintError>) {
    match stmt {
        Stmt::Bind(b)  => check_expr_for_unreachable(&b.expr, errors),
        Stmt::Chain(c) => check_expr_for_unreachable(&c.expr, errors),
        Stmt::Expr(e)  => check_expr_for_unreachable(e, errors),
        Stmt::Yield(y) => check_expr_for_unreachable(&y.expr, errors),
        Stmt::Return(r) => check_expr_for_unreachable(&r.expr, errors),
        Stmt::ForIn(f) => {
            check_expr_for_unreachable(&f.iter, errors);
            check_block_for_unreachable(&f.body, errors);
        }
        Stmt::Forall(f) => check_block_for_unreachable(&f.body, errors),
        Stmt::Expect(e) => check_expr_for_unreachable(&e.expr, errors),
    }
}

fn check_expr_for_unreachable(expr: &Expr, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Match(scrutinee, arms, _) => {
            check_expr_for_unreachable(scrutinee, errors);
            let mut catch_all_seen = false;
            let mut seen_lits: std::collections::HashSet<String> = std::collections::HashSet::new();
            for arm in arms {
                if catch_all_seen {
                    errors.push(LintError {
                        code: "W037".to_string(),
                        message: "unreachable pattern: previous arm catches all values".to_string(),
                        span: arm.pattern.span().clone(),
                    });
                    break; // 最初の到達不能アームのみ報告
                }
                // リテラル重複チェック
                if let Some(lit_key) = pattern_lit_key(&arm.pattern) {
                    if !seen_lits.insert(lit_key.clone()) {
                        errors.push(LintError {
                            code: "W037".to_string(),
                            message: format!(
                                "unreachable pattern: literal `{}` already matched above",
                                lit_key
                            ),
                            span: arm.pattern.span().clone(),
                        });
                    }
                }
                // ガードなしのワイルドカード / バインドパターン検出
                if pattern_is_catch_all(&arm.pattern) && arm.guard.is_none() {
                    catch_all_seen = true;
                }
                check_expr_for_unreachable(&arm.body, errors);
            }
        }
        Expr::Block(b) => check_block_for_unreachable(b, errors),
        Expr::If(cond, then, else_, _) => {
            check_expr_for_unreachable(cond, errors);
            check_block_for_unreachable(then, errors);
            if let Some(eb) = else_ {
                check_block_for_unreachable(eb, errors);
            }
        }
        Expr::Apply(func, args, _) => {
            check_expr_for_unreachable(func, errors);
            for a in args { check_expr_for_unreachable(a, errors); }
        }
        Expr::BinOp(_, l, r, _) => {
            check_expr_for_unreachable(l, errors);
            check_expr_for_unreachable(r, errors);
        }
        Expr::FieldAccess(obj, _, _) => check_expr_for_unreachable(obj, errors),
        Expr::Closure(_, body, _) => check_expr_for_unreachable(body, errors),
        Expr::Pipeline(steps, _) => {
            for s in steps { check_expr_for_unreachable(s, errors); }
        }
        Expr::Collect(b, _) => check_block_for_unreachable(b, errors),
        Expr::EmitExpr(inner, _) | Expr::Question(inner, _) => {
            check_expr_for_unreachable(inner, errors);
        }
        Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields { check_expr_for_unreachable(v, errors); }
        }
        Expr::RecordSpread(base, updates, _) => {
            check_expr_for_unreachable(base, errors);
            for (_, v) in updates { check_expr_for_unreachable(v, errors); }
        }
        Expr::TypeApply(f, _, _) => check_expr_for_unreachable(f, errors),
        Expr::AssertMatches(e, _, _) => check_expr_for_unreachable(e, errors),
        Expr::AssertSchema { arg, .. } => check_expr_for_unreachable(arg, errors),
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e) = part { check_expr_for_unreachable(e, errors); }
            }
        }
        Expr::ListComp { expr, clauses, .. } | Expr::ResultComp { expr, clauses, .. } => {
            check_expr_for_unreachable(expr, errors);
            for c in clauses {
                match c {
                    CompClause::For { src, .. } => check_expr_for_unreachable(src, errors),
                    CompClause::Guard(g) => check_expr_for_unreachable(g, errors),
                }
            }
        }
        Expr::Lit(..) | Expr::Ident(..) => {}
    }
}

/// パターンがキャッチオール（すべての値にマッチする）か判定する。
/// ガードなしの `_` またはバインドパターンがキャッチオール。
/// 呼び出し元で `arm.guard.is_none()` と組み合わせて使用する。
fn pattern_is_catch_all(pat: &Pattern) -> bool {
    matches!(pat, Pattern::Wildcard(_) | Pattern::Bind(_, _))
}

/// リテラルパターンの一意キーを返す（重複検出用）。
/// リテラル以外（OR パターン含む）は None を返す。
fn pattern_lit_key(pat: &Pattern) -> Option<String> {
    if let Pattern::Lit(lit, _) = pat {
        Some(format!("{:?}", lit))
    } else {
        None
    }
}
```

`Stmt` の全バリアント（`Bind` / `Chain` / `Expr` / `Yield` / `Return` / `ForIn` / `Forall` / `Expect`）を
`check_stmt_for_unreachable` で網羅する（`ForIn.body` 内の match 式も検出対象とする）。

---

### 3. `fav/src/lint.rs` — `run_lint` への統合

既存の lint 呼び出しパス（`cmd_lint` または `run_lint`）に追加する:

```rust
errors.extend(check_unreachable_patterns(program));
```

---

### 4. `fav/src/driver.rs` — `v56500_tests` 追加

`v56400_tests` の直前に挿入する。

```rust
// -- v56500_tests (v56.5.0) -- OR パターン + パターンガード強化 --
#[cfg(test)]
mod v56500_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;
    use crate::lint::check_unreachable_patterns;

    fn check_errors(src: &str) -> Vec<String> {
        let program = Parser::parse_str(src, "v56500_test.fav").expect("parse");
        Checker::check_program(&program)
            .0
            .iter()
            .map(|e| e.code.clone())
            .collect()
    }

    #[test]
    fn match_or_pattern() {
        // OR パターン: Ok(x) | Err(_) が正しく解析・型チェックされる
        let src = r#"
fn classify(r: Result<Int, String>) -> String {
    match r {
        Ok(_)  | Err(_) -> "handled"
    }
}
public fn main() -> Bool { true }
"#;
        let errors = check_errors(src);
        assert!(
            errors.is_empty(),
            "OR pattern should type-check without errors, got: {:?}", errors
        );
    }

    #[test]
    fn match_or_with_guard() {
        // OR パターン + guard の組み合わせ
        let src = r#"
fn label(s: String) -> String {
    match s {
        "yes" | "ok" if true -> "positive"
        _                    -> "other"
    }
}
public fn main() -> Bool { true }
"#;
        let errors = check_errors(src);
        assert!(
            errors.is_empty(),
            "OR pattern with guard should type-check without errors, got: {:?}", errors
        );
    }

    // W037 unit test: catch-all 後のアームに W037 が発行されることを確認
    #[test]
    fn w037_unreachable_after_wildcard() {
        let src = r#"
fn f(x: Int) -> String {
    match x {
        _ -> "catch-all"
        1 -> "one"
    }
}
public fn main() -> Bool { true }
"#;
        let program = Parser::parse_str(src, "w037_test.fav").expect("parse");
        let warnings = check_unreachable_patterns(&program);
        assert!(
            warnings.iter().any(|w| w.code == "W037"),
            "expected W037 for arm after wildcard, got: {:?}", warnings
        );
    }
}
```

---

### 5. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests` モジュール内の `cargo_toml_version_is_56_3_0` テストの期待値を
`"56.4.0"` から `"56.5.0"` に更新する。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `match_or_pattern` | `Ok(_) \| Err(_) -> "handled"` が型チェックエラーなし |
| `match_or_with_guard` | `"yes" \| "ok" if true -> "positive"` が型チェックエラーなし |
| `w037_unreachable_after_wildcard` | `_` の後のアームに W037 が発行される |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3237 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `match_or_pattern` pass
- `match_or_with_guard` pass
- `w037_unreachable_after_wildcard` pass（W037 が発行される）
- `lint.rs` に `check_unreachable_patterns`（pub）が追加されている
- `run_lint` / `cmd_lint` から `check_unreachable_patterns` が呼ばれている
- `CHANGELOG.md` に v56.5.0 エントリが追加されている
- `versions/current.md` が v56.5.0 / 3237 tests を反映
- 両ロードマップの v56.5.0 実績を COMPLETE に更新
- 両ロードマップの「`PatternOr` 新規追加」記述を実態（v17.2.0 実装済み）に修正

---

## 備考

- **`Pattern::Or` 非新規の根拠**: `ast.rs` L298 に `Pattern::Or(Vec<Pattern>, Span)` が存在する。
  `parser.rs` L3569-3578 で OR パターン構文を処理し、`checker.rs` L4206, L10386 でも対応済み。
- **W037 は `lint.rs` のみ**: `error_catalog.rs` は E コード専用。W コードは lint.rs のコメント + 関数で管理。
- **W037 catch-all break**: catch-all 後の直後アームにのみ W037 を発行して `break` する（2 件目以降は報告しない）。
  これは設計上の簡略化であり、将来のフルスキャンモードで改善可能。
- **ロードマップのテスト数差異**: 3238（ロードマップ記載）vs 3237（本 spec）の差は
  v56.4.0 実績ベースの違いによる（spec 側の 3237 = 3235 + 2 が正しい）。
- **`Stmt` 全バリアント対応**: `ForIn.body` 内の match 式も W037 検出対象とするため、
  `Stmt::ForIn`、`Stmt::Forall`、`Stmt::Yield`、`Stmt::Return`、`Stmt::Expect` も網羅する。
