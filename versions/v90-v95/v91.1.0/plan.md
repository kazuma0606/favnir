# Plan: v91.1.0 — `SelectClause<T>` 型定義

## 実装順序

### Step 1: `runes/sap-odata/query.fav` 新規作成

`runes/sap-odata/query.fav` を新規作成し、以下を実装する。

```favnir
-- SAP OData クエリ型定義（v91.1.0〜）
-- $select / $expand / $filter を型で表現するモジュール

use sap_odata.types

-- フィールド選択を表す型（OData $select に対応）
public type SelectClause<T> = {
    fields: List<String>
}

-- フィールドリストから SelectClause を生成するヘルパー
public fn select_fields<T>(fields: List<String>) -> SelectClause<T> {
    SelectClause { fields: fields }
}
```

### Step 2: `fav/src/driver.rs` にテストモジュール追加

`mod v91000_tests { ... }` の直後に以下を追加する。
`use super::*` は不要（`std::fs` / `std::path::Path` のみ使用、v91000_tests と同形式）。

```rust
#[cfg(test)]
mod v91100_tests {
    #[test]
    fn odata_query_file_exists() {
        assert!(
            std::path::Path::new("../runes/sap-odata/query.fav").exists(),
            "runes/sap-odata/query.fav should exist"
        );
    }
    #[test]
    fn select_clause_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("SelectClause"),
            "query.fav should define SelectClause type"
        );
    }
}
```

### Step 3: `cargo test` 全 pass 確認

```
cargo test 2>&1 | grep "test result"
# → test result: ok. 4067 passed; 0 failed
```

## 依存関係

```
Step 1 (query.fav 作成)
    └→ Step 2 (driver.rs テスト追加)
        └→ Step 3 (cargo test 確認)
```

## 注意事項

- `query.fav` は他の sap-odata ファイルを `use sap_odata.types` で参照するが、
  `SelectClause<T>` は `SapConfig` 等に依存しないため、types.fav の import は任意
- Favnir のジェネリクス型定義は `type Foo<T> = { ... }` の形式（実装済み）
- `bind` を使う（`let` は Rust のみ）
