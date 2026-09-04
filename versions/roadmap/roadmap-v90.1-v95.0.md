# Favnir ロードマップ v90.1〜v95.0 — SAP Advanced Era

Date: 2026-08-25
Status: 完了（v95.0.0 宣言済み・2026-08-30）

---

## 背景と方針

v90.0.0「SAP Integration 1.0」をもって、SAP Integration Era が完成した。
`business_partners()` / `sales_orders()` / `materials()` / `journal_entries()` の 4 業務シナリオが揃い、
Favnir は「SAP データを型安全なパイプラインで扱える言語」になった。

しかし SAP Integration Era には**2 つの未解決課題**が残った。

**課題 1 — Ctx 統合の欠落**
`ctx.s3` / `ctx.io` / `ctx.db` は `AppCtx` に統合されているのに、
SAP アクセスは `sap_odata.business_partners(cfg, filter)` という**Rune 直接呼び出し**のままだった。
`cfg: SapConfig` を手動で取得して渡す設計は、テストでの差し替えが困難で、Ctx パターンの一貫性を壊している。

**課題 2 — OData クエリが弱い**
`$select` / `$expand` / `$filter` / `$orderby` といった OData クエリオプションが型で表現できていない。
フィールド選択・ナビゲーションプロパティ展開・型安全フィルタが欠如しており、N+1 問題も防げない。

v90.1〜v95.0 では、これら 2 課題を解決し、さらに**メタデータ自動生成**（$metadata XML → Favnir 型）を実装して
SAP Advanced 1.0 を宣言する。

```
v91.0 — SAP Ctx 統合 1.0    : 「ctx.sap.* で SAP にアクセスできる」
v92.0 — SAP OData Query 1.0 : 「$select/$expand/$filter が型で書ける」
v93.0 — SAP QueryBuilder 1.0: 「QueryBuilder<T> で N+1 を型で防ぐ」
v94.0 — SAP Metadata Infer  : 「$metadata から Favnir 型を自動生成できる」
v95.0 — SAP Advanced 1.0    : 「$batch / SnapStart / ベンチマーク完成」
```

### 設計方針

- **ctx.sap 統合**: `SapClient` interface を定義し、`AppCtx` に `sap: SapClient` フィールドを追加する
  - 旧: `bind cfg <- sap_odata.sap_config_from_env()` → `bind bps <- sap_odata.business_partners(cfg, filter)`
  - 新: `bind bps <- ctx.sap.business_partners(filter)` — cfg は AppCtx 構築時に注入
- **QueryBuilder<T>**: Favnir の型システムで OData クエリを表現する型安全ビルダー
- **メタデータ自動生成**: SAP の `$metadata` XML を解析し、Favnir 型定義を自動生成する
  - `fav infer --from sap --metadata <url>` コマンドで実現
- **OData $batch**: 複数操作の一括送信でネットワーク往復を削減
- **Lambda SnapStart**: cold start 削減（1,200ms → 300ms 目標）

### 業務シナリオ（前スプリントからの継続）

| シナリオ | エンティティ | v90.1〜v95.0 での拡張 |
|---|---|---|
| 1. マスタデータ同期 | BusinessPartner | ctx.sap.business_partners() + $select/$expand |
| 2. 日次売上レポート | SalesOrder + SalesOrderItem | QueryBuilder + $batch |
| 3. 在庫×受注クロスチェック | Material × SalesOrder | Page<T> 並列ページネーション |
| 4. 購買→支払サイクル照合 | PurchaseOrder × JournalEntry | $metadata 自動生成型で再実装 |

---

## テスト数推移（本スプリント全体）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v90.0.0（ベース） | 4,041 | — |
| v90.1.0〜v90.9.0 | +2 × 9 = +18 | 4,059 |
| v91.0.0（宣言） | +4 | 4,063 |
| v91.1.0〜v91.9.0 | +2 × 9 = +18 | 4,081 |
| v92.0.0（宣言） | +4 | 4,085 |
| v92.1.0〜v92.9.0 | +2 × 9 = +18 | 4,103 |
| v93.0.0（宣言） | +4 | 4,107 |
| v93.1.0〜v93.9.0 | +2 × 9 = +18 | 4,125 |
| v94.0.0（宣言） | +4 | 4,142 |
| v94.1.0〜v94.9.0 | +2 × 9 = +18 | 4,160 |
| v95.0.0（宣言） | +4 | 4,164 |

