# Favnir v4.1.5 Implementation Plan

## Theme: 型制約システム — `schemas/*.yaml` + コンパイル時検査 + `T.validate` 自動生成

---

## Phase 0: バージョン更新

`fav/Cargo.toml` の version を `"4.1.5"` に更新。
`fav/src/main.rs` のヘルプテキスト・バージョン文字列を更新。

**追加 Cargo 依存**:
```toml
serde_yaml = "0.9"
regex      = "1"
```

`serde_yaml`: `schemas/*.yaml` の読み込みに使用。
`regex`: `pattern` 制約のコンパイル時リテラル検査に使用。

---

## Phase 1: `schemas/*.yaml` の読み込み（driver.rs + 新規モジュール）

### `src/schemas.rs` — 新規モジュール

YAML のデシリアライズ対象となる構造体と、スキャン関数を実装する。

```rust
// src/schemas.rs

use serde::Deserialize;
use std::collections::HashMap;

/// フィールド 1 つ分の制約定義
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FieldConstraints {
    #[serde(default)]
    pub constraints: Vec<String>,    // ["primary_key", "positive"]
    pub max_length:  Option<usize>,
    pub min_length:  Option<usize>,
    pub min:         Option<f64>,
    pub max:         Option<f64>,
    pub pattern:     Option<String>,
    #[serde(default)]
    pub nullable:    bool,
}

/// 型 1 つ分のスキーマ（フィールド名 → 制約）
pub type TypeSchema = HashMap<String, FieldConstraints>;

/// プロジェクト全体のスキーマ（型名 → TypeSchema）
pub type ProjectSchemas = HashMap<String, TypeSchema>;

/// schemas/ ディレクトリをスキャンして ProjectSchemas を返す
pub fn load_schemas(project_root: &std::path::Path) -> ProjectSchemas {
    let dir = project_root.join("schemas");
    if !dir.is_dir() {
        return HashMap::new();
    }
    let mut result = HashMap::new();
    let Ok(entries) = std::fs::read_dir(&dir) else { return result };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") { continue; }
        let Ok(src) = std::fs::read_to_string(&path) else { continue; };
        let Ok(parsed): Result<HashMap<String, TypeSchema>, _> = serde_yaml::from_str(&src)
        else { continue; };
        result.extend(parsed);
    }
    result
}
```

### `main.rs` — `mod schemas;` を追加

`mod schemas;` をモジュールリストに追加。

### `driver.rs` — スキーマのロードと受け渡し

`cmd_check` / `cmd_run` / `cmd_build_schema`（Phase 4）の冒頭で:

```rust
let schemas = schemas::load_schemas(root);
```

`schemas` を `Checker::check_program` に渡すため、`Checker` の引数または構造体フィールドに追加する。

---

## Phase 2: Checker — コンパイル時リテラル検査

### `checker.rs` — `ProjectSchemas` フィールドを追加

```rust
pub struct Checker {
    // 既存フィールド ...
    pub schemas: ProjectSchemas,  // schemas/*.yaml から読み込んだ制約
}
```

`Checker::new` および `check_with_self` に `schemas` を受け取るよう変更する。

### レコードリテラルの検査

`check_expr` の `Expr::RecordLiteral { type_name, fields, span }` アームに制約チェックを追加:

```rust
// 型名に対応するスキーマを取得
if let Some(type_schema) = self.schemas.get(type_name) {
    for (field_name, field_expr) in fields {
        if let Some(fc) = type_schema.get(field_name) {
            self.check_field_constraints(field_name, field_expr, fc, span)?;
        }
    }
}
```

### `check_field_constraints` 実装

