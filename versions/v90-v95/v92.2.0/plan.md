# Plan: v92.2.0 — `.select` / `.expand` / `.filter` チェーン実装

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4097 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92100_tests` が存在することを確認
- `runes/sap-odata/query_builder.fav` に `public type QueryBuilder` が含まれることを確認
- `runes/sap-odata/query.fav` を Read し、`SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>` のフィールド名を確認する
  - `SelectClause<T>`: `fields: List<String>`
  - `ExpandClause<T>`: `navigation_properties: List<String>`
  - `FilterExpr<T>`: variant 型（`Eq` / `And` / `Or` 等）
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: `query_builder.fav` に 3 関数を追加

`query<T>()` 関数の直後に追記する：

```favnir
-- select チェーン: フィールドリストを SelectClause に変換して set する
public fn with_select<T>(builder: QueryBuilder<T>, fields: List<String>) -> QueryBuilder<T> {
    { builder | select_clause: Option.some(SelectClause { fields: fields }) }
}

-- expand チェーン: ナビゲーションプロパティを ExpandClause に変換して set する
public fn with_expand<T>(builder: QueryBuilder<T>, nav_props: List<String>) -> QueryBuilder<T> {
    { builder | expand_clause: Option.some(ExpandClause { navigation_properties: nav_props }) }
}

-- filter チェーン: FilterExpr をそのまま set する
public fn with_filter<T>(builder: QueryBuilder<T>, expr: FilterExpr<T>) -> QueryBuilder<T> {
    { builder | filter_expr: Option.some(expr) }
}
```

### Step 2: `driver.rs` に `mod v92200_tests` を追加

`mod v92100_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92200_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn with_select_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn with_select"),
            "query_builder.fav should define public fn with_select"
        );
    }
    #[test]
    fn with_expand_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn with_expand"),
            "query_builder.fav should define public fn with_expand"
        );
    }
    #[test]
    fn with_filter_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn with_filter"),
            "query_builder.fav should define public fn with_filter"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4099 passed; 0 failed`

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（query_builder.fav に 3 関数追加）
  → Step 2（driver.rs: テスト追加）
  → Step 3（cargo test）
  → Step 4（CI 事前確認）
```
