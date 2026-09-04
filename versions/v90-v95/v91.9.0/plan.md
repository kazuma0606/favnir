# Plan: v91.9.0 — 安定化・コードフリーズ

## 実装ステップ

### Step 0: 着手前チェック

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4088 passed; 0 failed`

- `fav/src/driver.rs` に `mod v91800_tests` が存在することを確認
- `runes/sap-odata/query.fav` に `ODataQueryBuilder` が含まれることを確認
- `runes/sap-odata/query_client.fav` が存在することを確認

### Step 1: `driver.rs` に `mod v91900_tests` を追加

`mod v91800_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v91900_tests {
    #[test]
    fn odata_query_smoke_all_query_types() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public type SalesOrderQuery"),
            "query.fav should define SalesOrderQuery"
        );
        assert!(
            content.contains("public type BusinessPartnerQuery"),
            "query.fav should define BusinessPartnerQuery"
        );
        assert!(
            content.contains("public type MaterialQuery"),
            "query.fav should define MaterialQuery"
        );
        assert!(
            content.contains("public type PurchaseOrderQuery"),
            "query.fav should define PurchaseOrderQuery"
        );
        assert!(
            content.contains("public type JournalEntryQuery"),
            "query.fav should define JournalEntryQuery"
        );
    }
    #[test]
    fn odata_filter_expr_serializable() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public fn filter_to_odata_string"),
            "query.fav should define public fn filter_to_odata_string"
        );
    }
}
```

### Step 2: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4090 passed; 0 failed`

### Step 3: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（driver.rs: テスト追加）
  → Step 2（cargo test）
  → Step 3（CI 事前確認）
```
