# v36.8.0 spec — `fav schema diff`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.8.0 |
| テーマ | `fav schema diff` — スキーマ差分と後方互換性チェック |
| 前提 | v36.7.0 COMPLETE — Great Expectations 互換エクスポート実装済み |
| 完了条件 | `v36800_tests` 全テスト pass・`cargo test` 0 failures（≥ 2692 件） |

## 背景と目的

`schema Orders { ... }` 定義が複数バージョン間で変化した際、**どのフィールドが追加・削除・型変更されたか**と
**後方互換性への影響**を即座に把握できるツールが必要。

`fav schema diff old.fav new.fav` を実行することで:
- 追加フィールド（後方互換）
- 削除フィールド（BREAKING）
- 型変更フィールド（BREAKING）

を一覧表示する。

## 出力形式

```
schema Orders (old.fav → new.fav):
  + amount: Float         (added, backward-compatible)
  - status: String        (BREAKING: removed)
  ~ id: String -> Int     (BREAKING: type changed)
```

`+` 追加 / `-` 削除 / `~` 型変更。差分なし時は `schema Orders: no changes` を表示。
old.fav または new.fav に存在しないスキーマ名は「added schema」または「removed schema」として表示。

## 実装スコープ

### 1. `fav/src/driver.rs` — `schema_diff` 純粋関数

`validate_contract_file` の後（`// ── fav contract check` セクション末尾の後）に追加:

```rust
// ── fav schema diff (v36.8.0) ──────────────────────────────────────────────

/// TypeExpr を Span なしの文字列に変換するヘルパー。
/// TypeExpr の PartialEq / Debug は Span（file/line/col）を含むため
/// クロスファイル比較には使えない。このヘルパーは型の構造のみを文字列化する。
fn type_expr_kind(ty: &crate::ast::TypeExpr) -> String {
    use crate::ast::TypeExpr;
    match ty {
        TypeExpr::Named(name, args, _) => {
            if args.is_empty() {
                name.clone()
            } else {
                format!("{}<{}>", name, args.iter().map(type_expr_kind).collect::<Vec<_>>().join(", "))
            }
        }
        TypeExpr::Optional(inner, _) => format!("{}?", type_expr_kind(inner)),
        TypeExpr::Fallible(inner, _) => format!("{}!", type_expr_kind(inner)),
        TypeExpr::Arrow(a, b, _) => format!("{} -> {}", type_expr_kind(a), type_expr_kind(b)),
        TypeExpr::TrfFn { input, output, .. } => format!("{} => {}", type_expr_kind(input), type_expr_kind(output)),
        TypeExpr::Intersection(a, b, _) => format!("{} & {}", type_expr_kind(a), type_expr_kind(b)),
        TypeExpr::RecordType(fields, _) => {
            let f: Vec<String> = fields.iter().map(|(n, t)| format!("{}: {}", n, type_expr_kind(t))).collect();
            format!("{{ {} }}", f.join(", "))
        }
        TypeExpr::Schema(s, _) => format!("schema \"{}\"", s),
        TypeExpr::LinearArrow(a, b, _) => format!("{} -o {}", type_expr_kind(a), type_expr_kind(b)),
        TypeExpr::ConstInt(n, _) => n.to_string(),
    }
}

/// 2 つの .fav ソース文字列を受け取り、スキーマ差分行のリストを返す純粋関数。
/// 型比較は `type_expr_kind` ヘルパーで Span を除外した文字列で行う。
pub fn schema_diff(old_src: &str, new_src: &str, old_file: &str, new_file: &str) -> Vec<String> {
    use crate::ast::{Item, SchemaDef};

    let parse = |src: &str, file: &str| -> Vec<SchemaDef> {
        match Parser::parse_str(src, file) {
            Ok(prog) => prog
                .items
                .into_iter()
                .filter_map(|item| if let Item::SchemaDef(s) = item { Some(s) } else { None })
                .collect(),
            Err(_) => vec![],
        }
    };

    let old_defs = parse(old_src, old_file);
    let new_defs = parse(new_src, new_file);

    let old_names: Vec<&str> = old_defs.iter().map(|s| s.name.as_str()).collect();
    let new_names: Vec<&str> = new_defs.iter().map(|s| s.name.as_str()).collect();

    let mut lines: Vec<String> = Vec::new();

    // 削除されたスキーマ
    for old in &old_defs {
        if !new_names.contains(&old.name.as_str()) {
            lines.push(format!("schema {} (BREAKING: removed in {})", old.name, new_file));
        }
    }
    // 追加されたスキーマ
    for new in &new_defs {
        if !old_names.contains(&new.name.as_str()) {
            lines.push(format!("schema {} (added in {})", new.name, new_file));
        }
    }
    // 共通スキーマの差分
    for old in &old_defs {
        if let Some(new) = new_defs.iter().find(|s| s.name == old.name) {
            let old_fields: Vec<&str> = old.fields.iter().map(|(n, _)| n.as_str()).collect();
            let new_fields: Vec<&str> = new.fields.iter().map(|(n, _)| n.as_str()).collect();

            let mut diff: Vec<String> = Vec::new();

            // 削除フィールド
            for (name, ty) in &old.fields {
                if !new_fields.contains(&name.as_str()) {
                    diff.push(format!("  - {}: {}        (BREAKING: removed)", name, type_expr_kind(ty)));
                }
            }
            // 追加フィールド
            for (name, ty) in &new.fields {
                if !old_fields.contains(&name.as_str()) {
                    diff.push(format!("  + {}: {}        (added, backward-compatible)", name, type_expr_kind(ty)));
                }
            }
            // 型変更フィールド（Span を除外した型文字列で比較）
            for (name, old_ty) in &old.fields {
                if let Some((_, new_ty)) = new.fields.iter().find(|(n, _)| n == name) {
                    let old_kind = type_expr_kind(old_ty);
                    let new_kind = type_expr_kind(new_ty);
                    if old_kind != new_kind {
                        diff.push(format!(
                            "  ~ {}: {} -> {}  (BREAKING: type changed)",
                            name, old_kind, new_kind
                        ));
                    }
                }
            }

            if diff.is_empty() {
                lines.push(format!("schema {}: no changes", old.name));
            } else {
                lines.push(format!("schema {} ({} → {}):", old.name, old_file, new_file));
                lines.extend(diff);
            }
        }
    }

    lines
}
```

