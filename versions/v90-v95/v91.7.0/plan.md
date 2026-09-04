# Plan: v91.7.0 — `JournalEntryQuery` 実装

## 実装ステップ

### Step 0: 着手前チェック

- `cargo test` で 4,081 tests, 0 failures を確認
- `fav/src/driver.rs` に `mod v91600_tests` が存在することを確認
- `runes/sap-odata/query.fav` に `public type PurchaseOrderQuery` が含まれることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: 循環 dep チェック

```bash
head -5 runes/sap-odata/journal_entry.fav
grep "use sap_odata.query" runes/sap-odata/journal_entry.fav || echo "OK"
grep "use sap_odata.query" runes/sap-odata/types.fav || echo "OK"
```

期待: `journal_entry.fav` は `query.fav` を import していない。

### Step 2: `query.fav` に `use sap_odata.journal_entry` を追加

`use sap_odata.purchase_order` の直後に追記：

```favnir
use sap_odata.journal_entry
```

### Step 3: `query.fav` に `JournalEntryQuery` 型を追記

`purchase_order_query()` 定義の後に追加。`fiscal_year` フィールドを含む点が他クエリ型と異なる：

```favnir
-- 会計伝票クエリオプション（$filter / $select / $top / $skip + 会計年度専用フィールド）（v91.7.0）
-- fiscal_year: SAP 会計年度（例: "2024"）。Option.none() の場合は全年度対象。
-- SapClient.journal_entries_query への統合は v91.8.0 で実装（循環 dep のため延期）
public type JournalEntryQuery = {
    filter:      Option<FilterExpr<JournalEntry>>,
    select:      Option<SelectClause<JournalEntry>>,
    fiscal_year: Option<String>,
    top:         Option<Int>,
    skip:        Option<Int>
}
```

### Step 4: `query.fav` に `journal_entry_query()` ビルダーを追記

```favnir
-- 会計伝票クエリオプションを全フィールド none() で初期化するビルダー
-- 使用例: bind q <- journal_entry_query()
--         bind q <- { q | fiscal_year: Option.some("2024"), filter: Option.some(Gt("AmountInTransactionCurrency", "1000")) }
public fn journal_entry_query() -> JournalEntryQuery {
    JournalEntryQuery {
        filter:      Option.none(),
        select:      Option.none(),
        fiscal_year: Option.none(),
        top:         Option.none(),
        skip:        Option.none()
    }
}
```

### Step 5: `driver.rs` に `mod v91700_tests` を追加

`mod v91600_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v91700_tests {
    #[test]
    fn journal_entry_query_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public type JournalEntryQuery"),
            "query.fav should define public type JournalEntryQuery"
        );
    }
    #[test]
    fn journal_entry_query_builder_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public fn journal_entry_query"),
            "query.fav should define public fn journal_entry_query"
        );
    }
}
```

### Step 6: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4083 passed; 0 failed`

### Step 7: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）→ Step 1（循環 dep 確認）
  → Step 2（import 追加）
  → Step 3（型定義）
  → Step 4（ビルダー関数）
  → Step 5（テスト追加）
  → Step 6（cargo test）
  → Step 7（CI 事前確認）
```
