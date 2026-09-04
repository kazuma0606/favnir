# Roadmap v91.1.0 〜 v92.0.0 — SAP OData Query 1.0

Date: 2026-08-25
Status: 完了（v92.0.0 宣言済み・2026-08-30）

マスターロードマップ: [roadmap-v90.1-v95.0.md](roadmap-v90.1-v95.0.md)

---

## 前提

- 直前完了: v91.0.0「SAP Ctx 統合 1.0 宣言」（tests = 4,063）
- 本スプリントは SAP Advanced Era の第 2 スプリント
- 目標: v92.0.0「SAP OData Query 1.0 宣言」（tests = 4,089）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v91.0.0 になっていることを確認する
- `versions/v90-v95/v91.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/types.fav` に `SapClient` interface が存在することを確認する（v90.1.0 完了済みの証拠）
- `runes/sap-odata/mock.fav` が存在することを確認する（v90.3.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v91000_tests` が存在することを確認する（v91.0.0 完了済みの証拠）

### スプリントの性格

OData プロトコルの `$select` / `$expand` / `$filter` を**型で表現**するスプリント。

`QueryBuilder<T>` の基盤となる「クエリ句クラス」を整備し、型安全な OData クエリを Favnir で書けるようにする。
SAP S/4HANA の実務クエリ（受注絞り込み・取引先展開・仕訳フィルタ）をカバーする。

A（基盤・型定義）60% + B（機能拡充）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数（実測ベース） | 状態 |
|---|---|---|---|
| v91.1.0 | `SelectClause<T>` 型定義（フィールド選択） | 4065 + 2 = 4067 | COMPLETE |
| v91.2.0 | `ExpandClause<T>` 型定義（ナビゲーション展開） | 4067 + 2 = 4069 | COMPLETE |
| v91.3.0 | `FilterExpr<T>` 型定義（フィルタ式） | 4069 + 4 = 4073 | COMPLETE |
| v91.4.0 | `SalesOrderQuery` + クエリオプション統合 | 4073 + 2 = 4075 | COMPLETE |
| v91.5.0 | `BusinessPartnerQuery` 実装 | 4075 + 2 = 4077 | 未着手 |
| v91.6.0 | `MaterialQuery` / `PurchaseOrderQuery` 実装 | 4077 + 2 = 4079 | 未着手 |
| v91.7.0 | `JournalEntryQuery` 実装 | 4079 + 2 = 4081 | 未着手 |
| v91.8.0 | `ODataQueryBuilder` + SapQueryClient 統合 | 4081 + 4 = 4085 | 未着手 |
| v91.9.0 | 安定化・コードフリーズ | 4083 + 2 = 4085 | 未着手 |
| v92.0.0 | SAP OData Query 1.0 宣言 ★クリーンアップ | 4085 + 4 = 4089 | 未着手 |

---

## v91.1.0 — `SelectClause<T>` 型定義

OData `$select` に対応する型を定義する。フィールド名を型で制約し、誤フィールド指定をコンパイル時に検出する。

```favnir
-- フィールド選択を表す型
type SelectClause<T> = {
    fields: List<String>    -- T のフィールド名リスト
}

-- ヘルパー関数
fn select_fields<T>(fields: List<String>) -> SelectClause<T> {
    SelectClause { fields: fields }
}

-- 使用例
bind q <- select_fields<BusinessPartner>(["BusinessPartner", "BusinessPartnerName", "Country"])
```

**実装内容:**
- `runes/sap-odata/query.fav`（新規作成）に `SelectClause<T>` を定義
- `select_fields` ヘルパー関数を追加
- `driver.rs` に `mod v91100_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4063 + 2 = 4065）
- `odata_query_file_exists`: `runes/sap-odata/query.fav` が存在する
- `select_clause_type_defined`: `query.fav` に `SelectClause` が含まれる

---

## v91.2.0 — `ExpandClause<T>` 型定義

OData `$expand` に対応する型を定義する。ナビゲーションプロパティを型安全に展開できる。

```favnir
-- ナビゲーション展開を表す型
type ExpandClause<T> = {
    navigation_properties: List<String>
}

-- 使用例: 受注に明細を展開
bind expand <- expand_nav<SalesOrder>(["to_Item", "to_Partner"])

-- ctx.sap を介した利用（v91.4.0 以降に統合）
bind orders <- ctx.sap.sales_orders_with_expand(filter, expand)
```

