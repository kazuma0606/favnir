# Plan: v92.3.0 — `.top` / `.skip` / `.order_by` チェーン実装

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4100 passed; 0 failed`

- `fav/src/driver.rs` に `mod v92200_tests` が存在することを確認
- `runes/sap-odata/query_builder.fav` に `public fn with_select` / `with_expand` / `with_filter` が含まれることを確認（v92.2.0 完了済みの証拠）
- `fav/tmp/hello.fav` が存在することを確認
- CHANGELOG 更新は v93.0.0 宣言時にまとめて行う（本バージョンでは不要）

### Step 1: `query_builder.fav` に 3 関数を追加

`with_filter` の直後に追記する：

```favnir
-- top チェーン: 取得件数上限を set する（v92.3.0）
public fn with_top<T>(builder: QueryBuilder<T>, n: Int) -> QueryBuilder<T> {
    { builder | top_n: Option.some(n) }
}

-- skip チェーン: 先頭 n 件をスキップする（ページネーション用）（v92.3.0）
public fn with_skip<T>(builder: QueryBuilder<T>, n: Int) -> QueryBuilder<T> {
    { builder | skip_n: Option.some(n) }
}

-- order_by チェーン: ソートフィールドを set する（"FieldName asc" / "FieldName desc" 形式）（v92.3.0）
public fn with_order_by<T>(builder: QueryBuilder<T>, field: String) -> QueryBuilder<T> {
    { builder | order_by: Option.some(field) }
}
```

### Step 2: `driver.rs` に `mod v92300_tests` を追加

`mod v92200_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v92300_tests {
    // use super::* は不要（std::fs のみ使用）
    // パス基点: fav/ ディレクトリ（cargo test の実行カレント）
    #[test]
    fn with_top_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn with_top"),
            "query_builder.fav should define public fn with_top"
        );
    }
    #[test]
    fn with_skip_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn with_skip"),
            "query_builder.fav should define public fn with_skip"
        );
    }
    #[test]
    fn with_order_by_function_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query_builder.fav")
            .expect("runes/sap-odata/query_builder.fav should exist");
        assert!(
            content.contains("public fn with_order_by"),
            "query_builder.fav should define public fn with_order_by"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4103 passed; 0 failed`

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
