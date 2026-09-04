# Plan: v91.5.0 — `BusinessPartnerQuery` 実装

## 実装ステップ

### Step 0: 着手前チェック

- `cargo test` で 4,075 tests, 0 failures を確認
- `fav/src/driver.rs` に `mod v91400_tests` が存在することを確認
- `runes/sap-odata/query.fav` に `public type SalesOrderQuery` が含まれることを確認
- `runes/sap-odata/query.fav` に `use sap_odata.sales_order` が含まれることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: 循環 dep チェック

`business_partner.fav` の import 依存を確認する。

```bash
head -5 runes/sap-odata/business_partner.fav
```

期待: `use sap_odata.types`（または類似）があるが、`query.fav` を import していないこと。
→ `query.fav` が `business_partner.fav` を import しても循環しないことを確認する。

`types.fav` が `query.fav` を import していないことも確認:

```bash
grep "use sap_odata.query" runes/sap-odata/types.fav || echo "OK: no circular dep"
```

### Step 2: `query.fav` に `use sap_odata.business_partner` を追加

ファイル先頭の `use sap_odata.sales_order` の直後に追記する：

```favnir
use sap_odata.sales_order
use sap_odata.business_partner
```

### Step 3: `query.fav` に `BusinessPartnerQuery` 型を追記

`SalesOrderQuery` 定義の後に追加する：

```favnir
-- 取引先クエリ型（OData $filter/$select/$expand/$top/$skip をまとめた型）（v91.5.0）
-- SalesOrderQuery と同構造。T は BusinessPartner に束縛。
public type BusinessPartnerQuery = {
    filter: Option<FilterExpr<BusinessPartner>>,
    select: Option<SelectClause<BusinessPartner>>,
    expand: Option<ExpandClause<BusinessPartner>>,
    top:    Option<Int>,
    skip:   Option<Int>
}
```

### Step 4: `query.fav` に `business_partner_query()` ビルダーを追記

`sales_order_query()` 直後に追加する：

```favnir
-- BusinessPartnerQuery のデフォルトビルダー
-- 使用例: bind q <- business_partner_query()
--         bind q <- { q | filter: Option.some(Eq("Country", "JP")) }
public fn business_partner_query() -> BusinessPartnerQuery {
    BusinessPartnerQuery {
        filter: Option.none(),
        select: Option.none(),
        expand: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}
```

### Step 5: `driver.rs` に `mod v91500_tests` を追加

`mod v91400_tests { ... }` の直後に追加する：

```rust
#[cfg(test)]
mod v91500_tests {
    #[test]
    fn business_partner_query_type_defined() {
        let src = std::fs::read_to_string("runes/sap-odata/query.fav").unwrap();
        assert!(
            src.contains("public type BusinessPartnerQuery"),
            "query.fav should define BusinessPartnerQuery: {:?}",
            &src[..src.len().min(200)]
        );
    }

    #[test]
    fn business_partner_query_builder_defined() {
        let src = std::fs::read_to_string("runes/sap-odata/query.fav").unwrap();
        assert!(
            src.contains("public fn business_partner_query"),
            "query.fav should define business_partner_query builder: {:?}",
            &src[..src.len().min(200)]
        );
    }
}
```

### Step 6: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "passed"
```

期待: `4077 passed; 0 failed`

### Step 7: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

## 依存順序

```
Step 0（チェック）
  → Step 1（循環 dep チェック）
  → Step 2（import 追加）
  → Step 3（型定義）
  → Step 4（ビルダー関数）
  → Step 5（テスト追加）
  → Step 6（cargo test 全 pass）
  → Step 7（CI 事前確認）
```
