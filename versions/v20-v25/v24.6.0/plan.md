# v24.6.0 実装計画 — セキュリティ審査（エフェクトシステム形式検証）

## 前提確認

```bash
grep -n "version = " fav/Cargo.toml
# → "24.5.0" であること

grep -n "mod v246000_tests" fav/src/driver.rs | head -3
# → 0 件であること

grep -n "W021\|check_w021\|pure_fn_calls_effectful" fav/src/lint.rs | head -5
# → 全 0 件であること

ls SECURITY.md SECURITY_MODEL.md 2>/dev/null
# → ファイルが存在しないこと
```

---

## T0: `fav/src/lint.rs` — W021 追加

### T0-1: 事前確認

```bash
grep -n "Effect::Pure\|crate::ast::Effect" fav/src/lint.rs | head -5
# → 既存の Effect 使用パターンを確認
```

### T0-2: `check_w021_expr` / `check_w021_block` を追加

`lint.rs` の `check_w020_*` 関数群の直後に追加する:

```rust
// ── W021: pure_fn_calls_effectful (v24.6.0) ──────────────────────────────────
fn check_w021_block(
    block: &Block,
    caller: &str,
    effectful: &std::collections::HashSet<String>,
    errors: &mut Vec<LintError>,
) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b)  => check_w021_expr(&b.expr, caller, effectful, errors),
            Stmt::Chain(c) => check_w021_expr(&c.expr, caller, effectful, errors),
            Stmt::Expr(e)  => check_w021_expr(e, caller, effectful, errors),
            Stmt::Yield(y) => check_w021_expr(&y.expr, caller, effectful, errors),
            Stmt::ForIn(f) => {
                check_w021_expr(&f.iter, caller, effectful, errors);
                check_w021_block(&f.body, caller, effectful, errors);
            }
            Stmt::Forall(f) => {
                // Forall は iter フィールドなし（guard + body のみ）
                if let Some(g) = &f.guard { check_w021_expr(g, caller, effectful, errors); }
                check_w021_block(&f.body, caller, effectful, errors);
            }
        }
    }
    check_w021_expr(&block.expr, caller, effectful, errors);
}

fn check_w021_expr(
    expr: &Expr,
    caller: &str,
    effectful: &std::collections::HashSet<String>,
    errors: &mut Vec<LintError>,
) {
    match expr {
        // Expr::Apply(func, args, span) — 関数呼び出し
        Expr::Apply(func, args, span) => {
            if let Expr::Ident(name, _) = func.as_ref() {
                if effectful.contains(name) {
                    errors.push(LintError::new(
                        "W021",
                        format!(
                            "pure function `{caller}` calls effectful function `{name}` \
                             — declare the effect or mark `{caller}` as effectful"
                        ),
                        span.clone(),
                    ));
                }
            }
            check_w021_expr(func, caller, effectful, errors);
            for a in args { check_w021_expr(a, caller, effectful, errors); }
        }
        Expr::If(cond, then, else_, _) => {
            check_w021_expr(cond, caller, effectful, errors);
            check_w021_block(then, caller, effectful, errors);
            if let Some(e) = else_ { check_w021_block(e, caller, effectful, errors); }
        }
        Expr::Match(subject, arms, _) => {
            check_w021_expr(subject, caller, effectful, errors);
            // MatchArm.body は Expr（Block ではない）
            for arm in arms { check_w021_expr(&arm.body, caller, effectful, errors); }
        }
        Expr::Block(b) => check_w021_block(b, caller, effectful, errors),
        Expr::Closure(_, body, _) => check_w021_expr(body, caller, effectful, errors),
        Expr::Pipeline(steps, _) => {
            for s in steps { check_w021_expr(s, caller, effectful, errors); }
        }
        _ => {}
    }
}
```

### T0-3: `check_w021_pure_fn_calls_effectful` を追加