**本スプリント合計**: +123 tests（4,041 → 4,164）

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 1: SAP Ctx 統合（v90.1〜v91.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: `AppCtx` への `ctx.sap` 統合。設定注入・MockSapClient・pipeline.fav 書き換え。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v90.1.0 | `SapClient` interface 定義（5 関数） | 4041 + 2 = 4043 | 未着手 |
| v90.2.0 | `AppCtx` に `sap: SapClient` フィールドを追加 | 4043 + 2 = 4045 | 未着手 |
| v90.3.0 | `MockSapClient` 実装（テスト用スタブ） | 4045 + 2 = 4047 | 未着手 |
| v90.4.0 | `Ctx.build` に SAP 設定注入を統合（`fav.toml [sap]`） | 4047 + 2 = 4049 | 未着手 |
| v90.5.0 | `runes/sap-odata/sap_odata.fav` を `ctx.sap.*` スタイルに対応 | 4049 + 2 = 4051 | 未着手 |
| v90.6.0 | `infra/e2e-demo/sap-odata/pipeline.fav` を `ctx.sap.*` で書き換え | 4051 + 2 = 4053 | 未着手 |
| v90.7.0 | `Ctx.mock` に `sap: MockSapClient` を追加 | 4053 + 2 = 4055 | 未着手 |
| v90.8.0 | サイトドキュメント更新（ctx.sap パターンガイド） | 4055 + 2 = 4057 | 未着手 |
| v90.9.0 | 安定化・コードフリーズ | 4057 + 2 = 4059 | 未着手 |
| v91.0.0 | SAP Ctx 統合 1.0 宣言 ★クリーンアップ | 4059 + 4 = 4063 | 未着手 |

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 2: OData クエリ深化（v91.1〜v92.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: `$select` / `$expand` / `$filter` を Favnir の型で表現する。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v91.1.0 | `SelectClause<T>` 型定義（`$select` フィールド選択） | 4063 + 2 = 4065 | 未着手 |
| v91.2.0 | `ExpandClause<T>` 型定義（`$expand` ナビゲーションプロパティ） | 4065 + 2 = 4067 | 未着手 |
| v91.3.0 | `FilterExpr<T>` 型定義（`$filter` 型安全フィルタ式） | 4067 + 2 = 4069 | 未着手 |
| v91.4.0 | `SalesOrderQuery` に `select` / `expand` / `filter` を追加 | 4069 + 2 = 4071 | 未着手 |
| v91.5.0 | `BusinessPartnerQuery` に同様追加 | 4071 + 2 = 4073 | 未着手 |
| v91.6.0 | `MaterialQuery` / `PurchaseOrderQuery` に同様追加 | 4073 + 2 = 4075 | 未着手 |
| v91.7.0 | `JournalEntryQuery` に同様追加 | 4075 + 2 = 4077 | 未着手 |
| v91.8.0 | `ODataQueryBuilder` 共通ヘルパー（OData URL 生成） | 4077 + 2 = 4079 | 未着手 |
| v91.9.0 | 安定化・コードフリーズ | 4079 + 2 = 4081 | 未着手 |
| v92.0.0 | SAP OData Query 1.0 宣言 ★クリーンアップ | 4081 + 4 = 4085 | 未着手 |

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 3: QueryBuilder<T> + ページネーション（v92.1〜v93.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: 型安全ビルダー API と並列ページネーション。N+1 問題を型で防ぐ。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v92.1.0 | `QueryBuilder<T>` 型設計（ビルダーパターン） | 4085 + 2 = 4087 | 未着手 |
| v92.2.0 | `.select()` / `.expand()` / `.filter()` チェーン実装 | 4087 + 2 = 4089 | 未着手 |
| v92.3.0 | `.top()` / `.skip()` / `.order_by()` 追加 | 4089 + 2 = 4091 | 未着手 |
| v92.4.0 | `Page<T>` 型 + `fetch_all_pages()` 並列ページネーション | 4091 + 2 = 4093 | 未着手 |
| v92.5.0 | N+1 防止 lint ルール W020（`$expand` なし多段アクセス検出） | 4093 + 2 = 4095 | 未着手 |
| v92.6.0 | `ctx.sap.query<SalesOrder>().select(...).execute()` E2E テスト | 4095 + 2 = 4097 | 未着手 |
| v92.7.0 | ベンチマーク更新（1,000 件並列ページネーション計測） | 4097 + 2 = 4099 | 未着手 |
| v92.8.0 | サイトドキュメント更新（QueryBuilder リファレンス） | 4099 + 2 = 4101 | 未着手 |
| v92.9.0 | 安定化・コードフリーズ | 4101 + 2 = 4103 | 未着手 |
| v93.0.0 | SAP QueryBuilder 1.0 宣言 ★クリーンアップ | 4103 + 4 = 4107 | 未着手 |

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 4: メタデータ自動生成（v93.1〜v94.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: SAP `$metadata` XML から Favnir 型定義を自動生成する。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v93.1.0 | OData `$metadata` XML パーサー（`EntityType` / `Property` 抽出） | 4107 + 2 = 4109 | 未着手 |
| v93.2.0 | `EntityType` → Favnir `type { ... }` 変換エンジン | 4109 + 2 = 4111 | 未着手 |
| v93.3.0 | `NavigationProperty` → `Option<List<T>>` フィールド変換 | 4111 + 2 = 4113 | 未着手 |
| v93.4.0 | `EnumType` → Favnir variant 変換（`type Status = Active \| Inactive`） | 4113 + 2 = 4115 | 未着手 |
| v93.5.0 | `fav infer --from sap --metadata <url>` CLI コマンド実装 | 4115 + 2 = 4117 | 未着手 |
| v93.6.0 | `fav infer --from sap --metadata <file>` ローカル XML ファイル対応 | 4117 + 2 = 4119 | 未着手 |
| v93.7.0 | 生成コードの自動フォーマット（`fav fmt` 互換出力） | 4119 + 2 = 4121 | 未着手 |
| v93.8.0 | サイトドキュメント更新（`fav infer --from sap` リファレンス） | 4121 + 2 = 4123 | 未着手 |
| v93.9.0 | 安定化・コードフリーズ | 4123 + 2 = 4125 | 未着手 |
| v94.0.0 | SAP Metadata Infer 1.0 宣言 ★クリーンアップ | 4125 + 4 = 4129 | 未着手 |

