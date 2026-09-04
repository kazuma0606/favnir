# Plan: v93.1.0 — EDMX XML パーサー基盤（`parse_edmx`）

Status: TODO

---

## 実装ステップ

### Step 1: 着手前ベースライン確認

`cargo test` を実行し、4,120 tests, 0 failures であることを確認する。

### Step 2: `fav/src/sap_metadata.rs` を新規作成

構造体 3 件と `parse_edmx` スタブ関数を実装する。
**注**: すべて `pub` で公開するため Clippy の `dead_code` 警告は発生しない（`pub` アイテムは未使用でも警告対象外）。

```rust
// fav/src/sap_metadata.rs — SAP $metadata EDMX XML パーサー（v93.1.0〜）
// v93.1.0: 構造体定義 + parse_edmx スタブ
// v93.2.0 以降: 実際の XML 解析を段階的に実装

/// EDMX Property（EntityType の各フィールド）
pub struct EdmxProperty {
    pub name:     String,
    pub edm_type: String, // "Edm.String" / "Edm.Int32" / "Edm.Boolean" 等
}

/// EDMX EntityType（SAP エンティティの型定義）
pub struct EdmxEntityType {
    pub name:       String,
    pub properties: Vec<EdmxProperty>,
}

/// EDMX Schema（Namespace + EntityType リスト）
pub struct EdmxSchema {
    pub namespace:    String,
    pub entity_types: Vec<EdmxEntityType>,
}

/// EDMX XML を解析して EdmxSchema リストを返す（v93.1.0: スタブ）
/// 完全実装は v93.2.0 以降で段階的に追加する
pub fn parse_edmx(_xml: &str) -> Vec<EdmxSchema> {
    Vec::new()
}
```

### Step 3: `fav/src/main.rs` に `mod sap_metadata;` を追加

既存の `mod` 宣言のアルファベット順に合わせて挿入する。

### Step 4: `cargo build` で確認

コンパイルエラーがないことを確認する。

### Step 5: `mod v93100_tests` を `driver.rs` に追加

ファイル末尾の `mod v93000_tests { ... }` の直後に追加する:

```rust
#[cfg(test)]
mod v93100_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn sap_metadata_file_exists() {
        let path = std::path::Path::new("src/sap_metadata.rs");
        assert!(path.exists(), "src/sap_metadata.rs should exist");
    }

    #[test]
    fn parse_edmx_function_defined() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(src.contains("parse_edmx"), "sap_metadata.rs should define parse_edmx");
    }
}
```

### Step 6: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,122 tests, 0 failures を確認する。

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