```rust
/// v24.6.0: W021 — 純粋関数から副作用関数を呼び出す箇所を検出する。
/// 「capability 引数がなければ純粋」という Favnir エフェクトシステムの公理を lint として実装。
pub fn check_w021_pure_fn_calls_effectful(program: &Program, errors: &mut Vec<LintError>) {
    use std::collections::HashSet;
    // Effect は lint.rs 先頭の `use crate::ast::*;` により既にスコープ内
    // Step 1: エフェクト宣言のある fn 名のセットを収集（Pure 以外のエフェクトを持つもの）
    let effectful_fns: HashSet<String> = program.items.iter()
        .filter_map(|item| {
            if let Item::FnDef(fd) = item {
                let is_effectful = fd.effects.iter().any(|e| e != &Effect::Pure);
                if is_effectful { Some(fd.name.clone()) } else { None }
            } else {
                None
            }
        })
        .collect();
    if effectful_fns.is_empty() { return; }
    // Step 2: 純粋な FnDef（effects が空、または Pure のみ）の body を走査
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            let is_pure = fd.effects.is_empty()
                || fd.effects.iter().all(|e| e == &Effect::Pure);
            if is_pure {
                check_w021_block(&fd.body, &fd.name, &effectful_fns, errors);
            }
        }
    }
}
```

### T0-4: `lint_program()` に組み込み

`check_w020_deprecated_call` の直後に追加:

```rust
check_w020_deprecated_call(program, &mut errors);
// v24.6.0: W021
check_w021_pure_fn_calls_effectful(program, &mut errors);
```

- [ ] **事後確認**: `cargo check --bin fav` — エラー 0

---

## T1: `SECURITY.md` 作成

トップレベル（`C:\Users\yoshi\favnir\SECURITY.md`）に作成。

```markdown
# Favnir Security Policy

## 脆弱性の報告

セキュリティ上の問題を発見した場合は、**公開 Issue は作成しないでください**。
代わりに以下のメールアドレスへ報告してください:

**security@favnir.dev**

## 90日 Responsible Disclosure

報告を受けた後、以下のプロセスに従います:

1. **7 日以内**: 報告の受領確認
2. **30 日以内**: 初期調査・深刻度評価（CVSS スコア）
3. **90 日以内**: パッチのリリース
4. **90 日後**: 脆弱性の公開（報告者と合意の上）

## CVE 番号付与

CVSS スコア 4.0 以上の脆弱性には CVE 番号を申請します。

## パッチリリースポリシー

セキュリティパッチは patch バージョン（v1.x.y+1）として即時リリースします。
バージョン 1.x シリーズはリリース後 12 ヶ月間セキュリティパッチを受け取ります。

## 連絡先

- Email: security@favnir.dev
- PGP: 公開鍵は https://favnir.dev/security/pgp.txt で公開（v25.0 予定）
```

---

## T2: `SECURITY_MODEL.md` 作成

トップレベル（`C:\Users\yoshi\favnir\SECURITY_MODEL.md`）に作成。

