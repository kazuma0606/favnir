# v24.6.0 — セキュリティ審査（エフェクトシステム形式検証）

Date: 2026-06-23

## 目標

「capability 引数がなければ純粋」を形式的に証明し、Favnir のセキュリティ基盤を整備する。
具体的には:
1. `SECURITY_MODEL.md` — エフェクトシステムの形式的仕様（capability 公理・推論規則）
2. `SECURITY.md` — CVE 対応プロセス（`security@favnir.dev` + 90日 responsible disclosure）
3. W021 `pure_fn_calls_effectful` — 純粋関数から副作用関数を呼び出す箇所を検出する lint ルール
4. ドキュメントサイト更新（`site/content/docs/tools/security.mdx`）

---

## ロードマップとの対応

| ロードマップ | v24.6.0 での対応 |
|---|---|
| エフェクトシステムの形式的仕様（TLA+ / Coq） | `SECURITY_MODEL.md`（公理・推論規則を Markdown で記述）✓。実際の TLA+/Coq 証明ファイルは v25.x で別途追加予定 |
| 外部審査（言語設計の専門家によるレビュー） | `SECURITY_MODEL.md` の審査依頼テンプレートセクションを追加 ✓ |
| CVE 対応プロセスの確立（`security@favnir.dev` + 90日 responsible disclosure） | `SECURITY.md` を新規作成 ✓ |
| W021 `pure_fn_calls_effectful` | lint.rs に追加 ✓（ロードマップ追記事項） |

> **実装アプローチ**: ロードマップに「エフェクトシステムの lint 検証」は明記されていないが、
> 「capability 引数がなければ純粋」という命題をコードとして実現する最小単位として W021 を追加する。
> これにより形式的仕様がテスト可能な形で実装に結びつく。

---

## スコープ

### 変更種別

| 変更種別 | 対象 | 内容 |
|---|---|---|
| 新規作成 | `SECURITY.md` | CVE 対応プロセス（security@favnir.dev / 90日 responsible disclosure / 連絡先） |
| 新規作成 | `SECURITY_MODEL.md` | エフェクトシステム形式仕様（capability 公理 4 条 + 推論規則） |
| Lint 追加 | `lint.rs` | `check_w021_pure_fn_calls_effectful(program, errors)` + `lint_program()` への組み込み |
| ドキュメント更新 | `site/content/docs/tools/security.mdx` | セキュリティモデル・CVE プロセス紹介ページ |
| 新規作成 | `benchmarks/v24.6.0.json` | test_count: 1957 |
| 更新 | `CHANGELOG.md` | v24.6.0 エントリ追加 |

### スコープ外

- TLA+/Coq による機械検証済みの証明ファイル — v25.x で追加予定
- 実際の外部セキュリティ審査（専門家によるレビュー実施）— v25.0 前後を目標
- W021 の `TrfDef` / `FlwDef` 対応 — v24.7+ で検討
- `impl` ブロック内メソッドへの W021 適用 — v24.7+ で検討

---

## `SECURITY.md` 仕様

### 必須内容

```
- security@favnir.dev への連絡先
- 90日 responsible disclosure ルール
- CVE 番号付与プロセス
- パッチリリースポリシー（セキュリティパッチは patch バージョンで即時リリース）
```

**テスト要件**: `security_md_has_disclosure_policy` テストが以下を確認
- `"security@favnir.dev"` の文字列を含む
- `"90"` の文字列を含む（90日ルール）

---

## `SECURITY_MODEL.md` 仕様

### 形式的仕様の構造

```markdown
## Capability 公理（Axiom）

公理 1: 純粋性（Purity）
  fn f: A → B with no effects ⊢ f は参照透明（referentially transparent）

公理 2: 効果の伝播（Effect Propagation）
  fn f: A → B !E  かつ  fn g が f を呼び出す  ⟹  g は !E を宣言しなければならない

公理 3: 能力の封じ込め（Capability Confinement）
  !E を宣言しない関数からは !E エフェクトを発生させることができない

公理 4: 合成（Composition）
  fn f: A → B !E₁  かつ  fn g: B → C !E₂  ⟹  f |> g は !E₁ ∪ !E₂ を宣言する

## 推論規則

[T-Pure]   Γ ⊢ e : τ   Δ(e) = ∅
           ————————————————————
           Γ ⊢ pure fn e : τ

[T-Effect] Γ ⊢ f : A → B !E   Γ ⊢ g calls f
           ————————————————————————————————————
           Γ ⊢ g must declare !E

[T-Compose] Γ ⊢ f : A → B !E₁   Γ ⊢ g : B → C !E₂
            ————————————————————————————————————————
            Γ ⊢ f |> g : A → C !(E₁ ∪ E₂)
```