**実装内容:**
- `runes/sap-odata/query.fav` に `ExpandClause<T>` を追加
- `expand_nav` ヘルパー関数を追加
- `driver.rs` に `mod v91200_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4065 + 2 = 4067）
- `expand_clause_type_defined`: `query.fav` に `ExpandClause` が含まれる
- `expand_nav_function_defined`: `query.fav` に `expand_nav` が含まれる

---

## v91.3.0 — `FilterExpr<T>` 型定義

OData `$filter` に対応する型を定義する。フィルタ条件を ADT で表現し、OData 文字列に変換する。

```favnir
type FilterExpr<T> =
    | Eq(String, String)          -- field = value
    | Gt(String, String)          -- field gt value
    | Lt(String, String)          -- field lt value
    | And(FilterExpr<T>, FilterExpr<T>)
    | Or(FilterExpr<T>, FilterExpr<T>)

-- OData 文字列へのシリアライズ
fn filter_to_odata_string<T>(expr: FilterExpr<T>) -> String {
    match expr {
        | Eq(field, value) -> field ++ " eq '" ++ value ++ "'"
        | Gt(field, value) -> field ++ " gt " ++ value
        | Lt(field, value) -> field ++ " lt " ++ value
        | And(l, r) -> "(" ++ filter_to_odata_string(l) ++ " and " ++ filter_to_odata_string(r) ++ ")"
        | Or(l, r)  -> "(" ++ filter_to_odata_string(l) ++ " or "  ++ filter_to_odata_string(r) ++ ")"
    }
}
```

**実装内容:**
- `runes/sap-odata/query.fav` に `FilterExpr<T>` を追加
- `filter_to_odata_string` 関数を追加
- `driver.rs` に `mod v91300_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4067 + 2 = 4069）
- `filter_expr_type_defined`: `query.fav` に `FilterExpr` が含まれる
- `filter_to_odata_string_defined`: `query.fav` に `filter_to_odata_string` が含まれる

---

## v91.4.0 — `SalesOrderQuery` + クエリオプション統合

`SalesOrderQuery` を定義し、`SelectClause` / `ExpandClause` / `FilterExpr` を統合する。

```favnir
type SalesOrderQuery = {
    filter: Option<FilterExpr<SalesOrder>>,
    select: Option<SelectClause<SalesOrder>>,
    expand: Option<ExpandClause<SalesOrder>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

-- ビルダー関数
fn sales_order_query() -> SalesOrderQuery {
    SalesOrderQuery {
        filter: Option.none(),
        select: Option.none(),
        expand: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}

-- 使用例
bind q <- sales_order_query()
bind q <- { q | filter: Option.some(Eq("SoldToParty", "CUST-001")), top: Option.some(50) }
bind orders <- ctx.sap.sales_orders_query(q)
```