```markdown
# Favnir エフェクトシステム 形式的仕様

## 概要

Favnir のエフェクトシステムは「capability 引数がなければ純粋」という原則に基づく。
本ドキュメントは、この原則を形式的に記述する。

## Capability 公理（Axiom）

**公理 1: 純粋性（Purity）**
  fn f: A → B（エフェクト宣言なし）⊢ f は参照透明（referentially transparent）

**公理 2: 効果の伝播（Effect Propagation）**
  fn f: A → B !E  かつ  fn g が f を呼び出す  ⟹  g は !E を宣言しなければならない

**公理 3: 能力の封じ込め（Capability Confinement）**
  !E を宣言しない関数からは !E エフェクトを発生させることができない

**公理 4: 合成（Composition）**
  fn f: A → B !E₁  かつ  fn g: B → C !E₂  ⟹  f |> g : A → C !(E₁ ∪ E₂)

## 推論規則（Inference Rules）

[T-Pure]
  Γ ⊢ e : τ,  effects(e) = ∅
  ────────────────────────────
  Γ ⊢ fn e : τ  (純粋関数)

[T-Effect]
  Γ ⊢ f : A → B !E,  Γ ⊢ g calls f
  ────────────────────────────────────
  Γ ⊢ g must declare !E

[T-Compose]
  Γ ⊢ f : A → B !E₁,  Γ ⊢ g : B → C !E₂
  ──────────────────────────────────────────
  Γ ⊢ f |> g : A → C !(E₁ ∪ E₂)

## W021 Lint ルールとの対応

公理 2（Effect Propagation）および公理 3（Capability Confinement）の
コード内検証として W021 `pure_fn_calls_effectful` を実装している。

## 外部審査（External Audit）依頼事項

本仕様の正式な機械検証（TLA+ / Coq）は v25.0 前後を目標に実施予定。
審査依頼事項:
- 公理の無矛盾性（consistency）
- 効果システムの健全性（soundness: 型付け可能ならランタイムエラーなし）
- 完全性（completeness: 意図した副作用はすべてエフェクトで宣言される）
```

---

## T3: `fav/src/driver.rs` — v246000_tests 追加

### T3-1: `v245000_tests::version_is_24_5_0` を削除

`v245000_tests` モジュール内の `version_is_24_5_0` **関数のみ**を削除する。
モジュール自体と残りの4件のテストはそのまま保持すること。

### T3-2: `v246000_tests` モジュールを `v245000_tests` の直後に追加

```rust
// ── v246000_tests (v24.6.0) — セキュリティ審査 ──────────────────────────────
#[cfg(test)]
mod v246000_tests {
    use super::*;

    #[test]
    fn version_is_24_6_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(
            cargo.contains("version = \"24.6.0\""),
            "Cargo.toml should have version 24.6.0"
        );
    }

    #[test]
    fn security_md_has_disclosure_policy() {
        let md = include_str!("../../SECURITY.md");
        assert!(
            md.contains("security@favnir.dev"),
            "SECURITY.md should have security@favnir.dev"
        );
        assert!(
            md.contains("90"),
            "SECURITY.md should mention 90-day disclosure period"
        );
    }

    #[test]
    fn security_model_md_exists() {
        let md = include_str!("../../SECURITY_MODEL.md");
        assert!(
            md.contains("Capability"),
            "SECURITY_MODEL.md should contain 'Capability'"
        );
        assert!(
            md.contains("Purity"),
            "SECURITY_MODEL.md should contain 'Purity'"
        );
    }

    #[test]
    fn w021_pure_fn_calls_effectful_lint() {
        let src = r#"
            fn fetch_data(url: String) -> String !Http { url }

            fn process(url: String) -> String { fetch_data(url) }
        "#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("tokenize failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        let mut warnings = Vec::new();
        crate::lint::check_w021_pure_fn_calls_effectful(&prog, &mut warnings);
        assert_eq!(
            warnings.len(), 1,
            "expected 1 W021 warning, got: {warnings:?}"
        );
        assert!(
            warnings[0].message.contains("process"),
            "warning should mention caller 'process': {:?}", warnings[0]
        );
        assert!(
            warnings[0].message.contains("fetch_data"),
            "warning should mention callee 'fetch_data': {:?}", warnings[0]
        );
    }

    #[test]
    fn changelog_has_v24_6_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(
            cl.contains("[v24.6.0]"),
            "CHANGELOG.md should have [v24.6.0] entry"
        );
    }
}
```

- [ ] `cargo test v246000 --bin fav` — 5/5 PASS を確認
- [ ] `cargo test --bin fav` — リグレッションなし（1957 件合格）を確認

> **件数計算**: 現在 1953 件 → `version_is_24_5_0` 削除 (-1) → v246000_tests 追加 (+5) → 1957 件

---

## T4: ドキュメントサイト更新

`site/content/docs/tools/security.mdx` を新規作成:

