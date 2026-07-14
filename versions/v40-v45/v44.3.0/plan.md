# v44.3.0 Plan — Stream join x Opaque type

## 前提

- 現行バージョン: `44.2.0`（2947 tests）
- 追加テスト数: 3 件
- 目標テスト数: 2950
- ロードマップ推定（2940）は旧見積もり。実績 2947 を基準とする

---

## AST 確認事項（実装前に確認済み）

- `TypeDef.is_opaque: bool` — v43.11.0 追加
- `TypeBody::Alias(TypeExpr)` — エイリアス型の本体
- `TypeExpr::Named(String, Vec<TypeExpr>, Span)` — 3 フィールド
- `params.is_empty()` — 型引数なし opaque alias のみ対象
- `check_opaque_coerce_violations` の opaque 収集パターンを参考にする（`driver.rs` 行 3975〜）

---

## ステップ

### Step 1: driver.rs — `collect_opaque_alias_groups` 追加

`collect_cep_expr_refinement_refs` の直後（`bare_inner_literal_line` の直前）に配置:

```rust
/// v44.3.0: 同じ内部型を持つ opaque type エイリアスをグループ化して返す
/// Stream.join で誤 join される可能性のある opaque type ペアを検出するための AST レベル MVP。
/// NOTE: 型引数なし（params.is_empty()）の単純 opaque alias のみ対象。
/// NOTE: checker.fav への E0413 統合は将来版のスコープ。
pub fn collect_opaque_alias_groups(src: &str, filename: &str) -> Vec<String> {
    use crate::ast::{Item, TypeBody, TypeExpr};

    let program = match crate::frontend::parser::Parser::parse_str(src, filename) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    // inner type 名 -> Vec<(opaque_name, line)> のマップを構築
    let mut groups: std::collections::HashMap<String, Vec<(String, u32)>> =
        std::collections::HashMap::new();

    for item in &program.items {
        if let Item::TypeDef(td) = item {
            if td.is_opaque {
                if let TypeBody::Alias(TypeExpr::Named(inner, params, _)) = &td.body {
                    if params.is_empty() {
                        groups
                            .entry(inner.clone())
                            .or_default()
                            .push((td.name.clone(), td.span.line));
                    }
                }
            }
        }
    }

    let mut result = Vec::new();

    // 2 件以上の opaque type が同じ inner type を共有するグループをレポート
    let mut inner_types: Vec<String> = groups.keys().cloned().collect();
    inner_types.sort(); // 安定した出力のためソート

    for inner in &inner_types {
        let entries = &groups[inner];
        if entries.len() >= 2 {
            let first_line = entries.iter().map(|(_, l)| *l).min().unwrap_or(0);
            let mut names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();
            names.sort(); // アルファベット順
            result.push(format!(
                "{}:{}: {}: {}",
                filename,
                first_line,
                inner,
                names.join(", ")
            ));
        }
    }

    result
}
```

### Step 2: driver.rs — `v44300_tests` 追加 / スタブ化 / Cargo.toml

`v44200_tests` の直前に挿入:

```rust
// -- v44300_tests (v44.3.0) -- Stream join x Opaque type --
#[cfg(test)]
mod v44300_tests {
    #[test]
    fn cargo_toml_version_is_44_3_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"44.3.0\""), "Cargo.toml version mismatch");
    }
    #[test]
    fn opaque_alias_group_detected() {
        let src = r#"
opaque type OrderId = String
opaque type PaymentOrderId = String
"#;
        let groups = super::collect_opaque_alias_groups(src, "v44300_test.fav");
        assert!(!groups.is_empty(), "expected opaque alias group, got: {:?}", groups);
        assert!(
            groups.iter().any(|g| g.contains("String") && g.contains("OrderId") && g.contains("PaymentOrderId")),
            "expected 'String: OrderId, PaymentOrderId' in groups: {:?}", groups
        );
    }
    #[test]
    fn non_opaque_type_excluded_from_groups() {
        let src = r#"
type X = String
type Y = String
"#;
        let groups = super::collect_opaque_alias_groups(src, "v44300_test.fav");
        assert!(groups.is_empty(), "non-opaque types must not appear in groups: {:?}", groups);
    }
}
```

スタブ化: `v44200_tests::cargo_toml_version_is_44_2_0` の `assert!` を削除し、以下に置き換える:

```rust
// Stubbed: version bumped to 44.3.0 in v44.3.0.
```

`fav/Cargo.toml` version: `44.2.0` → `44.3.0`

### Step 3: CHANGELOG.md に v44.3.0 エントリ追加

### Step 4: テスト実行（2950 passed; 0 failed）

### Step 5: バージョン管理ドキュメント更新

---

## 注意事項

- `inner_types.sort()` で出力を安定させる（テストの assert が順序に依存しないよう `any()` を使う）
- `entries.iter().map(|(_, l)| *l).min().unwrap_or(0)` — `min()` は `Option` を返すため `unwrap_or(0)` で安全に処理
- `check_opaque_coerce_violations` と異なり、本関数は FnDef 走査を行わない（TypeDef のみ）