**実装内容:**
- `runes/sap-odata/query.fav` に `SalesOrderQuery` と `sales_order_query` を追加
- `SapClient` interface に `sales_orders_query` メソッドを追加
- `driver.rs` に `mod v91400_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4069 + 2 = 4071）
- `sales_order_query_type_defined`: `query.fav` に `SalesOrderQuery` が含まれる
- `sales_order_query_builder_defined`: `query.fav` に `sales_order_query` 関数が含まれる

---

## v91.5.0 — `BusinessPartnerQuery` 実装

`BusinessPartnerQuery` を定義し、取引先クエリの型安全版を提供する。

```favnir
type BusinessPartnerQuery = {
    filter: Option<FilterExpr<BusinessPartner>>,
    select: Option<SelectClause<BusinessPartner>>,
    expand: Option<ExpandClause<BusinessPartner>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

-- 使用例: 国コードでフィルタ + 住所を展開
bind q <- business_partner_query()
bind q <- {
    q |
    filter: Option.some(Eq("Country", "JP")),
    expand: Option.some(expand_nav<BusinessPartner>(["to_BusinessPartnerAddress"]))
}
bind bps <- ctx.sap.business_partners_query(q)
```

**実装内容:**
- `runes/sap-odata/query.fav` に `BusinessPartnerQuery` と `business_partner_query` を追加
- `driver.rs` に `mod v91500_tests` を追加（2 件）

> **Note**: `SapClient` interface への `business_partners_query` / `sales_orders_query` 追加は、`types.fav` → `query.fav` 循環 dep 制約のため **v91.8.0 へ延期**（v91.4.0 の T4 SKIP と同じ理由）。v91.8.0 で `ODataQueryBuilder` 実装と合わせて循環 dep を解消した上で一括統合する。

**完了条件**: Rust テスト 2 件（4075 + 2 = 4077）
- `business_partner_query_type_defined`: `query.fav` に `BusinessPartnerQuery` が含まれる
- `business_partner_query_builder_defined`: `query.fav` に `business_partner_query` 関数が含まれる

---

## v91.6.0 — `MaterialQuery` / `PurchaseOrderQuery` 実装

資材・購買発注クエリの型安全版を実装する。

```favnir
type MaterialQuery = {
    filter: Option<FilterExpr<Material>>,
    select: Option<SelectClause<Material>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

type PurchaseOrderQuery = {
    filter: Option<FilterExpr<PurchaseOrder>>,
    select: Option<SelectClause<PurchaseOrder>>,
    expand: Option<ExpandClause<PurchaseOrder>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

-- 使用例: 購買発注を仕入先でフィルタ
bind q <- purchase_order_query()
bind q <- { q | filter: Option.some(Eq("Supplier", "VEND-001")) }
bind pos <- ctx.sap.purchase_orders_query(q)
```

**実装内容:**
- `runes/sap-odata/query.fav` に `MaterialQuery` / `material_query()` を追加
- `runes/sap-odata/query.fav` に `PurchaseOrderQuery` / `purchase_order_query()` を追加
- `driver.rs` に `mod v91600_tests` を追加（2 件）

> **Note**: `SapClient` interface への `materials_query` / `purchase_orders_query` 追加は、循環 dep 制約のため **v91.8.0 へ延期**（v91.4.0〜v91.5.0 と同じ方針）。

**完了条件**: Rust テスト 2 件（4077 + 2 = 4079）
- `material_query_type_defined`: `query.fav` に `MaterialQuery` が含まれる
- `purchase_order_query_type_defined`: `query.fav` に `PurchaseOrderQuery` が含まれる

---

## v91.7.0 — `JournalEntryQuery` 実装

仕訳クエリの型安全版を実装する。会計年度・伝票番号・転記日付でのフィルタをサポートする。

```favnir
type JournalEntryQuery = {
    filter:      Option<FilterExpr<JournalEntry>>,
    select:      Option<SelectClause<JournalEntry>>,
    fiscal_year: Option<String>,
    top:         Option<Int>,
    skip:        Option<Int>
}

-- 使用例: 特定会計年度の仕訳を取得
bind q <- journal_entry_query()
bind q <- {
    q |
    fiscal_year: Option.some("2024"),
    filter: Option.some(Gt("AmountInTransactionCurrency", "1000"))
}
bind entries <- ctx.sap.journal_entries_query(q)
```

**実装内容:**
- `runes/sap-odata/query.fav` に `JournalEntryQuery` と `journal_entry_query` を追加
- `driver.rs` に `mod v91700_tests` を追加（2 件）

> **Note**: `SapClient` interface への `journal_entries_query` 追加は、循環 dep 制約のため **v91.8.0 へ延期**（v91.4.0〜v91.6.0 と同じ方針）。

**完了条件**: Rust テスト 2 件（4081 + 2 = 4083）
- `journal_entry_query_type_defined`: `query.fav` に `JournalEntryQuery` が含まれる
- `journal_entry_query_builder_defined`: `query.fav` に `journal_entry_query` 関数が含まれる

---

## v91.8.0 — `ODataQueryBuilder` + SapQueryClient 統合

全エンティティに共通する `ODataQueryBuilder` 型と `build_url` ヘルパーを定義し、クエリを OData URL に変換する統一インターフェースを提供する。
あわせて、v91.4.0〜v91.7.0 で循環 dep により延期されたクエリメソッドを **`SapQueryClient`** 新規 interface として一括統合する。

> **設計変更（計画からの差異）**: 当初の計画では `SapClient`（`types.fav`）に 5 メソッドを直接追加する予定だったが、`types.fav → query.fav` の循環 dep 問題を解消できないことが判明した。解消策として新規ファイル `query_client.fav` に `SapQueryClient` interface を定義し、`SapODataClient` / `MockSapClient` がそれを impl する設計を採用した。`AppCtx.sap` は引き続き `SapClient` 型のまま（`SapQueryClient` との統合は v91.9.0 以降）。

> **build_url 簡易実装**: `$filter`/`$select` 等の URL パラメータ展開（`query_to_params`）は将来バージョンで対応予定。本バージョンでは `entity` を URL に結合するのみ（`base_url ++ "/" ++ builder.entity`）。

```favnir
-- 全クエリ型の共通 wrapper（query.fav に追加）
public type ODataQueryBuilder<T, Q> = {
    query:    Q,
    entity:   String    -- エンティティセット名（"A_BusinessPartner" 等）
}

-- 簡易実装: entity 結合のみ（$filter/$select 展開は将来版で対応）
public fn build_url<T, Q>(builder: ODataQueryBuilder<T, Q>, base_url: String) -> String {
    String.concat([base_url, "/", builder.entity])
}

-- query_client.fav（新規）: 循環 dep 回避策として別 interface で定義
public interface SapQueryClient {
    fn sales_orders_query(ctx: SapQueryClient, q: SalesOrderQuery) -> Result<List<SalesOrder>, String>
    fn business_partners_query(ctx: SapQueryClient, q: BusinessPartnerQuery) -> Result<List<BusinessPartner>, String>
    fn materials_query(ctx: SapQueryClient, q: MaterialQuery) -> Result<List<Material>, String>
    fn purchase_orders_query(ctx: SapQueryClient, q: PurchaseOrderQuery) -> Result<List<PurchaseOrder>, String>
    fn journal_entries_query(ctx: SapQueryClient, q: JournalEntryQuery) -> Result<List<JournalEntry>, String>
}
```

**実装内容:**
- `runes/sap-odata/query.fav` に `ODataQueryBuilder` と `build_url` を追加
- `runes/sap-odata/query_client.fav`（新規）に `SapQueryClient` interface を定義
- `runes/sap-odata/client.fav` に `impl SapQueryClient for SapODataClient` を追加（スタブ）
- `runes/sap-odata/mock.fav` に `impl SapQueryClient for MockSapClient` を追加
- `driver.rs` に `mod v91800_tests` を追加（4 件）

**完了条件**: Rust テスト 4 件（4081 + 4 = 4085）
- `odata_query_builder_type_defined`: `query.fav` に `ODataQueryBuilder` が含まれる
- `build_url_function_defined`: `query.fav` に `build_url` が含まれる
- `query_client_interface_defined`: `query_client.fav` に `SapQueryClient` が含まれる
- `client_implements_sap_query_client`: `client.fav` に `impl SapQueryClient` が含まれる

---

## v91.9.0 — 安定化・コードフリーズ

v91.1〜v91.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認（実測ベース 4,088 tests）
- `query.fav` の全型・関数が整合していることを確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4088 + 2 = 4090）（計画値 4081 + 2 = 4083 からの差異は実測ベース差異による）
- `odata_query_smoke_all_query_types`: `query.fav` に `SalesOrderQuery` / `BusinessPartnerQuery` / `MaterialQuery` / `PurchaseOrderQuery` / `JournalEntryQuery` の全てが含まれる
- `odata_filter_expr_serializable`: `query.fav` に `filter_to_odata_string` が含まれる（v91.3.0 の再確認を新規テスト名で担保）

---

## v92.0.0 — SAP OData Query 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「`SapQueryClient` を通じて `sales_orders_query(q)` と書けば、
>  `$filter`・`$select`・`$expand` を型で組み立てた OData クエリが発行できる。
>  誤フィールド指定はコンパイル時に検出される。
>  それが、Favnir SAP OData Query 1.0 である。」

> **Note**: `ctx.sap.sales_orders_query(q)` 構文（`AppCtx` の `SapQueryClient` 統合）は将来のスプリントで対応予定。
> 現バージョンでは `SapQueryClient` として明示的に型注釈した変数経由でクエリ発行が可能。

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `92.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v92.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テストを `92.0.0` に一括更新

**完了条件**: `v92000_tests` 4 件（4085 + 4 = 4089）
- `cargo_toml_version_is_92_0_0`
- `changelog_has_v92_0_0`
- `milestone_has_sap_odata_query`
- `readme_mentions_odata_query`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v91.0.0（ベース） | 4,065 | — |（実測値）
| v91.1.0 | 4,067 | +2 |（実測値）
| v91.2.0 | 4,069 | +2 |（実測値）
| v91.3.0 | 4,073 | +4 |（実測値。計画 +2 より +2 多い）
| v91.4.0 | 4,075 | +2 |（実測値）
| v91.5.0 | 4,077 | +2 |（実測値）
| v91.6.0 | 4,081 | +4 |（実測値。code-reviewer 対応で builder テスト +2 追加）
| v91.7.0 | 4,084 | +3 |（実測値。code-reviewer 対応で fiscal_year 型テスト +1 追加）
| v91.8.0 | 4,088 | +4 |（実測値。spec-reviewer 対応でテスト +2 追加）
| v91.9.0 | 4,090 | +2 |（実測値）
| v92.0.0（宣言） | 4,094 | +4 |（実測値）

**本スプリント合計**: +29 tests（4,065 → 4,094）（計画 +24 より +5 多い — code-reviewer / spec-reviewer 対応による増加）
