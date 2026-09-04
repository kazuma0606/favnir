# Plan: v92.1.0 — `QueryBuilder<T>` 型定義

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4094 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92000_tests` が存在することを確認
- `runes/sap-odata/query.fav` に `ODataQueryBuilder` が含まれることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: `runes/sap-odata/query_builder.fav` を新規作成

```favnir
-- runes/sap-odata/query_builder.fav — QueryBuilder<T> 汎用クエリビルダー型（v92.1.0）
-- T: クエリ対象エンティティ型（ファントム型パラメータ）
-- SelectClause / ExpandClause / FilterExpr は query.fav からインポート

use sap_odata.query

-- 汎用クエリビルダー型
-- 全フィールドは Option; Option.none() は「指定なし（デフォルト）」を意味する
-- Fluent API（with_select / with_filter 等）は v92.2.0 以降で追加予定
public type QueryBuilder<T> = {
    select_clause: Option<SelectClause<T>>,
    expand_clause: Option<ExpandClause<T>>,
    filter_expr:   Option<FilterExpr<T>>,
    top_n:         Option<Int>,
    skip_n:        Option<Int>,
    order_by:      Option<String>
}

-- 全フィールドを none() で初期化した QueryBuilder を生成するコンストラクタ
-- 使用例: bind q <- query<SalesOrder>()
--         bind q <- { q | top_n: Option.some(50) }
public fn query<T>() -> QueryBuilder<T> {
    QueryBuilder {
        select_clause: Option.none(),
        expand_clause: Option.none(),
        filter_expr:   Option.none(),
        top_n:         Option.none(),
        skip_n:        Option.none(),
        order_by:      Option.none()
    }
}
```

### Step 2: `driver.rs` に `mod v92100_tests` を追加

`mod v92000_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92100_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn query_builder_file_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(!content.is_empty(), "query_builder.fav should not be empty");
    }
    #[test]
    fn query_builder_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public type QueryBuilder"),
            "query_builder.fav should define public type QueryBuilder"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4096 passed; 0 failed`

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
  → Step 1（query_builder.fav 新規作成）
  → Step 2（driver.rs: テスト追加）
  → Step 3（cargo test）
  → Step 4（CI 事前確認）
```
