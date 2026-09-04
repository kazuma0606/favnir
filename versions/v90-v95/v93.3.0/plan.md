# Plan: v93.3.0 — `NavigationProperty` → `ExpandClause` フィールド生成

Status: TODO

---

## 実装ステップ

### Step 1: 着手前ベースライン確認

`cargo test 2>&1 | grep "test result"` を実行し、4,124 tests, 0 failures であることを確認する。

### Step 2: `sap_metadata.rs` に `nav_property_to_favnir_comment` を追加

```rust
/// ナビゲーションプロパティ名のリストを Favnir コメント文字列に変換する（v93.3.0）
pub fn nav_property_to_favnir_comment(nav_names: &[&str]) -> String {
    if nav_names.is_empty() {
        return String::new();
    }
    let mut out = String::from("-- Navigation properties (use with ExpandClause):");
    for name in nav_names {
        out.push_str(&format!("\n-- \"{}\"", name));
    }
    out
}
```

### Step 3: `sap_metadata.rs` に `to_snake_case` 内部ヘルパーと `nav_to_expand_helper_fn` を追加

```rust
/// PascalCase → snake_case 変換（内部ヘルパー）
fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// EntityType 名と NavigationProperty 名から ExpandClause ヘルパー関数文字列を生成する（v93.3.0）
pub fn nav_to_expand_helper_fn(entity_name: &str, nav_name: &str) -> String {
    let snake_entity = to_snake_case(entity_name);
    // "to_" プレフィックスを除去してから snake_case 化
    let nav_body = nav_name.strip_prefix("to_").unwrap_or(nav_name);
    let snake_nav = to_snake_case(nav_body);
    let fn_name = format!("{}_expand_{}", snake_entity, snake_nav);
    format!(
        "fn {}() -> ExpandClause<{}> {{\n    expand_nav<{}>([\"{}\"])\n}}",
        fn_name, entity_name, entity_name, nav_name
    )
}
```

### Step 4: `cargo build` でコンパイル確認

コンパイルエラーがないことを確認する。

### Step 5: `mod v93300_tests` を `driver.rs` に追加

ファイル末尾の `mod v93200_tests { ... }` の直後に追加する:

```rust
#[cfg(test)]
mod v93300_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn nav_property_parser_defined() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("nav_property_to_favnir_comment"),
            "sap_metadata.rs should define nav_property_to_favnir_comment"
        );
    }

    #[test]
    fn nav_property_generates_expand_helper() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("nav_to_expand_helper_fn"),
            "sap_metadata.rs should define nav_to_expand_helper_fn"
        );
    }
}
```

### Step 6: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,126 tests, 0 failures を確認する。

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
