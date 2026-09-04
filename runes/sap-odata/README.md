# SAP OData Rune

Favnir の SAP S/4HANA OData 統合 Rune です。
BusinessPartner / SalesOrder / Material / JournalEntry などの主要エンティティに対して、
型安全なクエリ・バルクバッチ操作・メタデータ推論を提供します。

## Features

- **QueryBuilder** — OData `$filter` / `$select` / `$top` / `$skip` を型安全に組み立て
- **Batch** — OData `$batch` プロトコルによるバルク CRUD 操作（`BatchRequest<T>` / `ChangeSet<T>`）
- **Metadata Infer** — `fav infer --from sap --entity <EntityName>` で Favnir 型定義を自動生成
- **Benchmark** — `fav bench --sap` で QueryBuilder / BatchRequest / Metadata Infer の総合ベンチマーク

## Setup

### 1. `fav.toml` の設定

```toml
[sap]
base_url   = "https://your-sap-host.example.com/sap/opu/odata/sap/"
client_id  = "100"
username   = "${SAP_USER}"
password   = "${SAP_PASS}"
```

### 2. 環境変数の設定

```bash
export SAP_USER="your-sap-user"
export SAP_PASS="your-sap-password"
```

### 3. 本番環境（AWS SSM SecureString）

本番環境ではパスワードを環境変数に直接置かず、SSM SecureString を推奨します:

```toml
[sap]
base_url         = "https://your-sap-host.example.com/sap/opu/odata/sap/"
client_id        = "100"
user_ssm_path    = "/favnir/sap/username"
password_ssm_path = "/favnir/sap/password"
```

Lambda 実行ロールに `ssm:GetParameter` 権限を付与してください。

## Usage

### QueryBuilder

```favnir
use sap_odata

fn fetch_jp_partners(ctx: AppCtx) -> Result<List<BusinessPartner>, String> {
    bind q   <- query<BusinessPartner>()
    bind q   <- with_filter(q, Eq("BusinessPartnerCategory", "1"))
    bind q   <- with_filter(q, Eq("Country", "JP"))
    bind q   <- with_top(q, 100)
    ctx.sap.business_partners(q.filter)
}
```

### Batch（$batch）

```favnir
use sap_odata

fn batch_create_partners(ctx: AppCtx, partners: List<BusinessPartner>) -> Result<BatchResponse<BusinessPartner>, String> {
    bind ops <- List.map(partners, fn(p) { BatchCreate(p) })
    bind req <- batch_request_builder("A_BusinessPartner", ops)
    ctx.sap.batch(req)
}
```

### Metadata Infer

```bash
# SAP エンティティから Favnir 型定義を自動生成
fav infer --from sap --entity A_BusinessPartner

# EDMX ファイルから生成
fav infer --from sap-file --file metadata.edmx
```

## Rune ファイル構成

| ファイル | 内容 |
|---|---|
| `types.fav` | 共通型定義（`SapConfig` / `SapClient` interface） |
| `batch.fav` | Batch 型（`BatchOperation<T>` / `BatchRequest<T>` / `ChangeSet<T>`） |
| `query_builder.fav` | QueryBuilder / Page 型 |
| `query.fav` | OData クエリ補助関数 |
| `query_client.fav` | QueryClient 実装（ページング対応） |
| `business_partner.fav` | BusinessPartner エンティティ |
| `sales_order.fav` | SalesOrder エンティティ |
| `sales_report.fav` | SalesReport エンティティ |
| `material.fav` | Material エンティティ |
| `purchase_order.fav` | PurchaseOrder エンティティ |
| `stock.fav` | Stock エンティティ |
| `journal_entry.fav` | JournalEntry エンティティ |
| `client.fav` | SapHttpClient 実装 |
| `mock.fav` | MockSapClient（テスト用） |
| `sap_odata.test.fav` | Rune ユニットテスト |
| `sap_odata.fav` | 全 Rune の re-export エントリポイント |

## Testing

```bash
# SAP Rune テスト（接続不要のユニットテスト）
cd fav
cargo test sap

# SAP Advanced Benchmark Suite
./target/debug/fav bench --sap
```

## Contributing

SAP Rune への新エンティティ追加方法は [CONTRIBUTING.md](../../CONTRIBUTING.md) の
「SAP Rune — 新エンティティの追加手順」セクションを参照してください。

## License

MIT — Copyright (c) Favnir Contributors
