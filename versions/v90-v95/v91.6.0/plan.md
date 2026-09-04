# Plan: v91.6.0 — `MaterialQuery` / `PurchaseOrderQuery` 実装

## 実装ステップ

### Step 0: 着手前チェック

- `cargo test` で 4,077 tests, 0 failures を確認
- `fav/src/driver.rs` に `mod v91500_tests` が存在することを確認
- `runes/sap-odata/query.fav` に `public type BusinessPartnerQuery` が含まれることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 1: 循環 dep チェック

```bash
head -5 runes/sap-odata/material.fav
head -5 runes/sap-odata/purchase_order.fav
grep "use sap_odata.query" runes/sap-odata/types.fav || echo "OK"
```

期待: `material.fav` / `purchase_order.fav` とも `use sap_odata.types` のみ（`query.fav` を import していない）。

### Step 2: `query.fav` に import を追加

`use sap_odata.business_partner` の直後に追記：

```favnir
use sap_odata.material
use sap_odata.purchase_order
```

### Step 3: `query.fav` に `MaterialQuery` 型を追記

`BusinessPartnerQuery` 定義の後に追加（`expand` フィールドなし）：

```favnir
-- 品目クエリオプション（$filter / $select / $top / $skip）（v91.6.0）
-- expand は現スプリントではスコープ外（将来バージョンで追加予定）
public type MaterialQuery = {
    filter: Option<FilterExpr<Material>>,
    select: Option<SelectClause<Material>>,
    top:    Option<Int>,
    skip:   Option<Int>
}
```

### Step 4: `query.fav` に `material_query()` ビルダーを追記

```favnir
-- 品目クエリオプションを全フィールド none() で初期化するビルダー
public fn material_query() -> MaterialQuery {
    MaterialQuery {
        filter: Option.none(),
        select: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}
```

### Step 5: `query.fav` に `PurchaseOrderQuery` 型を追記

`MaterialQuery` 定義の後に追加（`expand` フィールドあり）：

```favnir
-- 購買発注クエリオプション（$filter / $select / $expand / $top / $skip）（v91.6.0）
public type PurchaseOrderQuery = {
    filter: Option<FilterExpr<PurchaseOrder>>,
    select: Option<SelectClause<PurchaseOrder>>,
    expand: Option<ExpandClause<PurchaseOrder>>,
    top:    Option<Int>,
    skip:   Option<Int>
}
```

### Step 6: `query.fav` に `purchase_order_query()` ビルダーを追記

```favnir
-- 購買発注クエリオプションを全フィールド none() で初期化するビルダー
public fn purchase_order_query() -> PurchaseOrderQuery {
    PurchaseOrderQuery {
        filter: Option.none(),
        select: Option.none(),
        expand: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}
```

### Step 7: `driver.rs` に `mod v91600_tests` を追加

`mod v91500_tests { ... }` の直後に追加：

```rust
#[cfg(test)]
mod v91600_tests {
    #[test]
    fn material_query_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public type MaterialQuery"),
            "query.fav should define public type MaterialQuery"
        );
    }
    #[test]
    fn purchase_order_query_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public type PurchaseOrderQuery"),
            "query.fav should define public type PurchaseOrderQuery"
        );
    }
}
```

### Step 8: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```

期待: `4079 passed; 0 failed`

### Step 9: CI 事前確認

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
  → Step 3〜4（MaterialQuery 型・ビルダー）
  → Step 5〜6（PurchaseOrderQuery 型・ビルダー）
  → Step 7（テスト追加）
  → Step 8（cargo test）
  → Step 9（CI 事前確認）
```