```rust
fn check_field_constraints(
    &self,
    field: &str,
    expr: &Expr,
    fc: &FieldConstraints,
    span: Span,
) -> Result<(), TypeError> {
    // リテラルのみ検査（変数・関数呼び出しはスキップ）
    match expr {
        Expr::Int(n) => {
            if fc.constraints.contains(&"positive".into()) && *n <= 0 {
                return Err(type_error_span(span, &format!(
                    "E0510: field '{}' must be positive (got {})", field, n
                )));
            }
            if fc.constraints.contains(&"non_negative".into()) && *n < 0 {
                return Err(type_error_span(span, &format!(
                    "E0510: field '{}' must be non-negative (got {})", field, n
                )));
            }
            if let Some(min) = fc.min {
                if (*n as f64) < min {
                    return Err(type_error_span(span, &format!(
                        "E0511: field '{}' must be >= {} (got {})", field, min, n
                    )));
                }
            }
            if let Some(max) = fc.max {
                if (*n as f64) > max {
                    return Err(type_error_span(span, &format!(
                        "E0511: field '{}' must be <= {} (got {})", field, max, n
                    )));
                }
            }
        }
        Expr::Float(f) => {
            // positive / non_negative / min / max — Int と同様
        }
        Expr::StringLiteral(s) => {
            if let Some(max_len) = fc.max_length {
                if s.len() > max_len {
                    return Err(type_error_span(span, &format!(
                        "E0513: field '{}' exceeds max_length {} (got {})", field, max_len, s.len()
                    )));
                }
            }
            if let Some(min_len) = fc.min_length {
                if s.len() < min_len {
                    return Err(type_error_span(span, &format!(
                        "E0513: field '{}' below min_length {} (got {})", field, min_len, s.len()
                    )));
                }
            }
            if let Some(pat) = &fc.pattern {
                let re = regex::Regex::new(pat).unwrap_or_else(|_| regex::Regex::new("").unwrap());
                if !re.is_match(s) {
                    return Err(type_error_span(span, &format!(
                        "E0512: field '{}' does not match pattern '{}' (got '{}')", field, pat, s
                    )));
                }
            }
        }
        _ => {} // 変数・関数呼び出しはスキップ
    }
    Ok(())
}
```

### `ValidationError` の事前登録

`checker.rs` の初期化処理（`pre_register_types` 相当）に追加:

```rust
// ValidationError = { field: String, constraint: String, value: String }
pre_register_type("ValidationError", vec![
    ("field",      Type::String),
    ("constraint", Type::String),
    ("value",      Type::String),
]);
```

---

## Phase 3: `T.validate` 自動生成（compiler.rs）

`schemas` に登録されている型 `T` に対して、コンパイル開始前に合成 AST ノードを生成する。

### アプローチ: 合成ソースコードを prepend

`compiler.rs` または `driver.rs` で、`schemas` に含まれる型ごとに:

```rust
// T.validate の合成 VM 関数を直接登録する（AST を経由しない）
fn register_validate_fn(compiler: &mut Compiler, type_name: &str, schema: &TypeSchema) {
    // VM 命令列を直接生成して "TypeName.validate" という名前で登録
    // 引数: raw: Map<String, String>
    // 処理: フィールドを取り出し、型変換し、制約を検査し、Result を返す
}
```

**実装ショートカット（v4.1.5）**: 合成 VM 関数の実装は複雑なため、
`vm.rs` に `Validate.run_raw(type_name, raw_map)` という VM プリミティブを追加し、
`T.validate` の呼び出しを `Validate.run_raw("T", raw)` にディスパッチする。

```rust
// vm.rs — Validate.run_raw(type_name, raw_map) アーム
"Validate.run_raw" => {
    let type_name = vm_string(&args[0], "type_name")?;
    let raw       = vm_map(&args[1], "raw")?;
    let schemas   = SCHEMA_REGISTRY.with(|s| s.borrow().clone());
    let Some(schema) = schemas.get(&type_name) else {
        return Ok(ok_vm(VMValue::Record(vec![])));
    };
    let mut errors: Vec<VMValue> = vec![];
    // 各フィールドを検査してエラーを収集
    ...
    if errors.is_empty() {
        // raw_map から Record を構築
        Ok(ok_vm(VMValue::Record(...)))
    } else {
        Ok(err_vm(VMValue::List(errors)))
    }
}
```

