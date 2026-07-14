# v44.1.0 Plan — Refinement type x Streaming 統合

## 前提

- 現行バージョン: `44.0.0`（2941 tests）
- 追加テスト数: 3 件
- 目標テスト数: 2944
- ロードマップ推定（2934）は旧見積もり。実績 2941 を基準とする

---

## AST 確認事項（実装前に確認済み）

- `TypeDef.invariants: Vec<Expr>` — 非空なら refinement type（`where` 節あり）
- `BindStmt.annotated_ty: Option<TypeExpr>` — `bind name: Type <- expr` の型注釈
- `BindStmt.pattern: Pattern` — `Pattern::Bind(name, span)` が変数束縛の名前（`Pattern::Ident` は存在しない）
- `TrfDef.body: Block` — `stage` 定義の本体（`Stmt::Bind` を含む）
- `TypeExpr::Named(name, params, span)` — 3 フィールド（`Stream<T>` → `Named("Stream", [Named("T", ...)], _)`）

---

## ステップ

### Step 1: driver.rs — `collect_refinement_stream_bindings` 追加

`check_opaque_coerce_violations` の直後あたりに配置:

```rust
/// v44.1.0: refinement type がストリーム要素型として型注釈されている bind 束縛を収集
pub fn collect_refinement_stream_bindings(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::{Item, Pattern, Stmt, TypeExpr};

    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    // refinement type 名を収集（invariants 非空の TypeDef）
    let mut refinement_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for item in &program.items {
        if let Item::TypeDef(td) = item {
            if !td.invariants.is_empty() {
                refinement_names.insert(td.name.clone());
            }
        }
    }

    let mut result = Vec::new();

    // FnDef / TrfDef の body.stmts を走査
    let blocks: Vec<&crate::ast::Block> = program.items.iter().filter_map(|item| {
        match item {
            Item::FnDef(fd) => Some(&fd.body),
            Item::TrfDef(td) => Some(&td.body),
            _ => None,
        }
    }).collect();

    for block in blocks {
        for stmt in &block.stmts {
            if let Stmt::Bind(b) = stmt {
                if let Some(TypeExpr::Named(container, params, _)) = &b.annotated_ty {
                    if (container == "Stream" || container == "List") && !params.is_empty() {
                        if let TypeExpr::Named(elem, _, _) = &params[0] {
                            if refinement_names.contains(elem) {
                                let name = match &b.pattern {
                                    Pattern::Bind(n, _) => n.clone(),
                                    _ => "<pattern>".to_string(),
                                };
                                result.push(format!(
                                    "{}:{}: {}: {}<{}>",
                                    filename, b.span.line, name, container, elem
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}
```

### Step 2: driver.rs — `v44100_tests` 追加 / スタブ化 / Cargo.toml

`v44000_tests` の直前に挿入:

```rust
// -- v44100_tests (v44.1.0) -- Refinement type x Streaming 統合 --
#[cfg(test)]
mod v44100_tests {
    #[test]
    fn cargo_toml_version_is_44_1_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.1.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn refinement_type_invariant_in_typedef_ast() {
        use crate::ast::{Item, TypeBody};
        use crate::frontend::parser::Parser;
        let src = "type PositiveFloat = Float where |v| v > 0.0";
        let prog = Parser::parse_str(src, "v44100_test.fav").expect("parse");
        let Item::TypeDef(td) = &prog.items[0] else { panic!("expected TypeDef") };
        assert!(!td.invariants.is_empty(), "PositiveFloat must have invariants");
        assert!(matches!(td.body, TypeBody::Alias(_)), "body must be Alias");
    }
    #[test]
    fn collect_refinement_stream_bindings_detects_annotated_bind() {
        let src = r#"
type PositiveFloat = Float where |v| v > 0.0
stage Validate: List<Float> -> List<Float> = |events| {
  bind valid: Stream<PositiveFloat> <- events
}
"#;
        let hits = super::collect_refinement_stream_bindings(src, "v44100_test.fav");
        assert!(!hits.is_empty(), "expected refinement stream binding, got: {:?}", hits);
        assert!(
            hits.iter().any(|h| h.contains("valid") && h.contains("Stream<PositiveFloat>")),
            "expected 'valid: Stream<PositiveFloat>' in hits: {:?}", hits
        );
    }
}
```

スタブ化: `v44000_tests::cargo_toml_version_is_44_0_0` の `assert!` を削除し、以下のコメントに置き換える:

```rust
// Stubbed: version bumped to 44.1.0 in v44.1.0.
```

`fav/Cargo.toml` version: `44.0.0` → `44.1.0`

### Step 3: CHANGELOG.md に v44.1.0 エントリ追加

### Step 4: テスト実行（2944 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

---

## 注意事項

- `collect_refinement_stream_bindings` は FnDef と TrfDef の 1 レベルのみ走査（ネストした Block は今回スコープ外）
- `bind valid: Stream<PositiveFloat> <- events` — Favnir パーサーが `annotated_ty` に `TypeExpr::Named("Stream", ...)` をセットする構文を確認すること
- `v44000_tests::cargo_toml_version_is_44_0_0` をスタブ化（version 44.1.0 になるため）