**テスト要件**: `security_model_md_exists` テストが以下を確認
- `"Capability"` の文字列を含む
- `"Purity"` または `"純粋性"` の文字列を含む

---

## W021: `pure_fn_calls_effectful` 仕様

### 検出条件

`effects` が空（または `[Effect::Pure]` のみ）の関数が、エフェクト宣言のある（`effects` が空でない、かつ `Pure` 以外を含む）関数を呼び出している箇所を検出する。

**「純粋」の定義**: `fd.effects.is_empty() || fd.effects.iter().all(|e| e == &Effect::Pure)`

### 警告メッセージ

```
W021: pure function `caller` calls effectful function `callee` — declare the effect or mark caller as effectful
```

### 検出例

```favnir
fn transform_http(url: String) -> String !Http {
    Http.get(url)
}

// W021: pure function `process` calls effectful function `transform_http`
fn process(url: String) -> String {
    transform_http(url)
}
```

### `check_w021_pure_fn_calls_effectful` 実装方針

```rust
pub fn check_w021_pure_fn_calls_effectful(program: &Program, errors: &mut Vec<LintError>) {
    use std::collections::HashSet;
    // Step 1: エフェクト宣言のある fn 名のセットを収集
    let effectful_fns: HashSet<String> = program.items.iter()
        .filter_map(|item| {
            if let Item::FnDef(fd) = item {
                let is_effectful = fd.effects.iter().any(|e| e != &Effect::Pure);
                if is_effectful { Some(fd.name.clone()) } else { None }
            } else { None }
        })
        .collect();
    if effectful_fns.is_empty() { return; }
    // Step 2: 純粋な FnDef の body を走査して effectful な呼び出しを検出
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

fn check_w021_block(block: &Block, caller: &str, effectful: &HashSet<String>, errors: &mut Vec<LintError>) { ... }

fn check_w021_expr(expr: &Expr, caller: &str, effectful: &HashSet<String>, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Apply(func, args, span) => {
            if let Expr::Ident(name, _) = func.as_ref() {
                if effectful.contains(name) {
                    errors.push(LintError::new(
                        "W021",
                        format!("pure function `{caller}` calls effectful function `{name}` — declare the effect or mark `{caller}` as effectful"),
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

> **重要**: `Effect` は `lint.rs` 先頭の `use crate::ast::*;` により既にスコープ内（追加 import 不要）。
> `Effect::Pure` との比較は `e != &Effect::Pure`（`ast.rs` の `#[derive(PartialEq)]` により有効）。

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_6_0` | Cargo.toml に `version = "24.6.0"` | — |
| `security_md_has_disclosure_policy` | `SECURITY.md` に `"security@favnir.dev"` と `"90"` が含まれる | — |
| `security_model_md_exists` | `SECURITY_MODEL.md` に `"Capability"` と `"Purity"` が含まれる | — |
| `w021_pure_fn_calls_effectful_lint` | 純粋関数から `!Http` 関数を呼ぶソースに W021 が 1 件 | `warnings.len() == 1` |
| `changelog_has_v24_6_0` | `CHANGELOG.md` に `[v24.6.0]` | — |

---

## 完了条件

- [ ] `check_w021_pure_fn_calls_effectful()` 実装済み（`lint_program()` に組み込み）
- [ ] `SECURITY.md` 作成済み（`security@favnir.dev` / `90` 日 / CVE プロセス）
- [ ] `SECURITY_MODEL.md` 作成済み（`Capability` / `Purity` / 推論規則）
- [ ] `v245000_tests::version_is_24_5_0` が削除済み（T5-1 より前）
- [ ] `cargo test v246000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1957 件合格）
- [ ] `CHANGELOG.md` に v24.6.0 エントリ
- [ ] `benchmarks/v24.6.0.json` 作成済み（test_count: 1957）
- [ ] `site/content/docs/tools/security.mdx` 作成済み
