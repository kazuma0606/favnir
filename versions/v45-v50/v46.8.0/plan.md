# Plan: v46.8.0 — `fav explain --types`

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `type_expr_str` ヘルパー + `cmd_explain_types` + `v468000_tests` |
| `fav/src/main.rs` | `--types` フラグ追加 |
| `fav/Cargo.toml` | version → `"46.8.0"` |
| `CHANGELOG.md` | v46.8.0 エントリ追加 |
| `versions/current.md` | v46.8.0（3009 tests）に更新 |
| `versions/v45-v50/v46.8.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### `fav/src/driver.rs`

#### 追加 1: `type_expr_str` ヘルパー

`cmd_explain_lineage` の直後（v46.8.0 セクション冒頭）に追加する。

```rust
// ── fav explain --types ────────────────────────────────────────────────────────

/// v46.8.0: TypeExpr → display 文字列に変換する。
/// fmt.rs の fmt_type_expr_simple と同等（private のため独立実装）。
fn type_expr_str(ty: &ast::TypeExpr) -> String {
    match ty {
        ast::TypeExpr::Named(name, args, _) if args.is_empty() => name.clone(),
        ast::TypeExpr::Named(name, args, _) => {
            let as_: Vec<String> = args.iter().map(type_expr_str).collect();
            format!("{}<{}>", name, as_.join(", "))
        }
        ast::TypeExpr::Optional(inner, _) => format!("{}?", type_expr_str(inner)),
        ast::TypeExpr::Fallible(inner, _) => format!("{}!", type_expr_str(inner)),
        ast::TypeExpr::Arrow(from, to, _) => {
            format!("{} -> {}", type_expr_str(from), type_expr_str(to))
        }
        ast::TypeExpr::TrfFn { input, output, .. } => {
            format!("{} -> {}", type_expr_str(input), type_expr_str(output))
        }
        ast::TypeExpr::RecordType(fields, _) => {
            let parts: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("{}: {}", n, type_expr_str(t)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        ast::TypeExpr::Intersection(lhs, rhs, _) => {
            format!("{} & {}", type_expr_str(lhs), type_expr_str(rhs))
        }
        ast::TypeExpr::Schema(uri, _) => format!("schema \"{}\"", uri),
        ast::TypeExpr::LinearArrow(a, b, _) => {
            format!("{} -o {}", type_expr_str(a), type_expr_str(b))
        }
        ast::TypeExpr::ConstInt(n, _) => format!("{}", n),
    }
}
```

#### 追加 2: `format_stage_types` + `cmd_explain_types`

ロジックを純粋関数 `format_stage_types` に切り出し、`cmd_explain_types` は print のみ担当。
テストは `format_stage_types` を直接呼ぶことでファイル I/O 不要。

```rust
/// v46.8.0: program 内の TrfDef 宣言型を `stage Name<T>: In -> Out\n` 形式で返す。
fn format_stage_types(program: &ast::Program) -> String {
    let mut out = String::new();
    let mut found = false;
    for item in &program.items {
        if let ast::Item::TrfDef(trf) = item {
            found = true;
            let type_params = if trf.type_params.is_empty() {
                String::new()
            } else {
                let ps: Vec<String> = trf.type_params.iter().map(|p| p.name.clone()).collect();
                format!("<{}>", ps.join(", "))
            };
            out.push_str(&format!(
                "stage {}{}: {} -> {}\n",
                trf.name,
                type_params,
                type_expr_str(&trf.input_ty),
                type_expr_str(&trf.output_ty),
            ));
        }
    }
    if !found {
        out.push_str("(no stages found)\n");
    }
    out
}

/// `fav explain --types [file]`
/// v46.8.0: パイプライン各ステージの宣言型を出力する。
pub fn cmd_explain_types(file: Option<&str>) {
    let paths: Vec<String> = if let Some(f) = file {
        vec![f.to_string()]
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FavToml::find_root(&cwd).unwrap_or_else(|| {
            eprintln!("error: no fav.toml found");
            process::exit(1);
        });
        let toml = FavToml::load(&root).unwrap_or_else(|| {
            eprintln!("error: could not read fav.toml");
            process::exit(1);
        });
        collect_fav_files(&toml.src_dir(&root))
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    };

    for path in &paths {
        let source = load_file(path);
        let program = Parser::parse_str(&source, path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        });
        if paths.len() > 1 {
            println!("\n=== {} ===", path);
        }
        print!("{}", format_stage_types(&program));
    }
}
```

#### 追加 3: `v468000_tests`

テストは `format_stage_types` を直接呼ぶ（ファイル I/O なし、並列実行安全）。

```rust
#[cfg(test)]
mod v468000_tests {
    use super::*;

    #[test]
    fn explain_types_shows_stage_types() {
        let src = r#"
stage ParseCsv: String -> List<Row> { List.empty() }
stage FilterRows: List<Row> -> List<Row> { input }
stage SaveToDb: List<Row> -> Result<Int> { Ok(0) }
"#;
        let program = Parser::parse_str(src, "test_explain_types.fav").unwrap();
        let out = format_stage_types(&program);
        assert!(out.contains("stage ParseCsv: String -> List<Row>"), "got: {}", out);
        assert!(out.contains("stage FilterRows: List<Row> -> List<Row>"), "got: {}", out);
        assert!(out.contains("stage SaveToDb: List<Row> -> Result<Int>"), "got: {}", out);
    }

    #[test]
    fn explain_types_generic_instantiation() {
        let src = r#"
stage Map<T, U>: List<T> -> List<U> { List.empty() }
"#;
        let program = Parser::parse_str(src, "test_explain_types_gen.fav").unwrap();
        let out = format_stage_types(&program);
        assert!(out.contains("stage Map<T, U>: List<T> -> List<U>"), "got: {}", out);
    }
}
```

---

### `fav/src/main.rs`

`Some("explain")` ブランチの `--lineage` チェック（line ~777）の直前に追加:

```rust
if args.iter().any(|a| a == "--types") {
    let file = args.iter().skip(2).find(|a| !a.starts_with('-')).map(|s| s.as_str());
    cmd_explain_types(file);
    return;
}
```

---

## 実装順序

1. `driver.rs`: `type_expr_str` + `cmd_explain_types` + `v468000_tests` を追加
2. `main.rs`: `--types` フラグを追加
3. `cargo test` で 3009 passed 確認
4. `cargo clippy -- -D warnings` クリーン確認
5. `Cargo.toml` version → `"46.8.0"`
6. `CHANGELOG.md` エントリ追加
7. `versions/current.md` 更新
8. `tasks.md` COMPLETE に更新

---

## 注意事項

- `type_expr_str` は `fmt.rs` の `fmt_type_expr_simple` と同等だが、
  `fmt_type_expr_simple` は private のため driver.rs 内に独立実装する。
  将来的に共通化する場合は `pub(crate)` 化を検討すること。
- `TrfDef.type_params` は `Vec<GenericParam>`。`GenericParam.name` フィールドを使用する。
- `cmd_explain_types` の引数は `file: Option<&str>`（`cmd_explain_lineage` と同パターン）。