### 2. `fav/src/driver.rs` — `cmd_schema_diff` コマンド関数

`schema_diff` の後に追加:

```rust
pub fn cmd_schema_diff(old_file: Option<&str>, new_file: Option<&str>) {
    let old_path = old_file.unwrap_or_else(|| {
        eprintln!("error: old schema file is required");
        process::exit(1);
    });
    let new_path = new_file.unwrap_or_else(|| {
        eprintln!("error: new schema file is required");
        process::exit(1);
    });
    let old_src = load_file(old_path);
    let new_src = load_file(new_path);
    let lines = schema_diff(&old_src, &new_src, old_path, new_path);
    for line in &lines {
        println!("{}", line);
    }
}
```

### 3. `fav/src/main.rs` — `Some("schema")` ルーティング追加

`cmd_schema_diff` を import に追加し、`Some("contract") =>` の後に追加:

```rust
Some("schema") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("diff") => {
            let old_file = args.get(3).map(|s| s.as_str());
            let new_file = args.get(4).map(|s| s.as_str());
            cmd_schema_diff(old_file, new_file);
        }
        sub => {
            eprintln!(
                "error: unknown subcommand `{}` for `fav schema`\n  usage: fav schema diff <old.fav> <new.fav>",
                sub.unwrap_or("(none)")
            );
            process::exit(1);
        }
    }
}
```

HELP 定数に追記:
```
schema diff <old.fav> <new.fav>
              Show field-level diff between two schema files.
              Marks added fields as backward-compatible, removed/type-changed as BREAKING.
```

### 4. `fav/src/driver.rs` — `v36800_tests` テストモジュール

