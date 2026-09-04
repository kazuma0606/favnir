# Plan: v92.9.0 — 安定化・コードフリーズ

Status: TODO

---

## 実装ステップ

### Step 1: 着手前ベースライン確認

`cargo test` を実行し、4,114 tests, 0 failures であることを確認する。

### Step 2: `mod v92900_tests` を `driver.rs` に追加

`mod v92800_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v92900_tests {
    #[test]
    fn query_builder_smoke_all_chains() {
        let src = std::fs::read_to_string("../runes/sap-odata/query_builder.fav").unwrap();
        assert!(src.contains("with_select"), "with_select missing");
        assert!(src.contains("with_expand"), "with_expand missing");
        assert!(src.contains("with_filter"), "with_filter missing");
        assert!(src.contains("with_top"),    "with_top missing");
        assert!(src.contains("with_skip"),   "with_skip missing");
        assert!(src.contains("with_order_by"), "with_order_by missing");
    }

    #[test]
    fn query_builder_page_type_in_rune_dir() {
        let path = std::path::Path::new("../runes/sap-odata/query_builder.fav");
        assert!(path.exists(), "query_builder.fav not found");
        let src = std::fs::read_to_string(path).unwrap();
        assert!(src.contains("Page"), "Page type missing from query_builder.fav");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,116 tests, 0 failures を確認する。

### Step 4: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
