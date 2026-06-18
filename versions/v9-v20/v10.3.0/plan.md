# Favnir v10.3.0 Plan

Date: 2026-06-04
Theme: Effect::Snowflake 追加（8 ファイル更新）

---

## Phase A: ast.rs + parser.rs + fmt.rs + lineage.rs

### A-1: `ast.rs` — Effect::Snowflake 追加

`fav/src/ast.rs` の `Effect` 列挙体（line 29 付近）、`Llm` の直後に追加:

```rust
    Llm,
    Snowflake,  // ← 追加
```

### A-2: `parser.rs` — トークン解析追加

`fav/src/frontend/parser.rs` の `"Llm"` ブランチ（line 1170 付近）の直後:

```rust
                        "Snowflake" => {
                            self.advance();
                            Effect::Snowflake
                        }
```

### A-3: `fmt.rs` — 文字列変換追加

`fav/src/fmt.rs` の Effect 変換（line 710 付近）、`Llm` の直後:

```rust
        Effect::Snowflake => Some("!Snowflake".to_string()),
```

### A-4: `lineage.rs` — lineage 出力追加

`fav/src/lineage.rs`（line 58 付近）、`Llm` の直後:

```rust
            Snowflake => "!Snowflake".into(),
```

---

## Phase B: driver.rs

`fav/src/driver.rs` には Effect 変換が 2 箇所ある（line 10244/10267 付近）。

### B-1: 表示用変換（`"!Snowflake"` 形式）

```rust
            Http      => "!Http".into(),
            Llm       => "!Llm".into(),
            Snowflake => "!Snowflake".into(),  // ← 追加
```

### B-2: 短縮名変換（`"Snowflake"` 形式）

```rust
            ast::Effect::Http      => "Http".into(),
            ast::Effect::Llm       => "Llm".into(),
            ast::Effect::Snowflake => "Snowflake".into(),  // ← 追加
```

---

## Phase C: ast_lower_checker.rs + reachability.rs

### C-1: `ast_lower_checker.rs` — lowering 変換追加

`fav/src/middle/ast_lower_checker.rs`（line 359 付近）、`Llm` の直後:

```rust
        ast::Effect::Snowflake => "Snowflake".to_string(),
```

### C-2: `reachability.rs` — 到達可能性解析追加

`fav/src/middle/reachability.rs`（line 81 付近）、`Llm` ブランチの直後:

```rust
                Effect::Snowflake => {
                    effects_required.insert("Snowflake".to_string());
                }
```

---

## Phase D: checker.rs — 型チェック本体

### D-1: builtin NS ホワイトリスト（1 箇所目、〜line 1256）

```rust
            "Http",
            "Llm",
            "Snowflake",  // ← 追加
```

### D-2: builtin NS ホワイトリスト（2 箇所目、〜line 2124）

```rust
            "Http",
            "Llm",
            "Snowflake",  // ← 追加
```

### D-3: effects ホワイトリスト（2 箇所、〜line 4513/4525）

```rust
                        | "Http"
                        | "Llm"
                        | "Snowflake"  // ← 追加
```

### D-4: `require_snowflake_effect` 関数を `require_llm_effect` の直後に追加

```rust
    fn require_snowflake_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Snowflake)) {
            self.type_error(
                "E0314",
                "Snowflake.* call requires `!Snowflake` effect on enclosing fn/stage",
                span,
            );
        }
    }
```

### D-5: Snowflake.* 型シグネチャを Llm セクションの直後に追加

```rust
            // Snowflake (v10.3.0) — require !Snowflake effect
            ("Snowflake", "execute_raw") => {
                self.require_snowflake_effect(span);
                Some(Type::Result(
                    Box::new(Type::String),
                    Box::new(Type::String),
                ))
            }
            ("Snowflake", "query_raw") => {
                self.require_snowflake_effect(span);
                Some(Type::Result(
                    Box::new(Type::String),
                    Box::new(Type::String),
                ))
            }
```

---

## Phase E: error_catalog.rs

### E-1: E0314 エントリを E0313 の直後に追加

```rust
    ErrorEntry {
        code: "E0314",
        title: "undeclared !Snowflake effect",
        category: "effects",
        description: "A Snowflake operation was used in a function that does not declare `!Snowflake`.",
        example: "fn run(sql: String) -> String {\n    Snowflake.execute_raw(sql)  // E0314: !Snowflake not declared\n}",
        fix: "Add `!Snowflake` to the function signature: `fn run(sql: String) -> String !Snowflake`.",
    },
```

---

## Phase F: テスト追加

`fav/src/driver.rs` の末尾に `v10300_tests` モジュールを追加。

```rust
// ── v10300_tests (v10.3.0) — !Snowflake effect ───────────────────────────────
#[cfg(test)]
mod v10300_tests {
    use super::*;

    #[test]
    fn snowflake_execute_requires_effect() {
        let src = r#"
fn run(sql: String) -> Result<String, String> {
  Snowflake.execute_raw(sql)
}
"#;
        let errors = check_source(src);
        let has_e0314 = errors.iter().any(|e| e.contains("E0314"));
        assert!(has_e0314, "expected E0314 for missing !Snowflake: {:?}", errors);
    }

    #[test]
    fn snowflake_execute_with_effect_ok() {
        let src = r#"
fn run(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.execute_raw(sql)
}
"#;
        let errors = check_source(src);
        let e0314 = errors.iter().filter(|e| e.contains("E0314")).collect::<Vec<_>>();
        assert!(e0314.is_empty(), "unexpected E0314 with !Snowflake: {:?}", e0314);
    }

    #[test]
    fn snowflake_lineage_shows_effect() {
        let src = r#"
stage RunQuery: String -> String !Snowflake = |sql| {
  match Snowflake.query_raw(sql) {
    Ok(json) -> json
    Err(e)   -> e
  }
}
seq Pipeline = RunQuery
"#;
        let text = explain_lineage_source(src);
        assert!(text.contains("!Snowflake"), "expected !Snowflake in lineage: {}", text);
    }
}
```

---

## Phase G: バージョン更新 + self-check + cargo test

### G-1: `fav/Cargo.toml` version → `"10.3.0"`

### G-2: `fav/self/cli.fav` の `run_version` → `"10.3.0"`

### G-3: self-check

```bash
fav check --legacy-check self/compiler.fav
fav check self/checker.fav
```

### G-4: `cargo test`

全件通過確認（目標: 1267 件 = 1264 + 3 新規テスト）。
