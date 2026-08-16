# v72.3.0 実装プラン — `fav ai generate`

Date: 2026-08-12

---

## 依存関係

```
Step 1 (driver.rs コア関数)
  └─ Step 2 (main.rs CLI)
  └─ Step 3 (v723000_tests)
       └─ Step 4 (バージョン更新)
            └─ Step 5 (テスト確認)
                 └─ Step 6 (ドキュメント更新)
```

---

## Step 1: `driver.rs` — コア関数追加

対象: `fav/src/driver.rs`

### 1-1. `infer_schema_from_description` 追加

```rust
pub fn infer_schema_from_description(description: &str) -> Vec<(&'static str, &'static str)> {
    let lower = description.to_lowercase();
    if lower.contains("order") || description.contains("注文") {
        vec![("order_id", "String"), ("amount", "Float"), ("status", "String")]
    } else {
        vec![("id", "String"), ("value", "String")]
    }
}
```

### 1-2. `cmd_ai_generate` 追加

```rust
pub fn cmd_ai_generate(description: &str) -> String {
    let mut imports = Vec::new();
    let lower = description.to_lowercase();
    if lower.contains("csv") { imports.push("import rune \"csv\""); }
    if lower.contains("postgres") || lower.contains("postgresql") {
        imports.push("import rune \"postgres\"");
    }
    if lower.contains("s3") { imports.push("import rune \"s3\""); }
    if lower.contains("json") { imports.push("import rune \"json\""); }

    let fields = infer_schema_from_description(description);
    let schema_name = if description.contains("order") || description.contains("注文") {
        "OrderRow"
    } else {
        "Row"
    };

    // imports ブロック
    let import_block = if imports.is_empty() {
        String::new()
    } else {
        imports.join("\n") + "\n\n"
    };

    // schema ブロック
    let schema_fields: String = fields
        .iter()
        .map(|(name, ty)| format!("    {name}: {ty}"))
        .collect::<Vec<_>>()
        .join("\n");
    let schema_block = format!("schema {schema_name} {{\n{schema_fields}\n}}\n\n");

    // fn main ブロック
    let main_block = format!(
        "fn main(ctx: AppCtx) -> Result<Unit, String> {{\n    ctx.io.println(\"Generated pipeline for: {description}\")\n}}\n"
    );

    format!("{import_block}{schema_block}{main_block}")
}
```

### 1-3. `cargo build` で確認

---

## Step 2: `main.rs` — CLI サブコマンド追加

対象: `fav/src/main.rs`

- 既存の `Some("ai")` アームに `"generate"` ブランチを追加:

```rust
"generate" => {
    let description: String = args.iter().skip(3).cloned().collect::<Vec<_>>().join(" ");
    let desc = if description.is_empty() { "default pipeline" } else { &description };
    println!("Generating pipeline...\n");
    let code = driver::cmd_ai_generate(desc);
    println!("{code}");
}
```

- `cargo build` で確認

---

## Step 3: `v723000_tests` 追加（`driver.rs`）

`v722000_tests` モジュールの直後に追加:

```rust
#[cfg(test)]
mod v723000_tests {
    use super::{cmd_ai_generate, infer_schema_from_description};

    #[test]
    fn ai_generate_returns_valid_fav_code() {
        let code = cmd_ai_generate("csv pipeline");
        assert!(code.contains("fn main"), "should contain fn main");
        assert!(code.contains("ctx: AppCtx"), "should contain ctx: AppCtx");
    }

    #[test]
    fn ai_generate_schema_inferred_from_description() {
        let code = cmd_ai_generate("注文データのETL");
        assert!(code.contains("order_id"), "should contain order_id field");
    }
}
```

- `cargo build` で確認

---

## Step 4: バージョン更新

- `fav/Cargo.toml`: `version = "72.2.0"` → `version = "72.3.0"`
- `driver.rs` 内 `version = \"72.2.0\"` → `version = \"72.3.0\"`（replace_all）
- `driver.rs` 内 `"Cargo.toml version should be 72.2.0"` → `"72.3.0"`（replace_all）
- `driver.rs` 内 `"Cargo.toml should declare version 72.2.0"` → `"72.3.0"`（replace_all）
- T0 で記録した `"72.2.0"` の件数と、置換後の `"72.3.0"` grep 件数が一致することを確認する

---

## Step 5: テスト確認

- `cargo test v723000` → 2 件 pass
- `cargo test` 全体 → 3618 tests pass（0 failures）

---

## Step 6: ドキュメント更新

- `CHANGELOG.md`: `## [v72.3.0]` エントリを先頭に追加
- `versions/current.md`: 進行中バージョンを v72.3.0、次を v72.4.0 に更新
