# v36.3.0 spec — W025 `schema_mismatch` lint ルール

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.3.0 |
| テーマ | W025 `schema_mismatch` lint ルール |
| 前提 | v36.2.0 COMPLETE — `expect` ブロック実装済み |
| 完了条件 | `v36300_tests` 全テスト pass・`cargo test` 0 failures（≥ 2671 件） |

## 背景と目的

v36.1.0 で `schema Orders { id: Int, ... }` インライン定義を追加した。
v36.2.0 で `expect` ブロック構文を追加した。
本バージョンは **スキーマ定義と使用箇所の型整合性を静的に検出する** lint ルール W025 を追加する。

**W025 の対象**:
関数パラメータが `TypeExpr::Named(n)` かつ `n` が既知の `SchemaDef` 名である場合に、
関数本体内の `row.field_name` 形式のフィールドアクセスが、そのスキーマの定義フィールドに存在しない場合に警告する。

```favnir
schema Orders { id: Int, amount: Float }

fn total(row: Orders) -> Float {
  row.nonexistent  // ← W025: field `nonexistent` not found in schema `Orders`
}
```

## 実装スコープ

### 1. `fav/src/lint.rs` — `check_w025_schema_mismatch` 追加

#### ヘルパー: スキーマフィールドマップの収集

```rust
/// `Item::SchemaDef` から schema_name → field_names の Map を構築
fn collect_schema_fields(program: &Program) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    for item in &program.items {
        if let Item::SchemaDef(sd) = item {
            let fields: Vec<String> = sd.fields.iter().map(|(n, _)| n.clone()).collect();
            map.insert(sd.name.clone(), fields);
        }
    }
    map
}
```

#### ヘルパー: フィールドアクセスを再帰収集

```rust
/// block 内の `Expr::FieldAccess(Ident(var), field, span)` を収集
fn collect_field_accesses(
    block: &Block,
    schema_params: &std::collections::HashMap<String, String>,
    out: &mut Vec<(String, String, Span)>, // (var_name, field_name, span)
) {
    for stmt in &block.stmts {
        collect_field_accesses_stmt(stmt, schema_params, out);
    }
    collect_field_accesses_expr(&block.expr, schema_params, out);
}

fn collect_field_accesses_stmt(
    stmt: &Stmt,
    schema_params: &std::collections::HashMap<String, String>,
    out: &mut Vec<(String, String, Span)>,
) {
    match stmt {
        Stmt::Bind(b) => collect_field_accesses_expr(&b.expr, schema_params, out),
        Stmt::Expr(e) => collect_field_accesses_expr(e, schema_params, out),
        Stmt::Chain(c) => collect_field_accesses_expr(&c.expr, schema_params, out),
        Stmt::Yield(y) => collect_field_accesses_expr(&y.expr, schema_params, out),
        Stmt::ForIn(f) => {
            collect_field_accesses_expr(&f.iter, schema_params, out);
            collect_field_accesses(&f.body, schema_params, out);
        }
        Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_field_accesses_expr(g, schema_params, out); }
            collect_field_accesses(&f.body, schema_params, out);
        }
        Stmt::Expect(e) => {
            collect_field_accesses_expr(&e.target, schema_params, out);
            for r in &e.rules { collect_field_accesses_expr(r, schema_params, out); }
        }
    }
}

fn collect_field_accesses_expr(
    expr: &Expr,
    schema_params: &std::collections::HashMap<String, String>,
    out: &mut Vec<(String, String, Span)>,
) {
    match expr {
        Expr::FieldAccess(obj, field, span) => {
            if let Expr::Ident(var_name, _) = obj.as_ref() {
                if schema_params.contains_key(var_name) {
                    out.push((var_name.clone(), field.clone(), span.clone()));
                }
            }
            // 再帰: ネストしたフィールドアクセス (e.g. a.b.c) にも対応
            collect_field_accesses_expr(obj, schema_params, out);
        }
        Expr::Apply(f, args, _) => {
            collect_field_accesses_expr(f, schema_params, out);
            for a in args { collect_field_accesses_expr(a, schema_params, out); }
        }
        Expr::Closure(_, body, _) => collect_field_accesses_expr(body, schema_params, out),
        Expr::Block(b) => collect_field_accesses(b, schema_params, out),
        Expr::If(cond, then, else_, _) => {
            collect_field_accesses_expr(cond, schema_params, out);
            collect_field_accesses(then, schema_params, out);
            if let Some(e) = else_ { collect_field_accesses(e, schema_params, out); }
        }
        _ => {} // リテラル・Ident 等はスキップ
    }
}
```

#### メイン: W025 チェック関数