### `SCHEMA_REGISTRY` — スレッドローカル

```rust
thread_local! {
    static SCHEMA_REGISTRY: RefCell<ProjectSchemas> = const { RefCell::new(HashMap::new()) };
}

pub fn set_schema_registry(schemas: ProjectSchemas) {
    SCHEMA_REGISTRY.with(|s| *s.borrow_mut() = schemas);
}
```

`cmd_run` / `cmd_check` の冒頭で `set_schema_registry(schemas)` を呼ぶ。

### checker.rs — `T.validate` のシグネチャ登録

型 `T` に `schemas` エントリがある場合、checker が `T.validate` を関数として認識できるよう登録:

```rust
// schemas に含まれる各型について
for type_name in schemas.keys() {
    // T.validate : Map<String, String> -> Result<T, List<ValidationError>>
    register_builtin_fn(&format!("{type_name}.validate"), ...);
}
```

---

## Phase 4: `fav build --schema` — SQL DDL 生成（driver.rs）

### CLI 追加

`main.rs` の引数パースに `--schema` フラグを追加。
`fav build src/types.fav --schema [--out path]` の形式。

### `cmd_build_schema` 実装

```rust
pub fn cmd_build_schema(file: &str, out: Option<&str>, root: &Path) {
    // 1. schemas/*.yaml をロード
    let schemas = schemas::load_schemas(root);

    // 2. file をパースして型定義を収集
    let prog = parse_file(file);
    let type_metas = collect_type_metas(&prog);

    // 3. schemas に含まれる型のみ DDL を生成
    let mut ddl = String::new();
    for (type_name, type_schema) in &schemas {
        if let Some(meta) = type_metas.get(type_name) {
            ddl.push_str(&generate_ddl(type_name, meta, type_schema));
            ddl.push('\n');
        }
    }

    // 4. out が指定されていればファイルに書き込み、なければ stdout
    match out {
        Some(path) => std::fs::write(path, &ddl).expect("write failed"),
        None       => print!("{ddl}"),
    }
}
```

### `generate_ddl` 実装

```rust
fn generate_ddl(type_name: &str, meta: &TypeMeta, schema: &TypeSchema) -> String {
    let table = to_snake_plural(type_name);  // Order → orders
    let mut cols: Vec<String> = vec![];

    for (field_name, field_type) in &meta.fields {
        let fc = schema.get(field_name).cloned().unwrap_or_default();
        let sql_type = favnir_type_to_sql(field_type, &fc);
        let not_null = if fc.nullable { "" } else { " NOT NULL" };
        let checks   = build_check_clauses(field_name, &fc);
        let unique   = if fc.constraints.contains(&"unique".into()) { " UNIQUE" } else { "" };
        let pk       = if fc.constraints.contains(&"primary_key".into()) {
            " PRIMARY KEY AUTOINCREMENT"
        } else { "" };

        cols.push(format!("    {field_name}  {sql_type}{pk}{unique}{not_null}{checks}"));
    }

    format!("CREATE TABLE {table} (\n{}\n);\n", cols.join(",\n"))
}
```

### `to_snake_plural` ユーティリティ

```rust
fn to_snake_plural(name: &str) -> String {
    // PascalCase → snake_case → + "s"
    // UserProfile → user_profiles
    // Order       → orders
    let snake = pascal_to_snake(name);
    if snake.ends_with('s') { snake } else { format!("{snake}s") }
}
```

---

## Phase 5: テスト

### `checker.rs` — コンパイル時制約テスト

`#[cfg(test)]` ブロックに追加:

```rust
#[test]
fn constraint_positive_catches_negative_int() {
    // schemas に positive 制約を持つ型でリテラル -1 → E0510 エラー
}

#[test]
fn constraint_max_length_catches_long_string() {
    // max_length: 5 で 10文字リテラル → E0513 エラー
}

#[test]
fn constraint_pattern_catches_invalid_email() {
    // pattern: "^[a-z]+$" で "BAD!" → E0512 エラー
}

#[test]
fn constraint_no_error_for_variable_value() {
    // 変数経由の値は検査しない → エラーなし
}

#[test]
fn constraint_non_constrained_type_unaffected() {
    // schemas に存在しない型のリテラルは従来通り通る
}
```

### `vm_stdlib_tests.rs` — `T.validate` 実行時テスト

```rust
#[test]
fn validate_ok_when_all_constraints_pass() { ... }

#[test]
fn validate_err_when_positive_violated() { ... }

#[test]
fn validate_err_when_pattern_violated() { ... }

#[test]
fn validate_err_accumulates_multiple_errors() { ... }
```

### `driver.rs` — 統合テスト

```rust
#[test]
fn schema_load_returns_empty_when_no_dir() { ... }

#[test]
fn schema_load_parses_yaml_correctly() { ... }

#[test]
fn cmd_build_schema_generates_create_table() { ... }

#[test]
fn cmd_build_schema_handles_type_without_schema() { ... }

#[test]
fn fav_check_reports_e0510_for_negative_literal() { ... }
```

### リグレッション

- 全既存テスト（797 件）がパスすること

---

## Phase 6: examples + docs

- `fav/examples/schema_demo/src/main.fav` — Order 型 + `schemas/order.yaml` + `Order.validate` の使用例
- `fav/examples/schema_demo/schemas/order.yaml` — 制約定義
- `versions/v4.1.5/spec.md` 作成済み
- `versions/v4.1.5/tasks.md` 更新（全フェーズ完了時）
- `memory/MEMORY.md` を v4.1.5 完了状態に更新

---

## 実装順序と依存関係

```
Phase 0: バージョン更新 + Cargo 依存追加（独立）
Phase 1: schemas.rs + driver.rs ロード — 独立。先に進めると後続が書きやすい
Phase 2: checker.rs 制約チェック — Phase 1 完了後
Phase 3: T.validate 生成 — Phase 1, 2 完了後
Phase 4: fav build --schema — Phase 1 完了後（Phase 2/3 と並行可能）
Phase 5: テスト — Phase 1〜4 完了後
Phase 6: docs — 最後
```

---

## 実装上の注意点

### `serde_yaml` の YAML 形式

`schemas/order.yaml` のトップレベルは `HashMap<String, TypeSchema>` ではなく
`HashMap<String, HashMap<String, FieldConstraints>>` としてデシリアライズする。
`FieldConstraints` の `constraints` フィールドは `Vec<String>` で、`serde(default)` を付けて
省略可能にする。

### Checker への `schemas` 受け渡し

`Checker::check_program` のシグネチャ変更は既存テストに影響する可能性がある。
`schemas` をオプション引数（`Option<&ProjectSchemas>`）にするか、
`Checker::new` で空の `HashMap::new()` をデフォルトにすることで
既存テストを変更せず対応する。

### `SCHEMA_REGISTRY` の初期化タイミング

`vm_stdlib_tests.rs` の既存テストは `cmd_run` を経由しないため、
`SCHEMA_REGISTRY` が空の状態で `Validate.run_raw` が呼ばれる可能性がある。
`Validate.run_raw` は `schemas.get(type_name)` が `None` の場合に
`Err(["no schema for type T"])` ではなく単に `Ok(Record([]))` を返すよう設計し、
既存テストへの影響を最小化する。

### DDL 生成の限界

`fav build --schema` は出発点（ドラフト）を生成するだけであり、
外部キー制約・インデックス・マルチカラム UNIQUE 等は生成しない（v4.2.0 以降で拡張）。
生成された DDL をそのまま本番に使うのではなく、`migrations/` に配置して人間が編集する運用を想定。
