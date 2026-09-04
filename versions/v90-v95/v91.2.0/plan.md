# Plan: v91.2.0 — `ExpandClause<T>` 型定義

## 実装順序

### Step 1: `runes/sap-odata/query.fav` に追記

`SelectClause<T>` の定義の後に以下を追加する。

```favnir
-- ナビゲーション展開を表す型（OData $expand に対応）
-- navigation_properties: 展開するナビゲーションプロパティ名のリスト
-- （例: ["to_Item", "to_Partner"]）
public type ExpandClause<T> = {
    navigation_properties: List<String>
}

-- ナビゲーションプロパティリストから ExpandClause を生成するヘルパー
public fn expand_nav<T>(navigation_properties: List<String>) -> ExpandClause<T> {
    ExpandClause { navigation_properties: navigation_properties }
}
```

### Step 2: `fav/src/driver.rs` にテストモジュール追加

`mod v91100_tests { ... }` の直後に以下を追加する。
`use super::*` は不要（`std::fs` のみ使用、v91100_tests と同形式）。

```rust
#[cfg(test)]
mod v91200_tests {
    #[test]
    fn expand_clause_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("ExpandClause"),
            "query.fav should define ExpandClause type"
        );
    }
    #[test]
    fn expand_nav_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("expand_nav"),
            "query.fav should define expand_nav function"
        );
    }
}
```

### Step 3: `cargo test` 全 pass 確認

```
cargo test 2>&1 | grep "passed"
# → test result: ok. 4069 passed; 0 failed
```

## 依存関係

```
Step 1 (query.fav 追記)
    └→ Step 2 (driver.rs テスト追加)
        └→ Step 3 (cargo test 確認)
```

## 注意事項

- `query.fav` への追記は `SelectClause<T>` の定義の後（ファイル末尾）に行う
- `use` 文の追加は不要（`ExpandClause<T>` も他モジュールに依存しない）
- `bind` を使う（`let` は Rust のみ）
