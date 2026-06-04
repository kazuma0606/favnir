# Favnir v10.5.0 Plan

Date: 2026-06-04
Theme: Snowflake × Favnir pipeline — E2E コンパイル確認

---

## Phase A: テスト追加（`driver.rs` 末尾に `v10500_tests` を追加）

```rust
// ── v10500_tests (v10.5.0) — Snowflake × Favnir pipeline compile ─────────────
#[cfg(test)]
mod v10500_tests {
    #[test]
    fn snowflake_compiles_with_favnir_pipeline() {
        // Favnir pipeline (compiler.fav) で Snowflake.execute_raw を含むソースがコンパイルできること
        let src = r#"
fn run(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.execute_raw(sql)
}
"#;
        let result = crate::compiler_fav_runner::compile_src_str_to_bytes(src);
        assert!(result.is_ok(), "Snowflake compile via Favnir pipeline failed: {:?}", result);
    }

    #[test]
    fn snowflake_query_compiles_with_favnir_pipeline() {
        // Favnir pipeline で Snowflake.query_raw を含むソースがコンパイルできること
        let src = r#"
fn query(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.query_raw(sql)
}
"#;
        let result = crate::compiler_fav_runner::compile_src_str_to_bytes(src);
        assert!(result.is_ok(), "Snowflake query compile via Favnir pipeline failed: {:?}", result);
    }
}
```

---

## Phase B: バージョン更新

### B-1: `fav/Cargo.toml` version → `"10.5.0"`

### B-2: `fav/self/cli.fav` の `run_version` → `"10.5.0"`

---

## Phase C: self-check + cargo test

### C-1: `fav check --legacy-check self/compiler.fav` — warning のみ（エラーなし）

### C-2: `cargo test v10500` — 2 件通過

### C-3: `cargo test bootstrap` — 通過確認

```bash
cargo test bootstrap
```

bootstrap 系テストが通ることで、compiler.fav 自身のコンパイル一貫性を確認。

### C-4: `cargo test` — 全件通過（目標: 1271 件 = 1269 + 2 新規）
