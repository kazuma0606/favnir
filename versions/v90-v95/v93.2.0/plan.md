# Plan: v93.2.0 — `EntityType` → Favnir `type` 変換

Status: TODO

---

## 実装ステップ

### Step 1: 着手前ベースライン確認

`cargo test` を実行し、4,122 tests, 0 failures であることを確認する。

### Step 2: `sap_metadata.rs` に `edm_type_to_favnir` を追加

```rust
/// EDM 型名を Favnir 型名に変換する
pub fn edm_type_to_favnir(edm_type: &str) -> &'static str {
    match edm_type {
        "Edm.String" | "Edm.DateTimeOffset" | "Edm.Guid" => "String",
        "Edm.Int32" | "Edm.Int64"                        => "Int",
        "Edm.Decimal"                                    => "Float",
        "Edm.Boolean"                                    => "Bool",
        _                                                => "String",
    }
}
```

### Step 3: `sap_metadata.rs` に `entity_type_to_favnir` を追加

```rust
/// EdmxEntityType を Favnir type 定義文字列に変換する
/// エンティティ名: 先頭の 2 文字 + "_" プレフィックス（A_/I_/C_ 等）と末尾の "Type" を除去
pub fn entity_type_to_favnir(et: &EdmxEntityType) -> String {
    // 先頭の "X_" パターン（1文字+アンダースコア）を除去
    let name = if et.name.len() > 2 && et.name.as_bytes()[1] == b'_' {
        &et.name[2..]
    } else {
        &et.name
    };
    // 末尾の "Type" を除去
    let name = name.strip_suffix("Type").unwrap_or(name);

    let mut out = format!("type {} = {{\n", name);
    for (i, prop) in et.properties.iter().enumerate() {
        let fav_type = edm_type_to_favnir(&prop.edm_type);
        if i + 1 < et.properties.len() {
            out.push_str(&format!("    {}: {},\n", prop.name, fav_type));
        } else {
            out.push_str(&format!("    {}: {}\n", prop.name, fav_type));
        }
    }
    out.push('}');
    out
}
```

### Step 4: `cargo build` でコンパイル確認

コンパイルエラーがないことを確認する。

### Step 5: `mod v93200_tests` を `driver.rs` に追加

ファイル末尾の `mod v93100_tests { ... }` の直後に追加する:

```rust
#[cfg(test)]
mod v93200_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn entity_type_to_favnir_defined() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("entity_type_to_favnir"),
            "sap_metadata.rs should define entity_type_to_favnir"
        );
    }

    #[test]
    fn edm_type_to_favnir_defined() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("edm_type_to_favnir"),
            "sap_metadata.rs should define edm_type_to_favnir"
        );
    }
}
```

### Step 6: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,124 tests, 0 failures を確認する。

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