---

## ━━━━━━━━━━━━━━━━━━━━━━━━━━
## Sprint 5: SAP Advanced 1.0 宣言（v94.1〜v95.0）
## ━━━━━━━━━━━━━━━━━━━━━━━━━━

**テーマ**: `$batch` / Lambda SnapStart / 総合ベンチマーク。SAP Advanced 1.0 を宣言する。

### バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v94.1.0 | OData `$batch` リクエスト型定義（`BatchRequest<T>` / `BatchResponse<T>`） | 4142 + 2 = 4144 | 未着手 |
| v94.2.0 | `ChangeSet` 型（バッチ内トランザクション境界）+ `ctx.sap.batch()` | 4144 + 2 = 4146 | 未着手 |
| v94.3.0 | Lambda SnapStart 対応（`infra/` Terraform 更新） | 4146 + 2 = 4148 | 未着手 |
| v94.4.0 | cold start ベースライン更新（SnapStart 効果計測・benchmarks/ 更新） | 4148 + 2 = 4150 | 未着手 |
| v94.5.0 | `fav bench --sap` サブコマンド（SAP 固有ベンチマーク自動計測） | 4150 + 2 = 4152 | 未着手 |
| v94.6.0 | OSS 整備（CONTRIBUTING Advanced SAP + ISSUE_TEMPLATE 更新） | 4152 + 2 = 4154 | 未着手 |
| v94.7.0 | E2E デモ更新（QueryBuilder + $batch + $metadata 生成型で再実装） | 4154 + 2 = 4156 | 未着手 |
| v94.8.0 | サイトドキュメント完成（SAP Advanced ガイド全体） | 4156 + 2 = 4158 | 未着手 |
| v94.9.0 | 安定化・コードフリーズ | 4158 + 2 = 4160 | 未着手 |
| v95.0.0 | SAP Advanced 1.0 宣言 ★クリーンアップ | 4160 + 4 = 4164 | 未着手 |

---

## スプリント総括

| スプリント | バージョン範囲 | テーマ | テスト増 | 累計 |
|---|---|---|---|---|
| Sprint 1 | v90.1〜v91.0 | SAP Ctx 統合（SapClient / AppCtx.sap / MockSapClient） | +22 | 4,063 |
| Sprint 2 | v91.1〜v92.0 | OData クエリ深化（$select / $expand / $filter） | +22 | 4,085 |
| Sprint 3 | v92.1〜v93.0 | QueryBuilder<T> + Page<T> + W020 N+1 lint | +22 | 4,107 |
| Sprint 4 | v93.1〜v94.0 | $metadata 自動生成（fav infer --from sap --metadata） | +22 | 4,142 |
| Sprint 5 | v94.1〜v95.0 | $batch / SnapStart / SAP Advanced 1.0 宣言 | +22 | 4,164 |

**合計**: +123 tests（4,041 → 4,164）

### 参考リンク

- 前フェーズ（完了）: [roadmap-v85.1-v90.0.md](roadmap-v85.1-v90.0.md)
- Sprint 1 詳細: [roadmap-v90.1-v91.0.md](roadmap-v90.1-v91.0.md)
- 進行状況: [../current.md](../current.md)
- マイルストーン: [../../MILESTONE.md](../../MILESTONE.md)

---

## 宣言文（予定）

> 「SAP が、Favnir の型になった。そして今、Favnir が SAP を語り始めた。
>
>  `ctx.sap.query<SalesOrder>().select("Id", "Amount").filter(status == Active).execute()`
>
>  `$metadata` を読めば Favnir の型が生まれ、
>  `$batch` でネットワークを束ね、
>  Lambda SnapStart で応答が速くなる。
>
>  それが、Favnir SAP Advanced 1.0 である。」