```rust
// ── v36800_tests (v36.8.0) — fav schema diff ─────────────────────────────────
#[cfg(test)]
mod v36800_tests {
    use super::schema_diff;

    #[test]
    fn cargo_toml_version_is_36_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.8.0"), "Cargo.toml must contain version 36.8.0");
    }
    #[test]
    fn changelog_has_v36_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.8.0]"), "CHANGELOG.md must contain [v36.8.0]");
    }
    #[test]
    fn schema_diff_detects_added_field() {
        let old = "schema Orders { id: Int }";
        let new = "schema Orders { id: Int, amount: Float }";
        let lines = schema_diff(old, new, "old.fav", "new.fav");
        let joined = lines.join("\n");
        assert!(joined.contains("+ amount"), "should detect added field `amount`");
        assert!(joined.contains("backward-compatible"), "added field should be marked backward-compatible");
    }
    #[test]
    fn schema_diff_detects_removed_field() {
        let old = "schema Orders { id: Int, status: String }";
        let new = "schema Orders { id: Int }";
        let lines = schema_diff(old, new, "old.fav", "new.fav");
        let joined = lines.join("\n");
        assert!(joined.contains("- status"), "should detect removed field `status`");
        assert!(joined.contains("BREAKING"), "removed field should be marked BREAKING");
    }
    #[test]
    fn schema_diff_no_changes() {
        let src = "schema Orders { id: Int }";
        let lines = schema_diff(src, src, "old.fav", "new.fav");
        let joined = lines.join("\n");
        assert!(joined.contains("no changes"), "identical schemas should report no changes");
    }
    #[test]
    fn schema_diff_detects_type_changed_field() {
        let old = "schema Orders { id: String }";
        let new = "schema Orders { id: Int }";
        let lines = schema_diff(old, new, "old.fav", "new.fav");
        let joined = lines.join("\n");
        assert!(joined.contains("~ id"), "should detect type change on field `id`");
        assert!(joined.contains("BREAKING"), "type change should be marked BREAKING");
        assert!(joined.contains("String"), "should show old type");
        assert!(joined.contains("Int"), "should show new type");
    }
}
```

## 注意事項

### `TypeExpr` の Span 問題と `type_expr_kind` ヘルパー

`TypeExpr` は `#[derive(PartialEq)]` を持つが、各バリアントに `Span`（`file: String`, `line`, `col` を含む）が埋め込まれている。
クロスファイル比較（old.fav vs new.fav）では同一の型でも Span が異なるため、`PartialEq` も `format!("{:?}", ty)` も正しく比較できない。
このため `type_expr_kind` ヘルパーで Span を除外した型構造文字列を生成し比較する。
`type_expr_kind` は `TypeExpr` の全バリアントをパターンマッチして再帰的に文字列化する。

### `schema_diff` は純粋関数

ファイル I/O を行わない。`cmd_schema_diff` が `load_file` でソースを読み込み、`schema_diff` に渡す。
これによりテストが副作用なしで実行できる。

### `parse` クロージャと `use crate::ast::{Item, SchemaDef}`

driver.rs のモジュールスコープは `use crate::ast;` のみ（glob インポートなし）。
`schema_diff` 関数内で `use crate::ast::{Item, SchemaDef};` をローカル宣言する。
`Item` のみでは `Vec<SchemaDef>` の型推論が失敗するため **両方** をインポートする。

### スコープ外（v36.9.0 以降）

- `where` 制約の差分検出
- 複数スキーマ間の依存関係解析
- `--json` 出力形式

## ロードマップとの整合

ロードマップ v36.8.0 完了条件:「Rust テスト 2 件」
本 spec では 6 テストを追加する（ロードマップの最小要件 2 件を上回る）。
ロードマップの完了条件に記載の件数（2 件）は更新しない（最小要件値として維持）。
完了ステータス（✅）の更新は tasks.md T8 で行う。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `schema_diff` が追加フィールドを検出する | `schema_diff_detects_added_field` テスト |
| 2 | `schema_diff` が削除フィールドを BREAKING として検出する | `schema_diff_detects_removed_field` テスト |
| 3 | 変更なし時に `no changes` を返す | `schema_diff_no_changes` テスト |
| 4 | 型変更フィールドを BREAKING として検出する | `schema_diff_detects_type_changed_field` テスト |
| 5 | `CHANGELOG.md` に `[v36.8.0]` が含まれる | `changelog_has_v36_8_0` テスト |
| 6 | `Cargo.toml` バージョンが `36.8.0` | `cargo_toml_version_is_36_8_0` テスト |
| 7 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2692） | `cargo test` 実行結果（v36.7.0 実績 2689 + v36800_tests 6 件 = 2695 ≥ 2692） |