```rust
// ── W025: schema_mismatch (v36.3.0) ──────────────────────────────────────────
/// フィールドアクセスがスキーマ定義に存在しない場合に W025 を発行する
fn check_w025_schema_mismatch(program: &Program, errors: &mut Vec<LintError>) {
    let schema_fields = collect_schema_fields(program);
    if schema_fields.is_empty() {
        return; // schema 定義なし → スキップ
    }

    for item in &program.items {
        if let Item::FnDef(fd) = item {
            // パラメータのうち schema 型を持つものを収集
            let mut schema_params: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for param in &fd.params {
                if let TypeExpr::Named(type_name, type_args, _) = &param.ty {
                    if type_args.is_empty() && schema_fields.contains_key(type_name) {
                        schema_params.insert(param.name.clone(), type_name.clone());
                    }
                }
            }
            if schema_params.is_empty() {
                continue;
            }

            // 本体からフィールドアクセスを収集
            let mut accesses: Vec<(String, String, Span)> = vec![];
            collect_field_accesses(&fd.body, &schema_params, &mut accesses);

            // 各アクセスがスキーマに存在するか検証
            for (var_name, field_name, span) in accesses {
                let schema_name = &schema_params[&var_name];
                let fields = &schema_fields[schema_name];
                if !fields.contains(&field_name) {
                    errors.push(LintError::new(
                        "W025",
                        format!(
                            "field `{}` not found in schema `{}` (available: {})",
                            field_name,
                            schema_name,
                            fields.join(", ")
                        ),
                        span, // Span は Clone のみ（Copy なし）。Vec から move する
                    ));
                }
            }
        }
    }
}
```

### 2. `lint_program` への呼び出し追加

`check_w021_pure_fn_calls_effectful` の呼び出し行の後に追加:

```rust
// v36.3.0: W025
check_w025_schema_mismatch(program, &mut errors);
```

### 3. `fav/src/driver.rs` — テストモジュール

`v36200_tests::cargo_toml_version_is_36_2_0` をスタブ化し、`v36300_tests` を追加。

## v36300_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_3_0` | Cargo.toml に `"36.3.0"` が含まれる |
| `changelog_has_v36_3_0` | `CHANGELOG.md` に `[v36.3.0]` が含まれる |
| `w025_in_lint_rs` | `lint.rs` に `W025` と `schema_mismatch` が含まれる |
| `w025_schema_mismatch_fires` | 未定義フィールドアクセスで W025 が発行される |
| `w025_schema_mismatch_silent` | 正しいフィールドアクセスでは W025 が発行されない |

### テスト実装

```rust
// ── v36300_tests (v36.3.0) — W025 schema_mismatch ────────────────────────────
#[cfg(test)]
mod v36300_tests {
    use crate::frontend::parser::Parser;
    use crate::lint::lint_program;

    fn parse_lint(src: &str) -> Vec<String> {
        let prog = Parser::parse_str(src, "test.fav").expect("parse");
        lint_program(&prog).iter().map(|e| e.code.clone()).collect()
    }

    #[test]
    fn cargo_toml_version_is_36_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.3.0"), "Cargo.toml must contain version 36.3.0");
    }
    #[test]
    fn changelog_has_v36_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.3.0]"), "CHANGELOG.md must contain [v36.3.0]");
    }
    #[test]
    fn w025_in_lint_rs() {
        let src = include_str!("lint.rs");
        assert!(
            src.contains("W025") && src.contains("schema_mismatch"),
            "lint.rs must contain W025 schema_mismatch implementation"
        );
    }
    #[test]
    fn w025_schema_mismatch_fires() {
        // 前提: v36.1.0 の SchemaDef パーサーが必須。parse_str が Err を返す場合は
        // SchemaDef パーサーのバグであり W025 とは別問題。
        let src = "schema Orders { id: Int }\nfn f(row: Orders) -> Int { row.nonexistent }";
        let codes = parse_lint(src);
        assert!(codes.contains(&"W025".to_string()), "expected W025, got: {:?}", codes);
    }
    #[test]
    fn w025_schema_mismatch_silent() {
        // 前提: v36.1.0 の SchemaDef パーサーが必須。
        let src = "schema Orders { id: Int }\nfn f(row: Orders) -> Int { row.id }";
        let codes = parse_lint(src);
        assert!(!codes.contains(&"W025".to_string()), "expected no W025, got: {:?}", codes);
    }
}
```

## 注意事項

### `LintError` 構造体の確認

W025 のエラー生成は既存の `LintError` 構造体を使う。
`LintError` の実際のフィールド名（`code` / `message` / `span` の型）を `cargo build` で確認してから実装すること。
フィールド名が異なる場合は `LintError::new()` コンストラクタを使用する（他の W コードの生成方法に倣う）。

### スコープ外（v36.4.0 以降）

- `List<Orders>` 形式のパラメータ型からのスキーマ推論（`TypeExpr::Named("List", [Named("Orders")])` の展開）
- `expect` ブロック内のルール式のフィールド検証
- ネストしたフィールドアクセス `row.sub.field` のスキーマ追跡

## ロードマップとの整合

ロードマップ v36.3.0 完了条件:「`fav lint` で W025 が報告される / Rust テスト 2 件」

本 spec では 5 テスト（ロードマップの 2 件を上回る）を追加する。
ロードマップの「テスト N 件」は**最小要件**を示す。spec は常に最小要件を満たしたうえで追加テストを設けてよい。
ロードマップの件数は更新しない（最小要件値として維持）。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `lint.rs` に `check_w025_schema_mismatch` が含まれる | `w025_in_lint_rs` テスト |
| 2 | `CHANGELOG.md` に `[v36.3.0]` が含まれる | `changelog_has_v36_3_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.3.0` | `cargo_toml_version_is_36_3_0` テスト |
| 4 | W025 が未定義フィールドアクセスで発行される | `w025_schema_mismatch_fires` テスト |
| 5 | 正常なフィールドアクセスでは W025 が発行されない | `w025_schema_mismatch_silent` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2671） | `cargo test` 実行結果 |
