# v24.4.0 — v1.0 後方互換性ポリシー確定

Date: 2026-06-23

## 目標

v25.0 = v1.0 リリース候補に向けて、破壊的変更ポリシーを確定する。
具体的には:
1. `STABILITY.md`（互換性ポリシーの文書化）
2. `#[deprecated]` アノテーション（fn への付与と W020 lint 警告）
3. `fav lint` W020 ルール — `#[deprecated]` 付き関数の呼び出し検出

---

## ロードマップとの対応

| ロードマップ | v24.4.0 での対応 |
|---|---|
| `STABILITY.md`（互換性ポリシーの文書化） | 新規作成 ✓（ロードマップ本文には `STABILITY.md` という名称はないが、実装上の成果物として追加） |
| deprecation warning 機能（`@deprecated` アノテーション） | `#[deprecated]` に変更（既存 `#[api]`/`#[stateful]` と構文統一）✓ |
| semver lint（v1.x 互換性違反を検出） | W020 `deprecated_call` として実装（`fav lint` に追加）✓ |

> **構文選択の理由**: ロードマップは `@deprecated` と記載しているが、Favnir の既存アノテーション構文は `#[annotation_name]` を採用（`#[api(...)]`、`#[stateful]`、`#[trigger(...)]` 等）。一貫性のため `#[deprecated]` を採用する。

---

## スコープ

### 変更種別

| 変更種別 | 対象 | 内容 |
|---|---|---|
| AST 拡張 | `ast.rs` | `FnDef` に `deprecated: bool` フィールドを追加 |
| パーサー拡張 | `parser.rs` | `parse_deprecated_annotation()` を追加し `parse_item()` に組み込む |
| Lint 追加 | `lint.rs` | `check_w020_deprecated_call(program)` を追加し `lint_program()` に組み込む |
| 新規作成 | `STABILITY.md` | v1.x 後方互換ポリシー・v2.0 破壊的変更ポリシー・SemVer 宣言 |
| 新規作成 | `benchmarks/v24.4.0.json` | test_count: 1948 |
| 更新 | `CHANGELOG.md` | v24.4.0 エントリ追加 |

### スコープ外

- `#[deprecated]` をユーザー定義メッセージ付きにする（`#[deprecated("use foo instead")]`）— v24.7+ で検討
- `--legacy` フラグ自体を `#[deprecated]` 扱いにする — v25.0 ポリシー宣言時に対応
- SemVer バージョン文字列バリデーション — v24.7 semver lint で対応

---

## `#[deprecated]` アノテーション仕様

### 構文

```favnir
#[deprecated]
fn old_func(x: Int) -> Int {
    x + 1
}
```

### 制約

- `fn` 定義にのみ付与可（`trf`・`flw` への付与は非対応、将来検討）
- `#[deprecated]` を付与した関数を**他の関数から呼び出す**と W020 が発生する
- `#[deprecated]` 関数の内部実装は lint 対象外（自己呼び出しは警告しない）
- `#[deprecated]` 関数の定義自体は lint 対象外（定義のみでは W020 は出ない）
- `#[api]` との併用（`#[deprecated] #[api(...)]`）は **スコープ外**（v24.7+ 予定）。parse_item() では `parse_deprecated_annotation` を `parse_api_annotation` より先に呼ぶため、`#[deprecated]` 単独付与は正常動作するが、両者の組み合わせでの動作は保証しない
- `impl` ブロック内 `fn` への `#[deprecated]` は **スコープ外**（parse_impl_def は parse_item() を通らないため）

### 実装パターン（`parse_item()` 内）

```rust
// v24.4.0: parse optional #[deprecated] annotation before fn
let deprecated_ann = self.parse_deprecated_annotation()?;
// ... existing annotations ...
let vis = self.parse_visibility();
match self.peek() {
    TokenKind::Fn => {
        let mut fd = self.parse_fn_def(vis, false)?;
        fd.deprecated = deprecated_ann;
        // ... api_annotation etc.
        return Ok(Item::FnDef(fd));
    }
    // ...
}
```

`parse_deprecated_annotation`:

