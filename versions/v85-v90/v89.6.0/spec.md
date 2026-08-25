# Spec: v89.6.0 — `site/content/docs/runes/sap-odata.mdx` ドキュメント

## Background

v89.5.0 で E2E デモ（4 シナリオ + Lambda + `scripts/run-sap-demo.sh`）が完成した。
本バージョンでは sap-odata Rune の公式ドキュメントをリファレンスサイトに追加する。

既存の `site/content/docs/runes/snowflake.mdx` と同じ MDX 形式で作成する。

## Goals

1. `site/content/docs/runes/sap-odata.mdx` を新規作成する
   - フロントマター（title / order / category / description）
   - 概要・`import rune "sap-odata"` 構文
   - セットアップ（`fav.toml [sap]` セクション + 環境変数一覧）
   - エンティティ別サンプルコード（BusinessPartner / SalesOrder / Material / JournalEntry）
   - 4 業務シナリオの解説（sync_business_partners / daily_sales_report / check_stock_vs_orders / outstanding_payables）
   - Docker Compose モックサーバーでの開発手順
2. `fav/src/driver.rs` に `mod v89600_tests` を追加する（2 件）

## MDX 構成

```
---
title: "SAP OData Rune"
order: 55
category: "Rune"
description: "SAP OData API を型安全に操作する sap-odata Rune"
---

# SAP OData Rune

import rune "sap-odata"

## fav.toml 設定
[sap] セクション

## 環境変数
SAP_BASE_URL / SAP_USERNAME / SAP_PASSWORD / SAP_CLIENT

## BusinessPartner
型定義 + サンプルコード

## SalesOrder
型定義 + サンプルコード

## Material
型定義 + サンプルコード

## JournalEntry
型定義 + サンプルコード

## 業務シナリオ
4 シナリオの解説（pipeline.fav への参照）

## Rune Registry
import rune "sap-odata" の取得方法・Registry URL

## Docker Compose モックサーバー
開発手順（SAP 接続環境変数 + AWS_ENDPOINT_URL の設定例を含む）
```

## Success Criteria（Rust テストで担保）

- `docs_sap_odata_mdx_exists`:
  `site/content/docs/runes/sap-odata.mdx` が存在する
- `docs_sap_odata_contains_business_partner_section`:
  `site/content/docs/runes/sap-odata.mdx` に `"BusinessPartner"` を含む
- `cargo test` で 4,031 tests, 0 failures（4,029 + 2）

## Files to Create / Modify

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/runes/sap-odata.mdx` | 新規作成 |
| `fav/src/driver.rs` | `mod v89600_tests` 追加 |

**前提確認**:
- `site/content/docs/runes/snowflake.mdx` が参照パターンとして存在する
- sap-odata Rune の型定義（BusinessPartner / SalesOrder / Material / JournalEntry）は v86〜v89 で完成済み
- 4 業務シナリオは `infra/e2e-demo/sap-odata/pipeline.fav` に実装済み（v89.3.0）

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