```mdx
---
title: "セキュリティモデル"
description: "Favnir のエフェクトシステムによるセキュリティ保証と CVE 対応プロセス"
---

# セキュリティモデル

## エフェクトシステムの保証

「capability 引数がなければ純粋」— Favnir の根本原則。
...（SECURITY_MODEL.md の内容を要約）
```

---

## T5: Cargo.toml + CHANGELOG + benchmarks

> **注意**: T3-1 の `version_is_24_5_0` 削除完了後に Cargo.toml を更新すること。

### T5-1: `fav/Cargo.toml` バージョン更新

```
version = "24.5.0" → "24.6.0"
```

### T5-2: `CHANGELOG.md` 先頭に v24.6.0 エントリ追加

```markdown
## [v24.6.0] — 2026-06-23 — セキュリティ審査（エフェクトシステム形式検証）

### Added
- W021 `pure_fn_calls_effectful` lint ルール — 純粋関数から副作用関数を呼び出す箇所を検出
- `SECURITY.md` — CVE 対応プロセス（security@favnir.dev、90日 responsible disclosure）
- `SECURITY_MODEL.md` — エフェクトシステムの形式的仕様（capability 公理 4 条 + 推論規則）
- `site/content/docs/tools/security.mdx` — セキュリティモデル解説ページ

### Notes
- W021 は `fn` 定義間の呼び出し関係のみ検出。`trf`/`flw` 対応は v24.7+ 予定
- TLA+/Coq による機械検証は v25.0 前後を目標
```

### T5-3: `benchmarks/v24.6.0.json` 作成

```json
{
  "version": "24.6.0",
  "date": "2026-06-23",
  "test_count": 1957,
  "feature": "セキュリティ審査（エフェクトシステム形式検証）",
  "metrics": {
    "test_count": 1957,
    "duration_ms": 17000
  }
}
```

---

## 実装順序

```
T0（lint.rs: check_w021_* 追加 + lint_program 組み込み）
cargo check → エラー 0 確認
T1（SECURITY.md 作成）
T2（SECURITY_MODEL.md 作成）
T3-1（version_is_24_5_0 関数のみ削除、モジュール・他4件は保持）← T5-1 より前に必須
T3-2（v246000_tests 追加）
cargo test v246000 → 5/5 PASS 確認
T4（site/content/docs/tools/security.mdx 作成）
T5-1（version 更新）← T3-1 完了後
T5-2〜3（CHANGELOG / v24.6.0.json）
cargo test --bin fav → リグレッションなし確認（1957 件）
```

---

## リスク対応表

| リスク | 検出方法 | 対応 |
|---|---|---|
| `Effect::Pure` との比較で PartialEq が使えない | `cargo check` コンパイルエラー | `ast.rs` の `#[derive(PartialEq)]` を確認（既存 derive あり）|
| `crate::ast::Effect` の import が lint.rs に不足 | `cargo check` エラー "use of undeclared type" | `use crate::ast::Effect;` を `check_w021_*` の先頭に `use crate::ast::Effect;` で追加 |
| W021 が effectful な fn の内部で再帰的に同じ fn を呼ぶ場合に誤検出 | `w021_pure_fn_calls_effectful_lint` テスト | effectful fn（`is_pure == false`）はスキップ済みのため問題なし |
| `Expr::Ident` が `Expr::Var` と誤記される | `cargo check` エラー "no variant named Var" | `Expr::Ident(name, span)` を使用（v24.4.0 の技術知見参照）|
| `MatchArm.body` が `Block` と誤記される | `cargo check` 型エラー | `check_w021_expr(&arm.body, ...)` — `body` は `Expr` 型（v24.4.0 参照）|
| テスト `fn fetch_data ... { url }` のパース確認 | `cargo test v246000` 実行で自明に検証 | `{ url }` → `block.expr = Ident("url")`, `block.stmts = []`。W021 は `block.expr` から check_w021_expr を呼ぶため正常に検出される |