```rust
fn parse_deprecated_annotation(&mut self) -> Result<bool, ParseError> {
    // peek: Hash, next: LBracket, then Ident("deprecated"), then RBracket
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

---

## W020: `deprecated_call` 仕様

### 検出条件

`#[deprecated]` アノテーション付きの `fn` を呼び出している箇所を検出する。

- 自己呼び出し（再帰）は対象外
- `#[deprecated]` 関数の定義内部からの呼び出しは対象外（定義者は意図的に使用している）

### 警告メッセージ

```
W020: call to deprecated function `old_func`
```

### `check_w020_deprecated_call` 実装方針

```rust
pub fn check_w020_deprecated_call(program: &Program) -> Vec<LintError> {
    // Step 1: deprecated fn 名のセットを収集
    let deprecated: HashSet<String> = program.items.iter()
        .filter_map(|item| {
            if let Item::FnDef(fd) = item {
                if fd.deprecated { Some(fd.name.clone()) } else { None }
            } else { None }
        })
        .collect();
    if deprecated.is_empty() { return vec![]; }

    // Step 2: 全 FnDef / TrfDef の body を走査して呼び出し検出
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd) => {
                // 自己定義が deprecated でも内部呼び出しは警告しない
                collect_deprecated_calls_in_block(&fd.body, &deprecated, &mut errors);
            }
            Item::TrfDef(td) => {
                collect_deprecated_calls_in_block(&td.body, &deprecated, &mut errors);
            }
            _ => {}
        }
    }
    errors
}

fn collect_deprecated_calls_in_expr(expr: &Expr, deprecated: &HashSet<String>, errors: &mut Vec<LintError>) {
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
            for a in args { collect_deprecated_calls_in_expr(a, deprecated, errors); }
        }
        // Expr::If(cond, then, else_, span)
        Expr::If(cond, then, else_, _) => {
            collect_deprecated_calls_in_expr(cond, deprecated, errors);
            collect_deprecated_calls_in_block(then, deprecated, errors);
            if let Some(e) = else_ { collect_deprecated_calls_in_block(e, deprecated, errors); }
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
```

---

## `STABILITY.md` 内容

```markdown
# Favnir Stability Policy

## v1.x 後方互換性保証

- マイナーバージョン（v1.x.y）では破壊的変更を行わない
- パッチバージョン（v1.x.y+1）はバグ修正のみ

## v2.0 破壊的変更ポリシー

- 破壊的変更は **2 年前から** `#[deprecated]` アノテーションで事前警告する
- `#[deprecated]` 付き API は `fav lint` で W020 警告が表示される
- 削除予定は CHANGELOG.md と DEPRECATIONS.md に記録する

## SemVer 準拠

Favnir は Semantic Versioning 2.0.0 に完全準拠する。

## `--legacy` フラグ

`--legacy` フラグは v2.0 まで維持する（削除しない）。
v1.x ユーザーは `--legacy` で旧挙動にアクセスできる。
```

---

## テスト（5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_4_0` | Cargo.toml に `version = "24.4.0"` | — |
| `deprecated_fn_annotation_parsed` | `#[deprecated] fn foo() -> Int { 42 }` を parse + `fn.deprecated == true` | `deprecated == true` |
| `deprecated_call_emits_w020` | `#[deprecated] fn old(); fn new() { old() }` → lint | W020 1件 |
| `stability_md_has_policy` | `STABILITY.md` に `"v1.x"` と `"v2.0"` が含まれる | — |
| `changelog_has_v24_4_0` | `CHANGELOG.md` に `[v24.4.0]` | — |

---

## 完了条件

- [ ] `FnDef.deprecated: bool` フィールド追加済み
- [ ] `parse_deprecated_annotation()` 実装済み（`#[deprecated]` を認識）
- [ ] `check_w020_deprecated_call()` 実装済み（`lint_program()` に組み込み）
- [ ] `STABILITY.md` 作成済み（v1.x / v2.0 / SemVer / --legacy の 4 セクション）
- [ ] `cargo test v244000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1948 件合格）
- [ ] `CHANGELOG.md` に v24.4.0 エントリ
- [ ] `benchmarks/v24.4.0.json` 作成済み（test_count: 1948）
