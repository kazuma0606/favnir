# Plan: v89.6.0 — `site/content/docs/runes/sap-odata.mdx` ドキュメント

## 実装ステップ

### Step 1: `site/content/docs/runes/sap-odata.mdx` を作成

`snowflake.mdx` と同形式で以下の内容を作成する:

```mdx
---
title: "SAP OData Rune"
order: 55
category: "Rune"
description: "SAP OData API を型安全に操作する sap-odata Rune"
---

# SAP OData Rune

```favnir
import rune "sap-odata"
```

SAP OData API（S/4HANA Cloud / Business One / ECC）を型安全に操作できる Rune。
BusinessPartner / SalesOrder / Material / JournalEntry の 4 エンティティと
購買（PurchaseOrder）をカバーする。

---

## fav.toml 設定

```toml
[sap]
base_url = "${SAP_BASE_URL}"
username = "${SAP_USERNAME}"
client   = "${SAP_CLIENT}"
```

---

## 環境変数

| 変数名 | 説明 | 必須 |
|---|---|---|
| `SAP_BASE_URL` | SAP OData エンドポイント（例: `https://my.sap.example.com`） | ✓ |
| `SAP_USERNAME` | 認証ユーザー名 | ✓ |
| `SAP_PASSWORD` | 認証パスワード | ✓ |
| `SAP_CLIENT` | SAP クライアント番号（例: `100`） | |

---

## BusinessPartner

```favnir
bind cfg      <- sap_odata.sap_config_from_env()
bind partners <- sap_odata.business_partners(cfg, BusinessPartnerFilter {
    country:       Option.some("JP"),
    category:      Option.none(),
    changed_after: Option.none(),
    top:           Option.some(100)
})
```

---

## SalesOrder

```favnir
bind cfg    <- sap_odata.sap_config_from_env()
bind orders <- sap_odata.sales_orders(cfg, SalesOrderFilter {
    status:         Option.some(SalesOrderStatus.Open),
    customer_id:    Option.none(),
    created_after:  Option.none(),
    created_before: Option.none(),
    sales_org:      Option.none(),
    top:            Option.some(50)
})
```

---

## Material

```favnir
bind cfg       <- sap_odata.sap_config_from_env()
bind materials <- sap_odata.materials(cfg, MaterialFilter {
    material_type: Option.some(MaterialType.FinishedProduct),
    plant:         Option.none(),
    top:           Option.none()
})
```

---

## JournalEntry

```favnir
bind cfg      <- sap_odata.sap_config_from_env()
bind journals <- sap_odata.journal_entries(cfg, JournalFilter {
    fiscal_year:       Option.some(2026),
    posting_date_from: Option.none(),
    company_code:      Option.none(),
    reference:         Option.none(),
    top:               Option.none()
})
```

---

## 業務シナリオ

| # | 関数名 | 概要 |
|---|---|---|
| 1 | `sync_business_partners` | 得意先・仕入先マスタを S3 に同期 |
| 2 | `daily_sales_report` | 日次売上レポートを集計して S3 に保存 |
| 3 | `check_stock_vs_orders` | 在庫不足アラートを検出 |
| 4 | `outstanding_payables` | 未照合の支払い伝票を特定 |

完全な実装は `infra/e2e-demo/sap-odata/pipeline.fav` を参照。

---

## Rune Registry

Rune Registry から sap-odata Rune を取得して利用できる:

```favnir
import rune "sap-odata"
```

Registry URL: `https://registry.favnir.dev/runes/sap-odata`

---

## Docker Compose モックサーバー

ローカル開発では SAP モックサーバーを利用できる:

```bash
# SAP モックサーバーへの接続情報（ローカル開発用）
export SAP_BASE_URL=http://localhost:8080
export SAP_USERNAME=admin
export SAP_PASSWORD=admin
export SAP_CLIENT=100
# LocalStack（S3 エミュレーション）
export AWS_ENDPOINT_URL=http://localhost:4566

scripts/run-sap-demo.sh
```

詳細は `infra/e2e-demo/sap-odata/README.md` を参照。
```

### Step 2: `mod v89600_tests` を `driver.rs` に追加

`mod v89500_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89600_tests {
    #[test]
    fn docs_sap_odata_mdx_exists() {
        assert!(
            std::path::Path::new("../site/content/docs/runes/sap-odata.mdx").exists(),
            "site/content/docs/runes/sap-odata.mdx should exist"
        );
    }

    #[test]
    fn docs_sap_odata_contains_business_partner_section() {
        let content = std::fs::read_to_string(
            "../site/content/docs/runes/sap-odata.mdx",
        )
        .expect("site/content/docs/runes/sap-odata.mdx should exist");
        assert!(
            content.contains("BusinessPartner"),
            "sap-odata.mdx should contain BusinessPartner section"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

4,029 + 2 = 4,031 tests, 0 failures を確認する。

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
