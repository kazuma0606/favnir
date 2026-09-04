# Plan: v95.5.0 — Deep Insert

## 実装順序

### Step 0: ベースライン確認

`cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,172 であることを確認する。

### Step 1: `sales_order.fav` に Deep Insert 型と関数を追加

`runes/sap-odata/sales_order.fav` の末尾（`SalesReport` 型定義の後）に追加する。

追加内容:
- `NewSalesOrderWithItems` 型定義（customer_id: String, currency: String, items: List<NewSalesOrderItem>）
- `create_sales_order_deep` 関数スタブ（戻り値: `Result.err("not implemented")`）

注意:
- `NewSalesOrderItem` は同ファイル 60〜64 行目に既定義（`material_id` / `quantity` / `unit`）
- ロードマップの `NewSalesOrderItem { material: "MAT001", ... }` は `material_id` の記述ゆれ
  → ファイル内の既存定義（`material_id`）に合わせる
- 関数シグネチャ: `create_sales_order_deep(cfg: SapConfig, order: NewSalesOrderWithItems) -> Result<SalesOrder, String>`
  （ctx パターンへの移行は後続バージョンで実施）

依存: なし

### Step 2: `driver.rs` にテストを追加

`fav/src/driver.rs` に `#[cfg(test)] mod v95500_tests` を追加する。

テスト 2 件:
1. `deep_insert_type_defined` — `sales_order.fav` に `NewSalesOrderWithItems` が含まれる
2. `create_sales_order_deep_defined` — `sales_order.fav` に `create_sales_order_deep` が含まれる

依存: Step 1

### Step 3: `cargo test` で全 pass 確認

4,174 tests, 0 failures を確認する。

### Step 4: CHANGELOG / current.md / tasks.md 更新

- `CHANGELOG.md` の先頭に `[v95.5.0]` エントリを追加する
- `versions/current.md` の最新安定版を `v95.5.0` に更新する
- `tasks.md` を COMPLETE に更新する
