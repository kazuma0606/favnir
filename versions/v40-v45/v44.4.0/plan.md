# v44.4.0 Plan — 型推論 x パイプライン lineage

## 前提

- 現行バージョン: `44.3.0`（2951 tests）
- 追加テスト数: 2 件
- 目標テスト数: 2953
- ロードマップ推定（2942）は旧見積もり。実績 2951 を基準とする

---

## AST 確認事項（実装前に確認済み）

- `TrfDef.body: Block` — stage 定義の本体
- `Stmt::Bind(BindStmt)` — bind 文（`Stmt::Bind(b)` でマッチ）
- `BindStmt.annotated_ty: Option<TypeExpr>` — 型注釈（`bind x: T <- expr` の `T`）
- `BindStmt.pattern: Pattern::Bind(String, Span)` — 変数名（`Pattern::Ident` は存在しない）
- `BindStmt.span: Span` — 行番号（`b.span.line`）
- `format_type_expr(te: &ast::TypeExpr) -> String` — driver.rs 内のプライベート関数（同ファイル内から呼び出し可）

---

## ステップ

### Step 1: driver.rs — `collect_annotated_lineage_bindings` 追加

`collect_opaque_alias_groups` の直後（`bare_inner_literal_line` の直前）に配置:

```rust
/// v44.4.0: ステージ内の型注釈付き bind 束縛を lineage エントリとして収集
/// `fav explain --lineage` への型情報統合の AST レベル MVP。
/// NOTE: TrfDef トップレベル stmts のみ走査（ネスト Block は将来版）。
/// NOTE: LineageEntry 構造体拡張・render_lineage_text 統合は将来版のスコープ。
pub fn collect_annotated_lineage_bindings(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::{Item, Pattern, Stmt};

    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let mut result = Vec::new();

    for item in &program.items {
        if let Item::TrfDef(td) = item {
            // MVP: トップレベル stmts のみ（ネストした if/match 内の bind は将来版）
            for stmt in &td.body.stmts {
                if let Stmt::Bind(b) = stmt {
                    if let Some(ty) = &b.annotated_ty {
                        let name = match &b.pattern {
                            Pattern::Bind(n, _) => n.clone(),
                            _ => "<pattern>".to_string(),
                        };
                        result.push(format!(
                            "{}:{}: {}: {}: {}",
                            filename,
                            b.span.line,
                            td.name,
                            name,
                            format_type_expr(ty),
                        ));
                    }
                }
            }
        }
    }

    result
}
```

### Step 2: driver.rs — `v44400_tests` 追加 / スタブ化 / Cargo.toml

`v44300_tests` の直前に挿入:

```rust
// -- v44400_tests (v44.4.0) -- 型推論 x パイプライン lineage --
#[cfg(test)]
mod v44400_tests {
    #[test]
    fn cargo_toml_version_is_44_4_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.4.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn annotated_lineage_bindings_detected() {
        let src = r#"
stage Validate: List<Float> -> List<Float> = |events| {
  bind valid: Stream<Float> <- events
}
"#;
        let entries = super::collect_annotated_lineage_bindings(src, "v44400_test.fav");
        assert!(!entries.is_empty(), "expected annotated lineage binding, got: {:?}", entries);
        assert!(
            entries.iter().any(|e| e.contains("Validate") && e.contains("valid") && e.contains("Stream<Float>")),
            "expected 'Validate: valid: Stream<Float>' in entries: {:?}", entries
        );
    }
}
```

スタブ化: `v44300_tests::cargo_toml_version_is_44_3_0` の `assert!` 行のみを削除し、以下に置き換える（`#[test]` アトリビュートと関数シグネチャは残す — テスト件数を変えないため）:

```rust
// Stubbed: version bumped to 44.4.0 in v44.4.0.
```

`fav/Cargo.toml` version: `44.3.0` → `44.4.0`

### Step 3: CHANGELOG.md に v44.4.0 エントリ追加

### Step 4: テスト実行（2953 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

---

## 注意事項

- `format_type_expr` は driver.rs のプライベート関数 — `super::` 不要（同モジュール内で直接呼び出し）
- `stage Validate: List<Float> -> List<Float> = |events| { ... }` が正しい stage 構文（`stage Name { }` は無効）
- `Pattern::Bind(n, _)` が正しいバリアント（v44.1.0 で確認済み）
