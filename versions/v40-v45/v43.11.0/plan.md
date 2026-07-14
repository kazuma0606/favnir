# v43.11.0 実装計画 — Opaque type 完全化

## 前提

- ベース: v43.10.0 COMPLETE（2929 tests）
- `opaque` は現在 `TokenKind::Ident("opaque")` として lexer から出力される（キーワード未登録）
- `TypeDef` に `is_opaque` フィールドなし（ast.rs 行 200〜208）
- `parse_type_def` 内 TypeDef 構築箇所: 4 箇所（Wrapper/Record/Alias は early return, Sum は最終 Ok）
- E0413 は `error_catalog.rs` で予約コメントのみ（実エントリなし）
- `get_explain_text` に E0413 エントリなし
- `TokenKind::Ident` は `Ident(String)` であり、`ref` は不要（cloned 値に対するマッチ）
- `Expr::Lit(Lit, Span)` が正しい AST 定義（`Lit::Str(_)` で文字列リテラルを表す）

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `TypeDef` に `is_opaque: bool` を追加 |
| `fav/src/frontend/parser.rs` | `parse_item` に `"opaque"` アーム追加 / TypeDef 構築 4 箇所に `is_opaque: false` 追加 |
| `fav/src/error_catalog.rs` | E0413 予約コメント削除 → 実エントリ追加 |
| `fav/src/driver.rs` | `check_opaque_coerce_violations` / `is_bare_inner_literal` 追加 / `get_explain_text` E0413 追加 / `cmd_check` opaque チェック追加 / `v431100_tests` 追加 / `v431000_tests` スタブ化 |
| `fav/Cargo.toml` | version `43.10.0` → `43.11.0` |
| `CHANGELOG.md` | v43.11.0 エントリ追加 |
| `versions/roadmap/roadmap-v43.1-v44.0.md` | v43.11.0 エントリを「AST レベル MVP」に修正 |

---

## 実装ステップ

### Step 1 — ast.rs: `TypeDef.is_opaque` 追加

```rust
pub struct TypeDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub with_interfaces: Vec<String>,
    pub invariants: Vec<Expr>,
    pub is_opaque: bool,   // v43.11.0: opaque type キーワード（デフォルト false）
    pub body: TypeBody,
    pub span: Span,
}
```

### Step 2 — parser.rs: 4 箇所の TypeDef 構築に `is_opaque: false` 追加

`parse_type_def` 内の以下 4 箇所すべてに追加する（T1 と同時適用必須）:

1. **Wrapper type**（行 1496 付近）— `TypeBody::Wrapper` の early return
2. **Record type**（行 1526 付近）— `TypeBody::Record` の early return
3. **Alias type**（行 1548 付近）— `TypeBody::Alias` の early return
4. **Sum type**（行 1559 付近）— 最終 `Ok(TypeDef { ... body })` ← body に `TypeBody::Sum(...)` が入る。early return ではなく関数末尾の `Ok` であることに注意

### Step 3 — parser.rs: `parse_item` に `"opaque"` アーム追加

`match self.peek().clone()` ブロックの `TokenKind::Type =>` の直前に追加。
`TokenKind::Ident` は `Ident(String)` なので `ref` は不要:

```rust
TokenKind::Ident(name) if name == "opaque" => {
    self.advance(); // consume "opaque" identifier
    // next must be `type` keyword — parse_type_def expects Type token
    let mut td = self.parse_type_def(vis)?;
    td.is_opaque = true;
    Ok(Item::TypeDef(td))
}
```

### Step 4 — error_catalog.rs: E0413 エントリ追加

予約コメント行（`// ── E0413〜E0419: 予約` を検索）を以下に置き換える:

```rust
// ── E0413: opaque type coerce (v43.11.0) ──────────────────────────────────
ErrorEntry {
    code: "E0413",
    title: "opaque type coerce forbidden",
    category: "types",
    description: "A value of the inner type is used directly as an opaque type. \
                  Opaque types require explicit construction to prevent accidental coercion.",
    example: "opaque type Token = String\nfn bad() -> Token { \"secret\" }  // E0413",
    fix: "Use an explicit constructor function instead of a bare literal.",
},
// ── E0414〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```

### Step 5 — driver.rs: `check_opaque_coerce_violations` + `is_bare_inner_literal` 追加

`collect_explain_output` の直後、`cmd_check` の前に追加:

```rust
/// v43.11.0: opaque type coerce チェック（AST レベル）。
/// opaque alias を返す fn で body が inner type のリテラルである場合を E0413 として返す。
/// スコープ: return_ty が opaque alias かつ body が Expr::Lit(Lit::Str(_), _) のケースのみ（MVP）。
pub fn check_opaque_coerce_violations(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::{Item, TypeBody, TypeExpr, Expr, Lit};
    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    // opaque alias map: name -> inner type name (only simple Named aliases)
    let mut opaque_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for item in &program.items {
        if let Item::TypeDef(td) = item {
            if td.is_opaque {
                if let TypeBody::Alias(TypeExpr::Named(inner_name, params)) = &td.body {
                    if params.is_empty() {
                        opaque_aliases.insert(td.name.clone(), inner_name.clone());
                    }
                }
            }
        }
    }
    if opaque_aliases.is_empty() {
        return vec![];
    }
    let mut violations = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            if let Some(TypeExpr::Named(ret_name, ret_params)) = &fd.return_ty {
                if ret_params.is_empty() {
                    if let Some(inner) = opaque_aliases.get(ret_name) {
                        if is_bare_inner_literal(&fd.body.expr, inner) {
                            violations.push(format!(
                                "{}:{}: E0413: opaque type coerce forbidden: \
                                 cannot return {} as opaque type {}",
                                filename, fd.span.line, inner, ret_name
                            ));
                        }
                    }
                }
            }
        }
    }
    violations
}

/// body expr が inner type のベアリテラルかどうかを判定する（MVP: String のみ対応）
fn is_bare_inner_literal(expr: &crate::ast::Expr, inner_type: &str) -> bool {
    use crate::ast::{Expr, Lit};
    match expr {
        Expr::Lit(Lit::Str(_), _) => inner_type == "String",
        _ => false,
    }
}
```

### Step 6 — driver.rs: `get_explain_text` に E0413 追加

E0412 エントリの直後に追加:

```rust
"E0413" => Some(
    "An opaque type cannot be constructed from its inner type directly. \
     Use an explicit constructor to cross the opaque boundary."
),
```

### Step 7 — driver.rs: `cmd_check` に opaque チェック追加

**挿入位置**: `if errors.is_empty()` ブランチの内側、`"no errors found"` 表示の直前に追加する。
型チェックエラーがある場合（`else` ブランチ）は opaque チェックをスキップする。

```rust
if errors.is_empty() {
    // v43.11.0: opaque type coerce check (E0413) — 型チェック通過後のみ実行
    let opaque_violations = check_opaque_coerce_violations(&source, path);
    if !opaque_violations.is_empty() {
        for v in &opaque_violations {
            eprintln!("{}", v);
        }
        process::exit(1);
    }
    if !show_types && !show_effects && !show_inference {
        println!("{}: no errors found", path);
    }
} else {
    // existing error handling
    ...
}
```

### Step 8 — driver.rs: `v431100_tests` 追加・スタブ化

`v431000_tests` モジュールの直前に `v431100_tests` を挿入（3 件）。
`v431000_tests::cargo_toml_version_is_43_10_0` をスタブ化（既存パターンに従う）:
`// Stubbed: version bumped to 43.11.0 in v43.11.0.`

### Step 9 — Cargo.toml / ロードマップ更新

- `43.10.0` → `43.11.0`
- `roadmap-v43.1-v44.0.md` の v43.11.0 エントリ: checker.fav 記述を「AST レベル MVP / checker.fav 統合は将来版」に修正

---

## T1/T2 アトミック適用

ast.rs の `TypeDef` 変更と parser.rs の 4 箇所更新は必ず同時適用すること（コンパイルエラー防止）。

---

## テスト設計

```rust
// v431100_tests — v43.11.0 の 3 件テスト

#[test]
fn cargo_toml_version_is_43_11_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("43.11.0"), "Cargo.toml must contain version 43.11.0");
}

#[test]
fn parser_recognizes_opaque_type_keyword() {
    use crate::ast::{Item, TypeBody};
    use crate::frontend::parser::Parser;
    let src = "opaque type Token = String";
    let prog = Parser::parse_str(src, "v431100_test.fav").expect("parse opaque type");
    let Item::TypeDef(td) = &prog.items[0] else {
        panic!("expected TypeDef")
    };
    assert!(td.is_opaque, "TypeDef must be marked is_opaque");
    assert_eq!(td.name, "Token");
    assert!(matches!(td.body, TypeBody::Alias(_)), "body must be Alias");
}

#[test]
fn e0413_opaque_coerce_blocked() {
    let src = r#"
opaque type Token = String
fn make_bad() -> Token { "secret" }
"#;
    let violations = super::check_opaque_coerce_violations(src, "v431100_e0413.fav");
    assert!(!violations.is_empty(), "E0413 violation expected: {:?}", violations);
    assert!(
        violations.iter().any(|v| v.contains("E0413")),
        "E0413 expected in violations: {:?}", violations
    );
}
```
