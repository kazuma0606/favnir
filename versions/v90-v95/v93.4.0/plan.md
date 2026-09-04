# Plan: v93.4.0 — `EnumType` → Favnir `type E = | A | B` 変換

Status: TODO

---

## 実装ステップ

### Step 1: 着手前ベースライン確認

`cargo test 2>&1 | grep "test result"` を実行し、4,126 tests, 0 failures であることを確認する。

### Step 2: `sap_metadata.rs` に `EdmxEnumMember` / `EdmxEnumType` 構造体を追加

```rust
/// EDMX EnumType の各メンバー（v93.4.0）
#[derive(Debug)]
pub struct EdmxEnumMember {
    pub name: String,
}

/// EDMX EnumType（列挙型の型定義）（v93.4.0）
#[derive(Debug)]
pub struct EdmxEnumType {
    pub name: String,
    pub members: Vec<EdmxEnumMember>,
}
```

### Step 3: `sap_metadata.rs` に `screaming_snake_to_pascal` と `enum_type_to_favnir` を追加

```rust
/// SCREAMING_SNAKE_CASE → PascalCase 変換（内部ヘルパー）
/// SAFETY: SAP OData EnumType 名は ASCII のみ保証される
fn screaming_snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper = first.to_ascii_uppercase().to_string();
                    upper + &chars.as_str().to_ascii_lowercase()
                }
            }
        })
        .collect()
}

/// EDMX EnumType を Favnir ADT 型定義文字列に変換する（v93.4.0）
/// 例: EdmxEnumType { name: "YY1_BPKIND_CODE", members: [EdmxEnumMember { name: "1" }, ...] }
///   → "type Yy1BpkindCode =\n    | Val1\n    | Val2\n    | Val3"
pub fn enum_type_to_favnir(et: &EdmxEnumType) -> String {
    let type_name = screaming_snake_to_pascal(&et.name);
    let variants: String = et
        .members
        .iter()
        .map(|m| {
            // 先頭が数字の場合は "Val" プレフィックスを付与
            if m.name.starts_with(|c: char| c.is_ascii_digit()) {
                format!("    | Val{}", m.name)
            } else {
                format!("    | {}", m.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("type {} =\n{}", type_name, variants)
}
```

### Step 4: `cargo build` でコンパイル確認

コンパイルエラーがないことを確認する。

### Step 5: `mod v93400_tests` を `driver.rs` に追加

ファイル末尾の `mod v93300_tests { ... }` の直後に追加する:

```rust
#[cfg(test)]
mod v93400_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn enum_type_to_favnir_defined() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("enum_type_to_favnir"),
            "sap_metadata.rs should define enum_type_to_favnir"
        );
    }

    #[test]
    fn edmx_enum_type_struct_defined() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("EdmxEnumType"),
            "sap_metadata.rs should define EdmxEnumType"
        );
    }
}
```

### Step 6: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,128 tests, 0 failures を確認する。

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
