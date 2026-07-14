# v44.2.0 Plan — CEP x Refinement type

## 前提

- 現行バージョン: `44.1.0`（2944 tests）
- 追加テスト数: 3 件
- 目標テスト数: 2947
- ロードマップ推定（2937）は旧見積もり。実績 2944 を基準とする

---

## AST 確認事項（実装前に確認済み）

- `CepPatternDef { name: String, body: Vec<CepClause>, span: Span }` — `Item::CepPatternDef`
- `CepClause { expr: CepExpr, within_secs: Option<i64>, span: Span }` — 各節
- `CepExpr::Event(String)` — 単純イベント名（型パラメータなし）
- `CepExpr::Seq(Vec<CepExpr>)` / `Any(Vec<CepExpr>)` / `Not(Box<CepExpr>)` — 複合
- `TypeDef.invariants: Vec<Expr>` — 非空なら refinement type
- `CepExpr` は `crate::ast::CepExpr` として import

---

## ステップ

### Step 1: driver.rs — `collect_cep_refinement_event_refs` 追加

`collect_refinement_stream_bindings` の直後に配置:

```rust
/// v44.2.0: refinement type 名と一致する CEP イベント参照を収集
/// MVP: CepExpr::Event 名と refinement type 名の一致のみ（型パラメータ付きイベントは将来版）
pub fn collect_cep_refinement_event_refs(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::{CepExpr, Item};

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

    if refinement_names.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();

    for item in &program.items {
        if let Item::CepPatternDef(cd) = item {
            for clause in &cd.body {
                collect_cep_expr_refinement_refs(
                    &clause.expr,
                    &cd.name,
                    clause.span.line,
                    filename,
                    &refinement_names,
                    &mut result,
                );
            }
        }
    }

    result
}

fn collect_cep_expr_refinement_refs(
    expr: &crate::ast::CepExpr,
    pattern_name: &str,
    line: u32,
    filename: &str,
    refinement_names: &std::collections::HashSet<String>,
    result: &mut Vec<String>,
) {
    use crate::ast::CepExpr;
    match expr {
        CepExpr::Event(name) => {
            if refinement_names.contains(name) {
                result.push(format!(
                    "{}:{}: {}: {}",
                    filename, line, pattern_name, name
                ));
            }
        }
        CepExpr::Seq(children) | CepExpr::Any(children) => {
            for child in children {
                collect_cep_expr_refinement_refs(
                    child, pattern_name, line, filename, refinement_names, result,
                );
            }
        }
        CepExpr::Not(child) => {
            collect_cep_expr_refinement_refs(
                child, pattern_name, line, filename, refinement_names, result,
            );
        }
    }
}
```

### Step 2: driver.rs — `v44200_tests` 追加 / スタブ化 / Cargo.toml

`v44100_tests` の直前に挿入:

```rust
// -- v44200_tests (v44.2.0) -- CEP x Refinement type --
#[cfg(test)]
mod v44200_tests {
    #[test]
    fn cargo_toml_version_is_44_2_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.2.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn cep_simple_event_matches_refinement_type() {
        let src = r#"
type HighValue = Float where |v| v > 1000.0
cep pattern HighValueDetected {
  HighValue within 300
}
"#;
        let hits = super::collect_cep_refinement_event_refs(src, "v44200_test.fav");
        assert!(!hits.is_empty(), "expected CEP refinement ref, got: {:?}", hits);
        assert!(
            hits.iter().any(|h| h.contains("HighValueDetected") && h.contains("HighValue")),
            "expected 'HighValueDetected: HighValue' in hits: {:?}", hits
        );
    }
    #[test]
    fn cep_seq_pattern_refinement_event_detected() {
        let src = r#"
type HighValue = Float where |v| v > 1000.0
cep pattern HighValuePurchase {
  seq(Login, HighValue) within 300
}
"#;
        let hits = super::collect_cep_refinement_event_refs(src, "v44200_test.fav");
        assert!(!hits.is_empty(), "expected CEP seq refinement ref, got: {:?}", hits);
        assert!(
            hits.iter().any(|h| h.contains("HighValuePurchase") && h.contains("HighValue")),
            "expected 'HighValuePurchase: HighValue' in hits: {:?}", hits
        );
    }
}
```

スタブ化: `v44100_tests::cargo_toml_version_is_44_1_0` の `assert!` を削除し、以下に置き換える:

```rust
// Stubbed: version bumped to 44.2.0 in v44.2.0.
```

`fav/Cargo.toml` version: `44.1.0` → `44.2.0`

### Step 3: CHANGELOG.md に v44.2.0 エントリ追加

### Step 4: テスト実行（2947 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

---

## 注意事項

- `collect_cep_expr_refinement_refs` はプライベートヘルパー（`pub` なし）
- `CepExpr` は `crate::ast::CepExpr` としてインポート
- `CepClause.span.line` で行番号を取得（`CepPatternDef.span` は宣言全体のスパン）
- `seq(Login, HighValue)` の CEP 構文がパーサーで受容されることは既存テスト（v42x）で確認済み
