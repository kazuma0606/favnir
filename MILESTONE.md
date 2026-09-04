# Favnir Milestones

## v100.0.0（2026-09-04）— SAP Platform 1.0 宣言

> 「Favnir が、SAP のプラットフォームになった。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに動き、
>  `ctx.sap_env("PRD")` で本番に向き、
>  Snowflake と型安全に JOIN し、
>  `!Approval` で人間の承認を型に閉じ込め、
>  KPI が SAC に流れ、Slack が鳴り、
>  `Masked<T>` が個人情報を守り、
>  `!Audit` が証跡を刻む。
>
>  OAuth2 が認証し、Circuit Breaker が守り、SLA が測る。
>
>  これが、Favnir SAP Platform 1.0 である。
>  SAP と Favnir の 5 年間の旅が、今、完成した。」

**SAP Platform 1.0** の宣言バージョン。v86.0〜v99.9 で構築した SAP 連携機能群の完成を宣言した。
テスト数: 4,279。

**SAP Platform 1.0（v86.0〜v99.9）達成内容:**
- **sap-odata Rune**: OData v4 型安全クライアント・QueryBuilder・$select/$expand/$filter
- **SapEnvironment**: Prd/Qas/Dev/Custom 環境切り替え
- **CircuitBreaker**: 障害遮断・自動復旧ガードレール
- **TenantContext**: マルチテナント分離型
- **Masked\<T\>**: GDPR データマスキング型
- **SlaDefinition / fav sla-check**: SLA 定義・違反検出 CLI
- **SAP Workflow**: `!Approval` 型・IFlowClient
- **SAP Analytics**: KpiDefinition・BW クエリ・SAC プッシュ・Slack アラート
- **統合ガイド**: sap-platform.mdx / sap-migration.mdx / sap-enterprise-checklist.mdx

---

## v99.0.0（2026-09-03）— SAP Analytics 1.0 宣言

> 「SAP のデータが、洞察になった。
>
>  `KpiDefinition<SalesOrder>` が売上の健全性を測り、
>  BW クエリの結果が SAC に流れ、
>  閾値を超えた瞬間に Slack が鳴る。
>
>  それが、Favnir SAP Analytics 1.0 である。」

**SAP Analytics 1.0** の宣言バージョン。v98.1.0〜v98.9.0 で実装した
KPI 型定義・BW クエリ・SAC プッシュ・KPI アラート・CLI・E2E デモ・サイトドキュメントの完成を宣言した。
テスト数: 4,257。

**SAP Analytics 1.0（v98.1〜v98.9）達成内容:**
- **`KpiDefinition<T>` / `KpiSnapshot<T>`**: KPI を型で定義し計測結果をスナップショットとして保持
- **`BwQuery<T>` / `BwResult<T>`**: BW/4HANA クエリの型安全なインターフェース
- **`SacDataset` / `sac_push_mock`**: SAC へのデータプッシュ API
- **`report_to_sac_rows`**: `SalesReport` → SAC CSV 行リスト変換
- **`KpiAlert` / `format_kpi_alert`**: 閾値超えアラートの型と整形関数
- **`fav report --sap`**: HTML レポート生成 CLI コマンド
- **`analytics_demo/`**: 日次売上 KPI → SAC → アラートの E2E デモ
- **`sap-analytics.mdx`**: KPI 定義・BW クエリ・SAC プッシュの完全ガイド

---

## v98.0.0（2026-09-02）— SAP Workflow 1.0 宣言

> 「Favnir が、人間の判断を型に閉じ込めた。
>
>  `!Approval` エフェクトが pipeline のシグネチャに現れた時、
>  それはコードが「ここで人間の承認が必要」と語っているのだ。
>
>  承認フローが型になった。それが、Favnir SAP Workflow 1.0 である。」

**SAP Workflow 1.0** の宣言バージョン。v97.1.0〜v97.9.0 で実装した
承認フロー型化・iFlow コネクタ・モッキング基盤・ガイドドキュメントの完成を宣言した。
テスト数: 4,235。

**SAP Workflow 1.0（v97.1〜v97.9）達成内容:**
- **`!Approval` エフェクト型**: pipeline シグネチャに承認要件を型で表現
- **`TaskDecision` variant**: `Approve` / `Reject(String)` による型安全な承認結果
- **`ApprovalClient` interface**: `request_approval(client, subject, context) -> TaskDecision`
- **`route_by_approval_result` pipeline**: 承認結果に基づく型安全なルーティング
- **`IFlowClient`**: SAP BTP iFlow コネクタ（base_url / oauth_url / client_id）
- **`iflow_send`**: SAP iFlow メッセージ送信 Rune
- **`MockWorkflowClient`**: テスト用承認クライアント（auto_approve / reject_reason）
- **`Ctx.mock_workflow`**: テスト用 AppCtx 構築関数（MockWorkflowClient 対応）
- **`sap-workflow.mdx`**: 承認フロー設計・実装・テストの完全ガイド

---

## v97.0.0（2026-09-01）— SAP Multi-system 1.0 宣言

> 「Favnir が、SAP の境界を越えた。
>
>  `ctx.sap_env("PRD")` で本番に向き、
>  SAP のデータが Snowflake に流れ、
>  カスタムサービスの型も `fav infer` が生み出す。
>
>  それが、Favnir SAP Multi-system 1.0 である。」

**SAP Multi-system 1.0** の宣言バージョン。v96.1.0〜v96.9.0 で実装した
型安全な環境切替・マルチ環境設定・SAP→Snowflake 同期・Clean Core・CrossSystem JOIN・RetryPolicy の完成を宣言した。
テスト数: 4,213。

**SAP Multi-system 1.0（v96.1〜v96.9）達成内容:**
- **SapEnvironment 型**: `Prd` / `Qas` / `Dev` + `ctx.sap_env("PRD")` による型安全な環境切替
- **マルチ環境設定**: `fav.toml [sap.environments]` で PRD/QAS/DEV を並列定義
- **SAP→DuckDB エクスポート**: `write_parquet` + DuckDB 分析パイプライン
- **SAP→Snowflake 同期**: `execute_raw` でリアルタイムロード
- **カスタム OData**: `fav infer --sap-service-name` でカスタムサービス型生成
- **CleanCoreClient**: S/4HANA Cloud Clean Core REST API ラッパー
- **CrossSystem.join<A,B>**: SAP エンティティ × Snowflake テーブルの型安全 JOIN
- **RetryPolicy / SapConnectionPool**: 本番運用向け接続管理型

---

## v96.0.0（2026-09-01）— SAP Real-time 1.0 宣言

> 「SAP が、Favnir の時間軸で動き始めた。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに変化を知り、
>  Deep Insert で一気に書き込み、`fav sap-mock` でオフラインでもテストできる。
>
>  それが、Favnir SAP Real-time 1.0 である。」

**SAP Real-time 1.0** の宣言バージョン。v95.1.0〜v95.9.0 で実装した
OData $delta / SAP Event Mesh / Deep Insert / Function Import / バッチ部分失敗 / `fav sap-mock` の完成を宣言した。
テスト数: 4,188。

**SAP Real-time 1.0（v95.1〜v95.9）達成内容:**
- **OData $delta**: `DeltaResult<T>` / `DeletedEntity` / `ctx.sap.delta_fetch<T>()`
- **SAP Event Mesh**: `SapEventClient` interface / `SapEventMessage` / `pipeline_realtime.fav`
- **Deep Insert**: `NewSalesOrderWithItems` / `create_sales_order_deep`
- **Function/Action Import**: `FunctionImportParam` / `function_import<T>` / `action_import`
- **バッチ部分失敗**: `BatchItemResult<T>` / `PartialSuccess<T>` / `batch_with_partial`
- **fav sap-mock**: `SapMockServer` / `cmd_sap_mock` / `Some("sap-mock")` CLI コマンド

---

## v95.0.0（2026-08-30）— SAP Advanced 1.0 宣言

> 「`ctx.sap.batch(req)` で複数 SAP エンティティをまとめて更新できる。
>  `QueryBuilder<T>` で型安全なクエリを組み立て、`fetch_all_pages` で全件自動取得できる。
>  `fav infer --sap-metadata` で SAP の型定義が自動生成される。
>  Lambda SnapStart でコールドスタートは 93% 削減される。
>  それが、Favnir SAP Advanced 1.0 である。」

**SAP Advanced 1.0** の宣言バージョン。v94.1.0〜v94.9.0 で実装した
OData $batch / Lambda SnapStart / ベンチマーク / E2E デモ / ドキュメント完全化の完成を宣言した。
テスト数: 4,164。

**SAP Advanced 1.0（v94.1〜v94.9）達成内容:**
- **$batch**: `BatchOperation<T>` ADT / `BatchRequest<T>` / `batch_request_builder<T>` / `ctx.sap.batch(req)`
- **Lambda SnapStart**: `infra/lambda/sap-sync/main.tf`（SnapStart 設定・コールドスタート 93% 削減）
- **コールドスタートベンチマーク**: `scripts/bench_sap_coldstart.sh`
- **SAP 総合ベンチマーク**: `fav bench --sap`（`bench_sap_all` 6 ベンチマーク）
- **E2E デモ（シナリオ 5）**: `infra/e2e-demo/sap-odata/pipeline_advanced.fav`（$batch 完全デモ）
- **ドキュメント**: `site/content/docs/guides/sap-integration.mdx`（SAP Advanced Era 総まとめ）

---

## v94.0.0（2026-08-30）— SAP Metadata Infer 1.0 宣言

> 「`fav infer --sap-metadata <url>` と打てば、SAP の $metadata から Favnir 型定義が自動生成される。
>  EntityType は `type` に、EnumType は ADT に、NavigationProperty は ExpandClause ヘルパーに変換される。
>  それが、Favnir SAP Metadata Infer 1.0 である。」

**SAP Metadata Infer 1.0** の宣言バージョン。v93.1.0〜v93.9.0 で実装した
$metadata XML → Favnir 型定義自動生成機能の完成を宣言した。テスト数: 4,142。

**SAP Metadata Infer 1.0（v93.1〜v93.9）達成内容:**
- **EDMX パーサー（スタブ）**: `parse_edmx` / `EdmxEntityType` / `EdmxEnumType` 構造体
- **型変換**: `edm_type_to_favnir`（8 種マッピング）/ `entity_type_to_favnir` / `enum_type_to_favnir`
- **NavigationProperty**: `nav_property_to_favnir_comment` / `nav_to_expand_helper_fn`
- **フォーマット**: `apply_fmt_to_generated`（`fmt_source_str` バックエンド）
- **CLI**: `fav infer --from sap --metadata <url>` / `--metadata-file <path>`（`CmdInferSapMetadata` / `CmdInferSapMetadataFile`）
- **ドキュメント**: `site/content/docs/cli/infer.mdx`（新規）/ `sap-odata.mdx` EDM 型マッピング表追加

---

## v93.0.0（2026-08-29）— SAP QueryBuilder 1.0 宣言

> 「`query<SalesOrder>() |> with_filter(Eq("SoldToParty", "CUST-001")) |> with_top(50)` と書けば、
>  型安全な OData クエリが組み立てられる。
>  ページネーションは `fetch_all_pages` で自動化され、N+1 は W060 で防がれる。
>  それが、Favnir SAP QueryBuilder 1.0 である。」

**SAP QueryBuilder 1.0** の宣言バージョン。v92.1.0〜v92.9.0 で実装した
汎用クエリビルダー（QueryBuilder<T> / Page<T> / fetch_all_pages / W060 N+1 lint）の完成を宣言した。テスト数: 4,120。

**SAP QueryBuilder 1.0（v92.1〜v92.9）達成内容:**
- **型**: `QueryBuilder<T>`（ファントム型付き汎用クエリビルダー、全フィールド Option）
- **Fluent API**: `with_select` / `with_expand` / `with_filter` / `with_top` / `with_skip` / `with_order_by`
- **ページネーション**: `Page<T>`（items / next_link / total）+ `fetch_all_pages`（スタブ）
- **lint**: W060 N+1 クエリ検出（`List.map` / `List.flat_map` 内の `ctx.sap.*` 呼び出し）
- **E2E**: `pipeline_query.fav`（fetch_all_pages デモ）+ `fav bench --sap-query`
- **ドキュメント**: `sap-odata.mdx` QueryBuilder パターンガイド追加

---

## v92.0.0（2026-08-27）— SAP OData Query 1.0 宣言

> 「`SapQueryClient` を通じて `sales_orders_query(q)` と書けば、
>  `$filter`・`$select`・`$expand` を型で組み立てた OData クエリが発行できる。
>  誤フィールド指定はコンパイル時に検出される。
>  それが、Favnir SAP OData Query 1.0 である。」

**SAP OData Query 1.0** の宣言バージョン。v91.1.0〜v91.9.0 で実装した
OData クエリ型基盤（SelectClause / ExpandClause / FilterExpr / 各エンティティ Query / ODataQueryBuilder / SapQueryClient）の完成を宣言した。テスト数: 4,094。

**SAP OData Query 1.0（v91.1〜v91.9）達成内容:**
- **型**: `SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>`（ファントム型付きクエリ型）
- **クエリ型**: `SalesOrderQuery` / `BusinessPartnerQuery` / `MaterialQuery` / `PurchaseOrderQuery` / `JournalEntryQuery`（各エンティティ専用クエリ型）
- **URL生成**: `ODataQueryBuilder<T, Q>` / `build_url`（エンティティパス結合、簡易実装）
- **interface**: `SapQueryClient`（5 クエリメソッド）— 循環 dep 回避のため `query_client.fav` に独立定義
- **実装**: `SapODataClient` / `MockSapClient` が `SapQueryClient` を impl

---

## v91.0.0（2026-08-26）— SAP Ctx 統合 1.0 宣言

> 「`ctx.sap.business_partners(filter)` と書けば、SAP にアクセスできる。
>  設定は `AppCtx` に隠れ、テストは `MockSapClient` で差し替わる。
>  それが、Favnir SAP Ctx 統合 1.0 である。」

**SAP Ctx 統合 1.0** の宣言バージョン。v90.1.0〜v90.9.0 で実装した
`ctx.sap.*` パターン統合（SapClient interface / AppCtx.sap / MockSapClient / Ctx.build / Ctx.mock）の完成を宣言した。テスト数: 4,065。

**SAP Ctx 統合 1.0（v90.1〜v90.9）達成内容:**
- **interface**: `SapClient`（5 メソッド — business_partners / sales_orders / materials / journal_entries / business_partner_by_id）
- **型**: `AppCtx.sap: SapClient`（Capability Context 統合）
- **実装**: `SapODataClient`（本番 HTTP クライアント）/ `MockSapClient`（テスト用スタブ）
- **DI**: `Ctx.build()`（環境変数から自動設定注入）/ `Ctx.mock()`（テスト用 AppCtx 構築）
- **移行**: `pipeline.fav` 全 4 シナリオを `ctx.sap.*` スタイルに書き換え
- **ドキュメント**: `sap-odata.mdx` を `ctx.sap.*` パターンガイドに更新

---

## v90.0.0（2026-08-25）— SAP Integration 1.0 宣言

> 「SAP が、Favnir の型になった。
>  `business_partners()` で得意先を取得し、
>  `sales_orders()` で受注を集計し、
>  `materials()` で在庫を確認し、
>  `journal_entries()` で支払を照合する。
>  世界最大の ERP データが、型安全なパイプラインとして流れる。
>  それが、Favnir SAP Integration 1.0 である。」

**SAP Integration 1.0** の宣言バージョン。v89.1.0〜v89.9.0 で実装した
SAP Integration Era の第 5 スプリント（最終スプリント）の完成を宣言した。テスト数: 4,041。

**SAP Integration 1.0（v89.1〜v89.9）達成内容:**
- **型定義**: `JournalEntry` / `JournalEntryItem` / `JournalFilter` / `DebitCredit`
- **型定義**: `OutstandingPayable` + `match_unposted_orders()`
- **E2E デモ**: 全 4 業務シナリオ（BusinessPartner / Sales / Procurement / Payment）
- **CLI**: `fav infer --from sap --entity <name>` — エンティティ型テンプレート生成
- **ドキュメント**: `site/content/docs/runes/sap-odata.mdx`
- **OSS 整備**: CONTRIBUTING SAP セクション + `.github/ISSUE_TEMPLATE/sap-integration-feedback.md`
- **パフォーマンス計測**: `benchmarks/sap-odata-v89.8.0.json`（Lambda cold start / ページネーション）

---

## v89.0.0（2026-08-24）— SAP Procurement 1.0 宣言

> 「在庫と発注が型になった。
>  Material と PurchaseOrder を並べれば、不足が見える。」

**SAP Procurement 1.0** の宣言バージョン。v88.1.0〜v88.9.0 で実装した
SAP Integration Era の第 4 スプリントの完成を宣言した。テスト数: 4,019。

**SAP Procurement 1.0（v88.1〜v88.9）達成内容:**
- **型定義**: `Material` / `MaterialType` / `MaterialFilter`
- **型定義**: `PurchaseOrder` / `PurchaseOrderItem` / `PurchaseOrderStatus` / `PurchaseOrderFilter`
- **型定義**: `NewPurchaseOrder` / `NewPurchaseOrderItem`
- **型定義**: `StockSeverity` / `StockAlert`
- **品目マスタ**: `material_by_id(cfg, material_id)` — 単一品目取得
- **品目リスト**: `materials(cfg, MaterialFilter)` — 品目絞り込み検索
- **発注検索**: `purchase_orders(cfg, PurchaseOrderFilter)` — フィルタ検索
- **発注取得**: `purchase_order_by_id(cfg, po_number, expand_items)` — 明細展開対応
- **発注作成**: `create_purchase_order(cfg, NewPurchaseOrder)` — 発注伝票作成
- **在庫チェック**: `detect_stock_shortage(orders, materials)` — 受注 × 品目クロスチェック
- **E2E パイプライン**: Scenario 3（`check_stock_vs_orders`）
- **Lambda 基盤**: `infra/e2e-demo/sap-odata/terraform/`（main.tf / ssm.tf / variables.tf）
- **実行スクリプト**: `infra/e2e-demo/sap-odata/scripts/run.sh`

---

## v88.0.0（2026-08-23）— SAP Sales 1.0 宣言

> 「受注が型になった。
>  `sales_orders()` で絞り込み、`sales_order_by_id()` で明細まで取得できる。
>  日次売上レポートが、Favnir の 10 行で書ける。」

**SAP Sales 1.0** の宣言バージョン。v87.1.0〜v87.9.0 で実装した
SAP Integration Era の第 3 スプリントの完成を宣言した。テスト数: 3,997。

**SAP Sales 1.0（v87.1〜v87.9）達成内容:**
- **型定義**: `SalesOrder` / `SalesOrderItem` / `SalesOrderStatus` / `SalesOrderFilter`
- **フィルタ検索**: `sales_orders(cfg, SalesOrderFilter)` — 顧客・ステータス・期間・販売組織でフィルタ
- **単件取得**: `sales_order_by_id(cfg, order_id, expand_items)` — 明細展開対応
- **作成**: `create_sales_order(cfg, NewSalesOrder)` — 明細リスト込みで受注作成
- **集計型**: `CurrencyTotal` / `SalesReport` + `build_sales_report()` / `format_sales_report()`
- **ページネーション**: `odata_list_paged()` / `odata_collect_all()`
- **E2E パイプライン**: `infra/e2e-demo/sap-odata/pipeline.fav`（日次売上レポート → S3 JSON）
- **テスト**: `sap_odata.test.fav` に SalesOrder CRUD + ページネーションテスト追加

---

## v87.0.0（2026-08-23）— SAP Master Data 1.0 宣言

> 「SAP の BusinessPartner が、Favnir の型になった。
>  得意先も仕入先も、`business_partners()` で型安全に取得できる。」

**SAP Master Data 1.0** の宣言バージョン。v86.1.0〜v86.9.0 で実装した
SAP Integration Era の第 2 スプリントの完成を宣言した。テスト数: 3,975。

**SAP Master Data 1.0（v86.1〜v86.9）達成内容:**
- **型定義**: `BusinessPartner` / `BusinessPartnerAddress` / `BusinessPartnerCategory`
- **フィルタ検索**: `business_partners(cfg, BusinessPartnerFilter)` — 国・カテゴリ・更新日・件数でフィルタ
- **単件取得**: `business_partner_by_id(cfg, partner_id, expand_address)`
- **作成**: `create_business_partner(cfg, NewBusinessPartner)`
- **更新**: `update_business_partner(cfg, partner_id, BusinessPartnerPatch)`
- **E2E パイプライン**: `infra/e2e-demo/sap-odata/pipeline.fav`（BusinessPartner → S3 JSON 同期）
- **Rune Registry**: `sap-odata` v86.8.0 をデプロイ・登録

---

## v86.0.0（2026-08-23）— SAP Foundation 1.0 宣言

> 「SAP に、型安全に接続できるようになった。
>  `fav.toml [sap]` を書けば、Favnir が SAP OData v4 と話せる。」

**SAP Foundation 1.0** の宣言バージョン。v85.1.0〜v85.9.0 で実装した
SAP Integration Era の第 1 スプリントの完成を宣言した。テスト数: 3,953。

**SAP Foundation 1.0（v85.1〜v85.9）達成内容:**
- **Rust 基盤**: `SapTomlConfig` / `inject_sap_config()` / `fav.toml [sap]` 解析・env 注入
- **Favnir 型**: `SapConfig` / `SapError` / `SapErrorCode` / `ODataParams`（`runes/sap-odata/types.fav`）
- **Rune**: `sap-odata`（`odata_get` / `odata_list` / `sap_config_from_env` / `basic_auth_header`）
- **インフラ**: Docker Compose モックサーバー（`infra/e2e-demo/sap-odata/`）+ SSM Parameter Store Terraform（`infra/sap/`）
- **テンプレート**: `fav new` が `[sap]` コメントブロックを生成（`default_fav_toml()`）

---

## v85.0.0（2026-08-22）— Favnir 4.0 宣言

> 「テストが型となり、品質が型となり、契約が型となり、観測が型となった。
>
>  `fav test` がパイプラインの正しさを証明し、
>  `QualityGate` が品質基準を守り、
>  `IoContract` がチームを安全に繋ぎ、
>  `AlertRule` が壊れる前に教えてくれる。
>
>  Favnir 4.0 は、データパイプラインの品質を
>  コードと同じ言語で語れる、唯一の言語である。」

**Favnir 4.0** の宣言バージョン。v80.1.0〜v84.9.0 で実装した
Quality-First Era（Test-Driven Data / Data Quality 2.0 / Pipeline Contracts / Observability 2.0）の完成を宣言した。テスト数: 3,931。

**Quality-First Era（v80.1〜v84.9）達成内容:**
- **Sprint 1: Test-Driven Data** — `TestSuite` / `StageTestCase` / `GoldenDataset` / `SchemaSnapshot` / `fav test`（v80.1〜v81.0）
- **Sprint 2: Data Quality 2.0** — `QualityRule` / `QualityCheck` / `QualityGate` / `AnomalyDetector` / `fav quality`（v81.1〜v82.0）
- **Sprint 3: Pipeline Contracts 1.0** — `IoContract` / `SlaContract` / `ContractRegistry` / `fav verify --contract`（v82.1〜v83.0）
- **Sprint 4: Observability 2.0** — `PipelineMetrics` / `AlertRule` / `SloTarget` / `HealthDashboard` / `fav observe`（v83.1〜v84.0）
- **Sprint 5: Favnir 4.0 宣言** — E2E ショーケース統合・ドキュメント完全化・OSS 公開強化（v84.1〜v85.0）

---

## v84.0.0（2026-08-21）— Observability 2.0 宣言

> 「メトリクスが型になり、アラートが型になり、SLO が型になった。
>  Favnir のパイプラインは壊れる前に教えてくれる。」

**Observability 2.0** の宣言バージョン。v83.1.0〜v83.9.0 で実装した
パイプライン観測性型基盤の完成を宣言した。テスト数: 3,909。

**v83.1.0〜v83.9.0 達成内容:**
- `PipelineMetrics` / `StageMetrics` / `compute_pipeline_metrics` / `format_metrics_summary` / `slowest_stage`（実行統計）— v83.1.0
- `AlertSeverity` / `AlertThreshold` / `AlertRule` / `AlertFiring` / `evaluate_alert_rules`（アラート型）— v83.2.0
- `SloTarget` / `SloMeasurement` / `SloStatus` / `compute_slo_status` / `format_slo_status`（SLO 型）— v83.3.0
- `ResourceUsage` / `ExecutionCost` / `CostBudget` / `BudgetStatus` / `evaluate_cost_budget` / `format_cost_report`（コスト追跡）— v83.4.0
- `TraceContext` / `TraceSpan` / `format_trace_span` / `compute_span_duration`（分散トレーシング）— v83.5.0
- `PerfBaseline` / `PerfRegression` / `detect_perf_regression` / `format_regression_report`（パフォーマンス回帰検知）— v83.6.0
- `ObserveFormat` / `ObserveOptions` / `ObserveReport` / `format_observe_report` / `cmd_observe`（`fav observe` コマンド）— v83.7.0
- `HealthStatus` / `PipelineHealth` / `HealthDashboard` / `compute_pipeline_health` / `format_health_dashboard`（健全性ダッシュボード）— v83.8.0
- スプリント統合確認・コードフリーズ — v83.9.0

---

## v83.0.0（2026-08-21）— Pipeline Contracts 1.0 宣言

> 「パイプライン間の約束が型になった。
>  `IoContract` がインターフェースを定義し、`SlaContract` が応答時間を保証し、
>  `ContractRegistry` がチームを繋ぐ。
>  Favnir のパイプラインは今、契約で安全に接続できる。」

**Pipeline Contracts 1.0** の宣言バージョン。v82.1.0〜v82.9.0 で実装した
パイプライン間の入出力契約型基盤の完成を宣言した。テスト数: 3,887。

**v82.1.0〜v82.9.0 達成内容:**
- `ContractField` / `ContractFieldType` / `IoContract` / `validate_io_contract`（IoContract 型基盤）— v82.1.0
- `SlaTarget` / `SlaContract` / `SlaStatus` / `evaluate_sla` / `format_sla_status`（SLA 遵守型）— v82.2.0
- `ContractDependency` / `DependencyGraph` / `build_dependency_graph` / `detect_circular_dependencies`（パイプライン間契約依存）— v82.3.0
- `ViolationKind` / `ContractViolation` / `ContractViolationReport` / `format_violation_report`（契約違反詳細レポート）— v82.4.0
- `infer_contract_from_schema` / `merge_contracts` / `format_contract_as_toml`（スキーマから契約自動生成）— v82.5.0
- `ContractVersion` / `CompatibilityResult` / `check_contract_compatibility`（契約バージョニング・後方互換チェック）— v82.6.0
- `VerifyContractOptions` / `ContractVerifyResult` / `cmd_verify_contract` / `format_verify_result`（`fav verify --contract` 強化）— v82.7.0
- `ContractRegistryEntry` / `ContractRegistry` / `format_registry_listing`（契約レジストリ）— v82.8.0
- 統合テスト・コードフリーズ — v82.9.0

---

## v82.0.0（2026-08-20）— Data Quality 2.0 宣言

> 「品質が型になった。外れ値はコンパイル時に検出され、
>  スキーマドリフトはパイプライン起動前に止まる。
>  Favnir のデータは今、品質を型で保証する。」

**Data Quality 2.0** の宣言バージョン。v81.1.0〜v81.9.0 で実装した
データ品質ルールの型化を完成させ宣言した。

**v81.1.0〜v81.9.0 達成内容:**
- `QualityRule` / `QualityCheck` / `run_quality_check` / `QualityViolation`（品質ルール基盤）— v81.1.0
- `DistributionStats` / `StatisticalCheck` / `detect_outliers`（統計的品質チェック）— v81.2.0
- `SchemaDriftDetector` / `DriftTolerance` / `DriftResult` / `detect_schema_drift`（スキーマドリフト検出）— v81.3.0
- `QualityDimension` / `DimensionScore` / `QualityScore` / `compute_quality_score`（品質スコアリング）— v81.4.0
- `ProvenanceQualityEntry` / `ProvenanceQualityReport` / `worst_quality_source`（来歴付き品質レポート）— v81.5.0
- `QualityGate` / `GateDecision` / `evaluate_quality_gate`（品質ゲート）— v81.6.0
- `ReportFormat` / `QualityReportOptions` / `build_quality_report` / `cmd_quality_report`（`fav quality report`）— v81.7.0
- `AnomalyDetector` / `AnomalyResult` / `detect_anomaly` / `scan_for_anomalies`（異常検知）— v81.8.0
- 統合テスト・コードフリーズ — v81.9.0

---

## v81.0.0（2026-08-19）— Test-Driven Data 1.0 宣言

> 「テストが型になり、カバレッジが数値になり、スキーマ変更が検出される。
>  Favnir のパイプラインは今、その正しさを `fav test` で証明できる。」

**Test-Driven Data 1.0** の宣言バージョン。v80.1.0〜v80.9.0 で実装した
テスト駆動データパイプライン基盤の完成を宣言した。

**v80.1.0〜v80.9.0 達成内容:**
- `TestSuite` / `TestCase` / `TestStatus` / `run_test_suite`（テスト実行基盤）— v80.1.0
- `TestFixture` / `DataFactory` / `FieldSpec`（テストデータ生成）— v80.3.0
- `PropertyTest` / `InvariantKind` / `run_property_test`（プロパティベーステスト）— v80.4.0
- `StageTestCase` / `run_stage_test`（ステージ単体テスト）— v80.5.0
- `TestCoverageReport` / `compute_test_coverage`（テストカバレッジ計測）— v80.6.0
- `SchemaSnapshot` / `compare_schema_snapshots`（スキーマスナップショットテスト）— v80.7.0
- `TestReport` / `format_junit_xml` / `format_test_summary`（CI 統合レポート）— v80.8.0
- 統合確認・コードフリーズ — v80.9.0

---

## v80.0.0（2026-08-16）— Favnir 3.0 宣言

> 「時間が型となり、来歴が型となり、正しさが型となり、実行戦略が型となった。
>
>  FreshnessPolicy がデータの鮮度を保証し、ProvenanceTag が来歴を追い、
>  PipelineInvariant が不変条件を証明し、!Adaptive がコストを最適化する。
>
>  Favnir 3.0 は、データパイプラインが「何を・どこから・どう正しく・どう速く」
>  処理するかを、すべて型で語れる言語である。」

**Favnir 3.0** の宣言バージョン。v75.1〜v79.9 で実装した
Temporal / Provenance / Verifiable / Execution Effects の全スプリントの完成を宣言した。

**v75.1〜v79.9 達成内容:**
- Temporal Data Native（FreshnessPolicy / AsOfQuery / SCD）— v75.x〜v76.x
- Data Provenance 1.0（ProvenanceTag / TracedData / OpenLineage）— v77.x
- Verifiable Pipelines（PipelineInvariant / invariant / check_aggregate_invariant）— v78.x
- Execution Effects 1.0（!Adaptive / !Cached / ExecutionStrategy）— v79.x
- E2E ショーケース統合確認（infra/e2e-demo/favnir3-showcase/）— v79.9.0

---

## v79.0.0（2026-08-16）— Execution Effects 1.0 宣言

> 「`!Cached` がメモを持ち、`!Adaptive` が状況を読み、`!Parallel` が仕事を分ける。
>  実行戦略が型となった Favnir は、最適解を自ら選ぶ。」

**Execution Effects 1.0** の宣言バージョン。v78.1〜v78.9 で実装した
Execution Effects 基盤の完成を宣言した。

**v78.1〜v78.9 達成内容:**
- `CacheEntry` / `CacheStats` / `simulate_lru_cache` / `format_cache_stats_report`（LRU キャッシュ統計）— v78.1.0
- `hit_rate` / `merge_cache_stats`（ヒット率・統計マージ）— v78.2.0
- `ExecutionStrategy` / `select_join_strategy`（結合戦略選択）— v78.3.0
- `CostEstimate` / `combine_costs` / `estimate_cost`（コスト推定）— v78.4.0
- `PlanStage` / `ExecutionPlan` / `format_execution_plan`（実行計画可視化）— v78.5.0
- `ParallelConfig` / `PartitionPlan` / `plan_parallel_execution` / `format_parallel_plan`（並列実行）— v78.6.0
- `ExecutionMode` / `ExecutionModeSelector` / `select_execution_mode`（実行モード選択）— v78.7.0
- `PlanCacheEntry` / `PlanCache` / `lookup_plan` / `insert_plan`（実行計画キャッシュ）— v78.8.0
- 安定化・E2E テスト（`execution_effects_full_sprint_all_stable` / `execution_effects_e2e_pipeline_runs`）— v78.9.0

---

## v78.0.0（2026-08-16）— Verifiable Pipelines 宣言

> 「不変条件が型となり、反例がコンパイラから届く。
>  Favnir のパイプラインは今、その正しさを証明できる。」

**Verifiable Pipelines** の宣言バージョン。v77.1〜v77.9 で実装した
Verifiable Pipelines 基盤の完成を宣言した。

**v77.1〜v77.9 達成内容:**
- `PipelineInvariant` / `InvariantViolation` / `check_count_invariant`（不変条件基盤）— v77.1.0
- `FilterInvariant` / `check_filter_invariant`（フィルター系不変条件）— v77.2.0
- `AggregateInvariant` / `AggregateProperty` / `check_aggregate_invariant`（集約系不変条件）— v77.3.0
- `JoinInvariant` / `JoinType` / `JoinNullPolicy` / `check_join_invariant`（Join 系不変条件）— v77.4.0
- `VerificationReport` / `cmd_verify` / `format_verification_report`（verify コマンド基盤）— v77.5.0
- `CiVerificationConfig` / `CiResult` / `run_ci_verification` / `format_ci_result_summary`（CI 統合）— v77.6.0
- `CounterExampleResult` / `generate_counter_example_values`（反例自動生成）— v77.7.0
- `ProbabilisticContract` / `check_probabilistic_invariant`（確率的契約）— v77.8.0
- 安定化・E2E テスト（`verifiable_full_sprint_all_stable` / `verifiable_e2e_pipeline_verified`）— v77.9.0

---

## v77.0.0（2026-08-15）— Data Provenance 1.0 宣言

> 「データの来歴が型となった。どこから来て、何を経て、PII がどこで消えたかを
>  Favnir が型で追跡する。GDPR はコンパイル時に通る。」

**Data Provenance 1.0** の宣言バージョン。v76.1〜v76.9 で実装した
Data Provenance 基盤の完成を宣言した。

**v76.1〜v76.9 達成内容:**
- `DataSource` / `DataSourceType` / `ProvenanceTag` / `format_provenance_tag`（来歴型基盤）— v76.1.0
- `TracedData` / `map_traced` / `merge_provenance`（来歴付きデータ型）— v76.2.0
- `PiiProvenanceReport` / `detect_pii_in_tag` / `ErasurePlan` / `generate_erasure_plan`（PII・GDPR）— v76.3.0
- `OpenLineageFacet` / `provenance_to_openlineage` / `format_openlineage_json`（OpenLineage 統合）— v76.4.0
- `LineageNodeType` / `LineageNode` / `LineageEdge` / `LineageGraph` / `format_lineage_dot`（グラフ可視化）— v76.5.0
- `PipelineProvenanceChain` / `chain_provenance` / `format_chain_report`（Cross-pipeline）— v76.6.0
- `DataProductSla` / `ProvenancePolicy` / `DataProduct` / `validate_data_product`（Data product 型）— v76.7.0
- `PiiPolicy` / `ProvenanceContract` / `validate_provenance_contract`（Provenance contracts）— v76.8.0
- 安定化・E2E テスト（`provenance_full_sprint_all_stable` / `provenance_e2e_pipeline_valid`）— v76.9.0

---

## v76.0.0（2026-08-15）— Temporal Data Native 宣言

> 「鮮度が型となり、SCD が構造となり、タイムトラベルが API となった。
>  Favnir のパイプラインは今、時間軸を型で保証する。」

**Temporal Data Native** の宣言バージョン。v75.1〜v75.9 で実装した
Temporal Data Native 基盤の完成を宣言した。

**v75.1〜v75.9 達成内容:**
- `FreshnessPolicy`（鮮度ポリシー型・Fail/Warn 戦略）— v75.1.0
- `TemporalRange` / `AsOfQuery` / `unix_secs_to_utc` / `is_leap`（時点型・UTC変換）— v75.2.0
- `ScdRow` / `apply_scd2_update` / `apply_scd1_update`（SCD 2.0 型安全更新）— v75.3.0
- `TemporalJoinConfig` / `format_temporal_join_sql`（時点結合SQL生成）— v75.4.0
- `RetentionPolicy` / `apply_retention_check`（データ保持ポリシー）— v75.5.0
- `StreamFreshnessMonitor` / `check_stream_lag`（ストリーム遅延監視）— v75.6.0
- `TemporalContract` / `validate_temporal_contract`（統合コントラクト検証）— v75.7.0
- `TimeTravelQuery` / `cmd_time_travel` / `parse_time_travel_timestamp`（タイムトラベルSQL）— v75.8.0
- 安定化・E2E テスト（`temporal_full_sprint_all_stable` / `temporal_e2e_pipeline_valid`）— v75.9.0

---

## v75.0.0（2026-08-14）— Favnir 2.0 宣言

> 「compiler.fav が Favnir を完全に記述し、型システムが次元と制約を保証する。
>  依存型がベクトルの次元を守り、refined type がゼロ除算をコンパイル時に止める。
>  VS Code がパイプラインを補完し、AI がエラーを修正し、
>  実際のデータチームが本番で Favnir を走らせている。
>
>  データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  Favnir が Favnir 自身を運用し、Rune マーケットプレイスが
>  コミュニティの知恵を型安全なピースとして流通させる。
>
>  これが Favnir v75.0 — Favnir 2.0 の姿である。」

**Favnir 2.0** の宣言バージョン。v74.1〜v74.9 で実装した
Rune マーケットプレイス・マルチテナント・Documentation Site 2.0・OSS Hardening・
Pipeline Scheduling・fav audit 拡張・コミュニティ Rune 品質基準・統合デモ・安定化の統合を宣言した。

**v74.1〜v74.9 達成内容:**
- Rune マーケットプレイス（`RunePackage` / `search_rune_packages` / `generate_rune_registry_index`）— バージョン管理・依存解決
- マルチテナント Runtime（`TenantConfig` / `TenantIsolationLevel` / `resolve_tenant_config`）— テナント分離
- Documentation Site 2.0（`DocSiteEntry` / `generate_doc_site_index`）— 言語リファレンス自動生成
- OSS Hardening（`OssLicenseEntry` / `scan_oss_licenses`）— ライセンス監査
- Pipeline Scheduling（`ScheduleEntry` / `validate_cron_expr` / `cmd_schedule_list`）— cron 管理
- fav audit 拡張（`DepVulnerability` / `format_audit_deps_report` / `apply_audit_fix`）— 依存セキュリティ
- コミュニティ Rune 品質基準（`RuneValidationReport` / `validate_rune_score`）— 公開品質保証
- 統合デモ（`infra/e2e-demo/favnir2-showcase/`）— v71〜v74 全機能ショーケース
- 安定化・コードフリーズ（v74.9.0）— Favnir 2.0 前最終調整

---

## v74.0.0（2026-08-13）— Production Proven

> 「データコントラクトがスキーマ境界を守り、品質スコアが劣化を警告する。
>  PII が型で保護され、監査ログが法的要件を満たす。
>  Favnir が Favnir 自身を運用し、GitHub Action が CI に溶け込む。
>
>  これが Favnir v74.0 — Production Proven の姿である。」

**Production Proven** の宣言バージョン。v73.1〜v73.9 で実装した
データコントラクト・品質スコア・PII 保護・監査ログ・SLA 監視・Rune 品質パス・
ドッグフーディング Sprint・GitHub Actions 公式 Action・安定化の統合を宣言した。

**v73.1〜v73.9 達成内容:**
- データコントラクト（`DataContract` / `validate_contract_schema`）— スキーマ境界の型安全保証
- 品質スコア（`QualityReport` / `compute_quality_report`）— データ劣化の自動警告
- PII 検出・マスキング（`mask_pii_fields` / `PiiMaskStrategy`）— 型での個人情報保護
- 監査ログ + OpenLineage（`AuditLogEntry` / `OpenLineageEvent`）— 法的要件を満たすリネージ追跡
- SLA 監視（`SlaConfig` / `check_sla`）— アラート統合
- Rune 品質パス（`rune_linalg_matmul` / `rune_stats_mean_std`）— VM primitive 接続
- ドッグフーディング Sprint（`fav/pipelines/`）— Favnir が Favnir を運用
- GitHub Actions 公式 Action（`.github/actions/setup-fav/`）— CI に溶け込む
- 安定化・コードフリーズ（v73.9.0）— Production Proven 前調整

---

## v73.0.0（2026-08-13）— Developer Experience 2.0

> 「VS Code がパイプラインを補完し、AI がエラーを修正し、
>  REPL が型を即座に返し、Playground がコードを世界と共有する。
>  自然言語一文が、型安全なパイプラインの雛形になる。
>
>  これが Favnir v73.0 — Developer Experience 2.0 の姿である。」

**Developer Experience 2.0** の宣言バージョン。v72.1〜v72.9 で実装した
VS Code 拡張・AI アシスタント・REPL 2.0・Playground 2.0・`fav learn` の統合を宣言した。

**v72.1〜v72.9 達成内容:**
- VS Code 拡張（`editors/vscode/`）— LSP 統合・シンタックスハイライト・型ホバー・エラーアンダーライン
- AI エラーアシスタント（`fav ai explain` / `fav ai fix`）— エラーコード + ソースを AI に送信して説明・修正
- `fav ai generate` — 自然言語 → Favnir パイプライン雛形生成
- REPL 2.0 — `:timing on/off` モード・TAB 補完ヘルパー（`repl_tab_complete`）
- Playground 2.0 — テンプレートギャラリー（5 エントリ）・共有リンク（`/playground?code=<base64>`）
- `fav init` テンプレートギャラリー拡充 — ai-etl / streaming / enterprise / data-quality / distributed
- `fav watch` 2.0 — `--on-change <cmd>` フラグ（任意コマンドを変更検知時に実行）
- `fav learn` — インタラクティブチュートリアル（5 章: パイプライン → 型 → Rune → AI → 分散実行）

---

## v72.0.0（2026-08-11）— Type System 2.0

> 「依存型がベクトルの次元を守り、refined type がゼロ除算を型で止める。
>  Phantom type が ID の混用を防ぎ、定数がコンパイル時に評価される。
>  AOT バイナリが Docker 不要で動き、WASM がパイプラインをブラウザへ運ぶ。
>
>  これが Favnir v72.0 — Type System 2.0 の姿である。」

**Type System 2.0** の宣言バージョン。v71.1〜v71.9 で実装した
依存型・refined type・phantom type・const eval・generic constraints・AOT・WASM・型推論強化の統合を宣言した。

**v71.1〜v71.9 達成内容:**
- 依存型 `Vec<T>[N]`: 次元違いベクトルを型で防止（E0421）
- Refined Types: `type PositiveFloat = Float where self > 0.0`（E0425）
- Phantom Types: `type UserId = phantom String`（ID 混用防止）
- Const Eval: `const EMBED_DIM: Int = 1536`（コンパイル時定数）
- Generic Constraints: `<T: A & B>`、`<T: impl A>`
- AOT Native: `fav build --target native --arch arm64`
- WASM: `fav build --target wasm`（`\0asm` マジック確認）
- 型推論: `bind n <- fn()` 型注釈省略可

---

## v71.0.0（2026-08-09）— Language Complete 1.0

> 「compiler.fav が Favnir の全構文を処理し、
>  積み残しのない CI が毎回グリーンで終わる。
>  エラーメッセージは修正方法を即座に示し、
>  fav migrate が旧コードを自動で現代に変換する。
>
>  これが Favnir v71.0 — Language Complete 1.0 の姿である。」

**Language Complete 1.0** の宣言バージョン。v70.1〜v70.9 で実装した
compiler.fav 完全化・診断 UI 強化・fav migrate・bench.yml strict mode の統合を宣言した。

**v70.1〜v70.9 達成内容:**
- compiler.fav: 2 段メソッドチェーン / bind 分割束縛 / if-guard パターン対応
- `fav migrate`: !Effect → ctx.io.* 自動変換
- `fav bench --all`: JSON 形式ベンチマーク出力
- ErrorReport / `suggest_similar_name` 診断 UI
- `fav self-coverage`: self-hosting 網羅率レポート
- `fav doctor`: Paper Rune 検出 / CHANGELOG 整合性チェック
- bench.yml strict mode 化（Compare ステップ無条件実行）

---

## v70.0.0（2026-08-08）— Intelligent ETL 1.0

> 「型チェックが、LLM の出力を安全にする。
>  ベクトルの次元は型で保証され、スキーマ違反は推論の前に止まる。
>  自動微分は数値安定性を型レベルで保ち、
>  デバッガがパイプラインを時間遡行し、AI が次の最適化を提案する。
>  型安全な並列処理が、AI パイプラインをクラスタ規模で動かす。
>
>  Favnir は「AI データエンジニアリングのための型安全言語」になった。
>
>  これが Favnir v70.0 — Intelligent ETL 1.0 の姿である。」

**Intelligent ETL 1.0** の宣言バージョン。v65.1〜v69.9 で実装した
Math Rune 群（linalg/stats/autodiff/optim/numeric/timeseries/ml）・
AI Rune 群（embed/llm/VectorDB/serve/featurestore）・
Playground 拡張・E2E AI ETL デモ・パフォーマンスベースラインの統合を宣言した。

**v65.1〜v69.9 達成内容:**
- v65.1〜v65.8（Math Rune 群）: 型付き行列・ベクトル演算・統計・自動微分・最適化
- v66.1〜v66.8（AI-Native Stage Layer）: LLM 型安全抽出・埋め込み・VectorDB・モデルサービング
- v67.1〜v67.9（Developer Intelligence）: デバッガ・タイムトラベル・DAG 可視化・AI アドバイザー
- v68.1〜v68.9（Distributed Favnir）: マルチノード par・チェックポイント・K8s・コスト見積もり
- v69.1〜v69.9（Intelligent ETL 統合）: E2E デモ・Playground・ドキュメント整備・コードフリーズ

---

## v69.0.0（2026-08-07）— Distributed Favnir

> 「`par` がクラスタを越え、チェックポイントが失敗を無効にする。
>  Kubernetes が AI ステージのスケールを決め、
>  コスト見積もりが LLM 呼び出しの予算を守る。
>  型安全な AI パイプラインが、大規模でも壊れない。
>
>  これが Favnir v69.0 — Distributed Favnir の姿である。」

**Distributed Favnir** の宣言バージョン。v68.1〜v68.9 で実装した
分散実行・チェックポイント・K8s・リトライ・分散キャッシュ・コスト見積もり・
AI ルーティング・分散トレーシングの統合を宣言した。

**v68.1〜v68.9 達成内容:**
- v68.1（Multi-Node `par`）: --cluster workers.yaml / --partition-by による分散並列実行
- v68.2（Pipeline Checkpointing）: --checkpoint / --resume による耐障害性・再開
- v68.3（Kubernetes-Native Orchestration）: fav deploy --target kubernetes / Pipeline CRD
- v68.4（Stage Retry Policies）: ExponentialBackoff / LinearBackoff / DeadLetterQueue
- v68.5（Distributed Incremental Cache）: --distributed-cache redis:// / L1/L2 キャッシュ
- v68.6（Cost-Aware Scheduling）: fav cost-estimate --provider --scale / 最適化提案
- v68.7（Multi-Cloud AI Routing）: fav.toml [ai] セクション / --env dev/prod/test
- v68.8（Distributed Observability）: --otel-endpoint / OpenTelemetry / Grafana ダッシュボード
- v68.9（安定化・コードフリーズ）: distributed.mdx 作成・全機能統合確認

**テスト数**: 3541

---

## v68.0.0（2026-08-07）— Developer Intelligence

> 「ステップ実行デバッガが、AI パイプラインの内部を露わにする。
>  時間を遡って本番障害を再現し、DAG 可視化が依存関係を一目で示す。
>  AI アドバイザーがプロファイリングデータを読み、次の最適化を提案する。
>
>  これが Favnir v68.0 — Developer Intelligence の姿である。」

**Developer Intelligence** の宣言バージョン。v67.1〜v67.9 で実装した
デバッグ・可視化・AI 提案・テストツール群の統合を宣言した。

**v67.1〜v67.9 達成内容:**
- v67.1（`fav debug`）: ステップ実行デバッガ（inspect / breakpoint / diff）
- v67.2（Time-Travel Debugging）: --record / --replay / rewind / forward
- v67.3（`fav viz`）: パイプライン DAG 可視化（ascii / svg / mermaid）
- v67.4（`fav suggest`）: AI 最適化アドバイザー（--from-profile）
- v67.5（`fav simulate`）: 合成データパイプラインテスト（--seed）
- v67.6（`Rune.proptest`）: Pipeline Property Testing（forall / shrink / --proptest-runs）
- v67.7（`fav profile --interactive`）: インタラクティブプロファイリング（drill / Suggestion）
- v67.8（`fav doc --math`）: 数式対応ドキュメント生成（MathJax / $$...$$）
- v67.9（安定化）: developer-intelligence.mdx / コードフリーズ

**テスト数**: 3519

---

## v67.0.0（2026-08-06）— AI-Native Stage Layer

> 「LLM の出力にスキーマが付き、ベクトルの次元が型で保証される。
>  埋め込みモデルの選択が型エラーを生まず、
>  リアルタイム推論パイプラインがバックプレッシャー制御下で動く。
>
>  これが Favnir v67.0 — AI-Native Stage Layer の姿である。」

**AI-Native Stage Layer** の宣言バージョン。v66.1〜v66.9 で実装した
9 AI Rune 群と AI Pipeline Lint Rules W055〜W059 の統合を宣言した。

**v66.1〜v66.9 達成内容:**
- v66.1（Rune.vec）: ベクトル演算（normalize / dot / cosine_similarity / euclidean_distance）
- v66.2（LLM Extraction）: 型安全 JSON 抽出ステージ
- v66.3（Rune.embed）: 統一埋め込みインターフェース（OpenAI / Cohere / ローカル）
- v66.4（Vector DB Runes）: Pinecone / pgvector / Weaviate / Qdrant
- v66.5（Rune.inference）: ストリーミング ML 推論（backpressure / SLA / stateful）
- v66.6（Rune.serve）: モデルサービングエンドポイント（rate limit / OpenAPI）
- v66.7（Rune.featurestore）: 型安全フィーチャーストア
- v66.8（AI Lint Rules）: W055〜W059 AI パイプラインアンチパターン検出スタブ
- v66.9（安定化）: ai-runes-overview.mdx / 全 AI Rune 存在確認

**テスト数**: 3497

---

## v66.0.0（2026-08-05）— Math & Science Foundation

> 「行列の次元は型で保証され、勾配は自動的に伝播する。
>  統計的検定は型安全に呼び出せ、時系列の周期は型パラメータに刻まれる。
>  数学的正確性が、AI パイプラインの信頼性を支える土台になった。
>
>  これが Favnir v66.0 — Math & Science Foundation の姿である。」

**Math & Science Foundation** の宣言バージョン。v65.1〜v65.9 で実装した
線形代数・統計・自動微分・最適化・数値計算・時系列・ML Primitives の 7 Rune 群と
Math Lint Rules W050〜W054 の統合を宣言した。

**v65.1〜v65.9 達成内容:**
- v65.1（Rune.linalg）: 線形代数（matmul / svd / eig / solve）
- v65.2（Rune.stats）: 統計解析（describe / t_test / linear_regression）
- v65.3（Rune.autodiff）: 自動微分（grad / jacobian / hessian / tape）
- v65.4（Rune.optim）: 最適化（adam / sgd / l_bfgs / scheduler）
- v65.5（Rune.numeric）: 数値計算（integrate / fft / ode_solve / bisection）
- v65.6（Rune.timeseries）: 時系列（arima / sarima / decompose / adf_test）
- v65.7（Rune.ml）: ML Primitives（knn / random_forest / cross_validate）
- v65.8（Math Lint Rules）: W050〜W054 静的解析ルール
- v65.9（安定化）: math-runes-overview.mdx / 全 Rune 存在確認

**テスト数**: 3475

---

## v65.0.0（2026-08-02）— Performance 1.0

> 「型安全なパイプラインがネイティブコードに変わる。
>  変更差分だけが再コンパイルされ、エラーはソースを指す。
>  ベンチマークは pandas を超え、flamegraph はボトルネックを露わにする。
>
>  Favnir は「型安全」と「高速」を両立したデータパイプライン言語になった。
>
>  これが Favnir v65.0 — Performance 1.0 の姿である。」

**Performance 1.0** の宣言バージョン。v64.1〜v64.9 で実装した全機能を統合し、
AOT ネイティブコンパイル・差分ビルド・flamegraph プロファイリング・外部ベンチマーク比較・
パフォーマンス lint・WASM ビルドの完成を宣言した。

**v64.1〜v64.9 達成内容:**
- v64.1（CI 統合）: `cmd_build_ci` / GitHub Actions テンプレート
- v64.2（リグレッション検出）: `BenchTomlConfig` / `[bench] regression_threshold_pct`
- v64.3（パフォーマンスガイド）: `site/content/docs/runtime/performance.mdx`
- v64.4（flamegraph AOT）: `cmd_profile_flamegraph_aot` / IR fns → SVG
- v64.5（外部ベンチ比較）: `site/content/docs/runtime/benchmarks.mdx` / `run_comparison.sh`
- v64.6（lint --perf）: `LintTomlConfig.perf` / `[lint] perf = true`
- v64.7（WASM ビルド）: `cmd_build_wasm` / `wasm_codegen_program`
- v64.8（総括記事）: `site/content/docs/performance/performance1-overview.mdx`
- v64.9（安定化）: `scale_all_v64_features_stable` / `performance1_overview_doc_complete`

**テスト数**: 3453

---

## v64.0.0（2026-08-02）— Incremental & Scale

> 「変更されたステージだけが再コンパイルされ、未使用のステージは除去される。
>  スレッドはコアの数だけ走り、キューはバックプレッシャーで制御される。
>  ベンチマークは数字で真実を語る。
>
>  Favnir は大規模 ETL を安心して任せられるエンジンになった。
>
>  これが Favnir v64.0 — Incremental & Scale の姿である。」

**Incremental & Scale** の宣言バージョン。v63.1〜v63.9 で実装した全機能を統合し、
差分コンパイル・DAG 最適化・並列実行・バックプレッシャー制御・ETL ベンチマークの完成を宣言した。

**v63.1〜v63.9 達成内容:**
- v63.1（差分キャッシュ）: `cmd_run_with_cache` / `IncrementalCache` / E0428
- v63.2（fav watch 改善）: ポーリング最適化・変更ファイルのみ再コンパイル
- v63.3（E0428）: キャッシュ無効化エラー
- v63.4（par 動的スレッドプール）: `cmd_parallel_stats` / `[parallel]` 設定
- v63.5（メモリプロファイリング）: `cmd_profile_memory`
- v63.6（バックプレッシャー・W041）: `BackpressureConfig` / `[backpressure]` / W041 lint
- v63.7（DAG 最適化）: `cmd_opt_stats` / dead stage elimination / pure stage fusion
- v63.8（ETL ベンチスイート）: `cmd_bench_suite` / "etl-standard" スイート
- v63.9（安定化）: `scale_e2e_incremental_par` / `scale_dag_opt_dead_and_fused`

**テスト数**: 3431

---

## v63.0.0（2026-08-02）— AOT Native

> 「パイプラインはネイティブバイナリにコンパイルされ、VM オーバーヘッドを超える速度で動く。
>  クロスコンパイルで ARM にも届き、Docker イメージは最小限のサイズに収まる。
>
>  Favnir は型安全なコンパイル言語として新たな段階に達した。
>
>  これが Favnir v63.0 — AOT Native の姿である。」

**AOT Native** の宣言バージョン。v62.1〜v62.9 で実装した AOT 全機能を統合し、
ネイティブバイナリ生成・クロスコンパイル・Docker イメージ化・AOT 互換性チェックの完成を宣言した。

**v62.1〜v62.9 達成内容:**
- v62.1（`fav build` 基盤）: `cmd_build_basic` / `cmd_build_link` / `cmd_build_docker` API 基盤
- v62.2（native binary 生成）: Cranelift AOT コンパイル → `.o` ファイル生成
- v62.3（クロスコンパイル）: x86_64 / aarch64 クロスコンパイル対応
- v62.4（Pure stage インライン化）: `analyze_for_inlining` / `is_aot_pure` による最適化
- v62.5（`fav bench`）: ステージ別ベンチマーク計測
- v62.6（Docker 出力）: `fav build --docker` OCI イメージ生成
- v62.7（`fav.toml [build]`）: `BuildConfig` / `ResolvedBuildConfig` AOT 設定
- v62.8（E0427）: AOT 未サポート機能検出バリデーター・エラーカタログ登録
- v62.9（E2E デモ）: `infra/e2e-demo/aot/` + `site/content/docs/runtime/aot.mdx`

**テスト数**: 3406

---

## v62.0.0（2026-08-01）— Language Polish

> 「パターンは OR で分岐し、as で束縛される。
>  レコードは `{ base | field: value }` で一部だけ書き換えられる。
>  型注釈に `_` を置けば推論が答えを返す。
>  エラーは期待値と実際値の差分を語り、修正の道筋を示す。
>
>  Favnir の型システムはデータエンジニアの思考を助ける存在になった。
>
>  これが Favnir v62.0 — Language Polish の姿である。」

**Language Polish** の宣言バージョン。v61.1〜v61.9 で実装した全 Language Polish 機能を統合し、
「型システムがデータエンジニアの思考を助ける」言語としての完成を宣言した。

**v61.1〜v61.9 達成内容:**
- v61.1（OR パターン強化）: 全アーム型チェック・W037 重複リテラル検出
- v61.2（as-pattern 拡張）: ネスト対応・LSP inlay hints・W039 束縛衝突警告
- v61.3（OR パターン個別ガード）: 各アームに独立ガード・E0395 非 Bool ガードエラー
- v61.4（record update 式）: `{ base | field: val }` 構文・E0396 型不一致・E0397 未定義フィールド
- v61.5（f-string 強化）: ネスト関数呼び出し・フィールドアクセス・マルチライン `f"""`
- v61.6（型エラー差分表示）: E0103 に構造的差分 hint・`fav explain` 拡充
- v61.7（`_` 型プレースホルダー）: `TypeExpr::Hole`・W040 lint・LSP inlay hints
- v61.8（`fav check --strict`）: `LintConfig`・W040 `[strict]` タグ・`[lint] strict = true`
- v61.9（安定化）: v61.1〜v61.8 全機能の統合テスト確認

**Tests: 3382 passed, 0 failed**

---

## v61.0.0（2026-07-31）— Developer Experience 2.0

> 「エラーはソース位置を指し、修正候補は即座に現れる。
>  エディタは意図を理解し、フォーマッタはコメントを守る。
>  REPL でパイプラインを対話的に探索でき、ドキュメントは自動生成される。
>
>  Favnir のエラーメッセージはデータエンジニアの道標になった。
>
>  これが Favnir v61.0 — Developer Experience 2.0 の姿である。」

**Developer Experience 2.0** の宣言バージョン。v60.1〜v60.9 で実装した全 DX 機能を統合し、
「エラーメッセージから修正まで一気通貫」の開発体験を確立した。

**v60.1〜v60.9 達成内容:**
- v60.1（エラー span 表示）: `-->` / `|` / `^` アンダーライン形式
- v60.2（`fav check --fix`）: 自動修正提案・dry-run モード
- v60.3（LSP Code Action）: `fav check --fix` と LSP の Code Action 統合
- v60.4（LSP Diagnostic 統合）: span 情報を LSP Diagnostic に反映
- v60.5（REPL 強化）: `:load` / `:debug` / マルチライン入力
- v60.6（`fav explain-error` 全コード）: `long_description` 全エラーコード補完
- v60.7（`fav fmt` 拡張）: コメント保持・`.favfmt` 設定ファイル対応
- v60.8（`fav doc` 強化）: HTML 出力・Rune ドキュメント統合・`@param`/`@returns` タグ
- v60.9（安定化）: v60.1〜v60.8 全機能の統合テスト確認

---

## v60.0.0（2026-07-30）— Enterprise 1.0

> 「ストリームはウィンドウで区切られ、型システムは制約で守られる。
>  アクセスはロールで制御され、シークレットはコードに現れない。
>  デプロイは無停止で切り替わり、ポリシーはコードで記述される。
>  コストは可視化され、SLA は保証され、コンプライアンスは証明される。
>
>  Favnir はデータエンジニアリングのエンタープライズ標準になった。
>
>  これが Favnir v60.0 — Enterprise 1.0 の姿である。」

**Enterprise 1.0** の宣言バージョン。v56〜v59 で実装した全エンタープライズ機能を統合し、
「企業で安心して選ばれるデータパイプライン言語」として完成した。

**v56〜v59 達成内容:**
- v57.1（RBAC）: `fav run --rbac`・ロールベースアクセス制御
- v57.2（Secret 管理）: AWS SM / Vault / GCP SM 統合
- v57.3（mTLS）: `fav run --mtls`・相互 TLS 接続
- v57.5（監査ログ）: 署名付き監査ログ
- v57.6（コンプライアンス）: GDPR / SOC2 / HIPAA レポート
- v58.1（Blue-Green Deploy）: `fav deploy --strategy blue-green`・無停止デプロイ
- v59.2（SLA Guarantee）: `fav sla report`・SLA 監視・アラート統合
- v59.3（Cost Visibility）: `fav cost estimate`・パイプラインコスト見積もり
- v59.5（Migration Toolkit）: `fav migrate`・v1 → Enterprise 自動移行
- v59.6（Enterprise Certify）: `fav certify --level enterprise`・Enterprise 1.0 認定

---

## v59.0.0（2026-07-29）— Governance & Deployment 2.0

> 「パイプラインは Blue/Green で無停止デプロイされ、
>  カナリアは段階的にトラフィックを引き受ける。
>  スキーマはバージョン管理され、データはカタログで検索できる。
>  ポリシーはコードで記述され、コンプライアンスは自動で証明される。
>  Favnir のパイプラインは運用チームに信頼される。
>
>  これが Favnir v59.0 — Governance & Deployment 2.0 の姿である。」

**Governance & Deployment 2.0** の宣言バージョン。v58.1〜v58.9 の全機能統合を経て、
Blue/Green デプロイ・カナリア・HA・Schema Migration・Data Catalog・Policy-as-Code・
マルチ環境設定の成熟を宣言する。

**v58.1〜v58.9 達成内容:**
- v58.1（Blue/Green デプロイ）: `fav deploy --strategy blue-green`・無停止デプロイ
- v58.2（カナリアデプロイ）: `--canary-weight` によるトラフィック段階移行
- v58.3（Schema Migration）: `fav schema migrate`・バージョン管理マイグレーション
- v58.4（Data Catalog）: `fav catalog push/search`・データアセット登録・検索
- v58.5（Policy-as-Code・E0426）: `fav policy check`・コンプライアンス自動証明
- v58.6（マルチ環境設定）: `--env dev/staging/prod`・`fav.toml [env]` セクション
- v58.7（HA / DR）: `fav run --ha --replica <n>`・フェイルオーバー・`/healthz`
- v58.8（ドキュメントサイト記事）: deployment.mdx / governance.mdx / multi-env-pipeline.mdx 作成
- v58.9（安定化・コードフリーズ）: governance-overview.mdx・全テスト通過確認

---

## v58.0.0（2026-07-28）— Enterprise Security

> 「アクセスはロールで制御され、シークレットはコードに現れず、
>  通信は mTLS で守られ、監査ログは改ざんできない。
>  コンプライアンスレポートはボタン一つで生成される。
>  Favnir は企業のセキュリティ要件を満たす言語になった。
>
>  これが Favnir v58.0 — Enterprise Security の姿である。」

**Enterprise Security** の宣言バージョン。v57.1〜v57.9 の全機能統合を経て、
RBAC・シークレット管理・TLS/mTLS・監査ログ署名・コンプライアンスレポート・
マルチテナント分離の成熟を宣言する。

**v57.1〜v57.9 達成内容:**
- v57.1（RBAC）: ロールベースアクセス制御・E0424 エラーコード
- v57.2（Secrets 管理）: AWS SM / Vault 連携・実行時シークレット注入
- v57.3（TLS / mTLS）: HTTP / gRPC Rune 証明書設定・`is_mtls()` メソッド
- v57.4（依存関係スキャン）: CVE スキャン・`--fail-on-high`
- v57.5（監査ログ署名）: HMAC-SHA256 署名・tamper-proof audit
- v57.6（コンプライアンスレポート）: GDPR / SOC2 フレームワーク対応
- v57.7（マルチテナント分離）: `TenancyConfig` / strict モード・`is_strict()`
- v57.8（ドキュメント）: Enterprise Security 記事群（rbac / secrets / compliance）
- v57.9（安定化）: コードフリーズ・enterprise-security-overview.mdx

---

## v57.0.0（2026-07-26）— Language Power 2.0

> 「ジェネリクスは制約で安全に縛られ、レコードは行変数で柔軟に合成され、
>  エフェクトは推論によって自然に表れる。
>  パターンはガード節と OR 構文で表現力を増し、モジュールは名前空間で整理される。
>  Favnir の型システムは開発者の意図を正確に表現できる。
>
>  これが Favnir v57.0 — Language Power 2.0 の姿である。」

**Language Power 2.0** の宣言バージョン。v56.1〜v56.9 の全機能統合を経て、
境界付きジェネリクス・行多相レコード・エフェクト推論 LSP・OR パターン・
as-パターン・モジュール名前空間の成熟を宣言する。

**v56.1〜v56.9 達成内容:**
- v56.1（境界付きジェネリクス本番品質化）: `where T: Interface` 正式化・E0422
- v56.2（複数 constraint・coherence 強化）: `T with Ord with Serialize`・E0423
- v56.3（行多相レコード活用拡張）: `{ field: Type | r }` 行変数明示・LSP ホバー
- v56.4（エフェクト推論 LSP 統合）: inlay hints・`fav check --show-types`
- v56.5（OR パターン + パターンガード強化）: `Ok(x) | Err(x)`・W037
- v56.6（パターンエイリアス）: `head @ { id, amount }` as-パターン
- v56.7（モジュール名前空間）: `import "path" as alias.*`・W038
- v56.8（ドキュメント）: bounded-generics / row-polymorphism / effect-inference MDX
- v56.9（安定化）: language-power2-overview.mdx 骨子・コードフリーズ

---

## v56.0.0（2026-07-24）— Streaming Native 2.0

> 「ウィンドウはイベントを時間で区切り、ウォーターマークは遅延を許容し、
>  チェックポイントは障害から瞬時に回復する。
>  CEP はイベントの流れからパターンを検出する。
>  Favnir はリアルタイムデータの言語になった。
>
>  これが Favnir v56.0 — Streaming Native 2.0 の姿である。」

**Streaming Native 2.0** の宣言バージョン。v55.1〜v55.9 の全機能統合を経て、
ウィンドウ・ウォーターマーク・Exactly-once チェックポイント・CEP・Stateful stage・
Checkpoint / Replay API の成熟を宣言する。

**v55.1〜v55.9 達成内容:**
- v55.1（ウィンドウ + Exactly-once 統合）: Tumbling/Sliding ウィンドウとチェックポイント統合
- v55.2（セッションウィンドウ + ウォーターマーク）: 遅延イベント許容・`--stream-stats`
- v55.3（Exactly-once チェックポイント）: 冪等リトライ・処理済みオフセット永続化
- v55.4（ストリーム結合）: `Stream.join_inner` / `Stream.join_left`
- v55.5（Stateful stage）: `State.get` / `State.set` / `State.get_or_default`
- v55.6（CEP 統合）: `CEP.sequence` / `CEP.skip_until`
- v55.7（Checkpoint / Replay API）: `set/get/clear_resume_from_checkpoint`
- v55.8（ドキュメント）: streaming-v2 / stateful-pipeline / cep-patterns MDX
- v55.9（安定化）: streaming-native2-overview.mdx 骨子・コードフリーズ

---

## v55.0.0（2026-07-23）— Production 3.0

> 「型安全なガード節、スケールする並列パイプライン、
>  保証されたデータ品質、そして考えを助ける開発体験。
>  Favnir はデータエンジニアが現場で選ぶ言語になった。
>
>  これが Favnir v55.0 — Production 3.0 の姿である。」

**Production 3.0** の宣言バージョン。v54.1〜v54.9 の最終整備を経て、
v51〜v54 で積み上げた全機能（DX 3.0 / Performance & Scale / Data Quality 2.0 / Integration Sprint）を
統合・安定化する。

**v51〜v54 達成内容:**
- v51（DX 3.0）: 全エラーコード診断・LSP インレイヒント・trace/watch
- v52（Performance & Scale）: par 並列実行・バックプレッシャー・bench 回帰検出・WASM 最適化
- v53（Data Quality 2.0）: assert_schema・lineage 強化・audit-log・OTel 強化
- v54（Integration Sprint）: fav explain 全コード・watch-diff・CI 統合・dq-report・doctor

---

## v54.0.0（2026-07-22）— Integration Sprint

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。
>
>  これが Favnir v54.0 — Integration の姿である。」

**Integration Sprint** の宣言バージョン。v53.1〜v53.8 の統合作業（lineage × LSP・par bench・assert_schema 詳細診断・
E2E デモ・cookbook・用語集・CHANGELOG/MILESTONE 整理・integration-overview ドキュメント）および
v53.9 のコードフリーズを経て、v51.0〜v53.0 の 3 マイルストーンを一体として機能させた。

---

## v51.0〜v53.0 Integration Sprint サマリー（2026-07-22）

> 「エディタはデータの来歴を示し、並列パイプラインの性能は
>  計測可能で、スキーマ違反は即座に修正できる。
>  Favnir の 3 つの柱が一体となった。」

DX 3.0（v51）・Performance & Scale（v52）・Data Quality & Observability 2.0（v53）の 3 マイルストーンを
**Integration Sprint** として統合。v53.1〜v53.8 の統合作業・v53.9 のコードフリーズを完了し、
v54.0「Integration Sprint 宣言」を達成した。

---

## v53.0.0（2026-07-22）— Data Quality & Observability 2.0

> 「スキーマはランタイムで検証され、データの来歴はグラフで見え、
>  SLA 違反は即座に検知され、アクセスはすべて記録される。
>  Favnir のパイプラインは信頼できるデータを届ける。
>
>  これが Favnir v53.0 — Data Quality & Observability 2.0 の姿である。」

**Data Quality & Observability 2.0** の宣言バージョン。v52.1〜v52.9 の全機能統合を経て、
assert_schema・リネージ可視化・SLA 監視・audit-log・OTel 強化の成熟を宣言する。

---

## v52.0.0（2026-07-20）— Performance & Scale

> 「並列パイプラインはコアを使い切り、バックプレッシャーは
>  データの氾濫を防ぎ、ベンチマークは退行を即座に検出する。
>  Favnir は大規模データに立ち向かえる言語になった。
>
>  これが Favnir v52.0 — Performance & Scale の姿である。」

**Performance & Scale** の宣言バージョン。v51.1〜v51.9 の全機能統合を経て、
並列実行・バックプレッシャー・ベンチマーク回帰検出・WASM 最適化の成熟を宣言する。

---

## v51.0.0（2026-07-19）— Developer Experience 3.0

> 「全エラーコードに修正提案が付き、JSON / LSP / CLI で一貫して届く。
>  エディタは型を表示し、trace はパイプラインの流れを可視化する。
>  Favnir の診断は開発者の思考を止めない。
>
>  これが Favnir v51.0 — Developer Experience 3.0 の姿である。」

**Developer Experience 3.0** の宣言バージョン。v50.1〜v50.9 の全機能統合を経て、
診断・エディタ統合・デバッグ体験の成熟を宣言する。

---

## v50.0.0（2026-07-18）— Language Maturity / Production 2.0

> 「`return` による安全なガード節、成熟した標準ライブラリ、
>  明確なモジュールシステム、インラインテストが揃い、
>  Favnir は迷わず使える実用言語になった。
>
>  これが Favnir v50.0 — Production 2.0 の姿である。」

**Language Maturity** の宣言バージョン。v46〜v49 の全機能統合・安定化・セキュリティ審査を経て、
データエンジニアが迷わず使える実用言語としての成熟を宣言する。

---

## v49.0.0 — Module & Package 2.0（2026-07-18）

> 「パッケージ import とローカル import が構文で明確に分離され、
>  `fav.toml` が依存関係の唯一の真実となった。
>
>  これが Favnir v49.0 — Module & Package 2.0 の姿である。」

v49.0.0 をもって、Favnir の **Module & Package 2.0** を正式に宣言する。

### 達成コンポーネント（v48.1〜v48.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `ImportKind::Package` / `ImportKind::Local` AST + parser | v48.1.0 | パッケージ import 構文刷新 |
| ローカル import `"./"` プレフィックス対応 | v48.2.0 | ローカルファイル import 構文 |
| `fav.toml [runes]` 解決ロジック + E0417 | v48.3.0 | 依存関係の一元管理 |
| `fav install`（`runes/` スタブ展開）| v48.4.0 | パッケージインストールコマンド |
| W035 `legacy_import_rune` lint ルール + E0417 実発行 | v48.5.0 | 旧構文の非推奨化 + E0417 実発行 |
| 循環 import 検出 + E0418 | v48.6.0 | import グラフ循環検出 |
| `rune.toml` 標準化（`validate_rune_toml`）| v48.7.0 | Rune 仕様の統一 |
| `list_installed_runes` / `get_rune_version` ヘルパー | v48.8.0 | runes/ ディレクトリ管理 |
| Module ドキュメント + migration guide | v48.9.0 | ユーザー向け移行ガイド |

---

## v48.0.0 — Standard Library 2.0（2026-07-18）

> 「List・String・Float・Option・Result・Map の主要操作が揃い、
>  外部ライブラリなしに実務的なデータ変換が書ける。
>
>  これが Favnir v48.0 — Standard Library 2.0 の姿である。」

v48.0.0 をもって、Favnir の **Standard Library 2.0** を正式に宣言する。

### 達成コンポーネント（v47.1〜v47.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `List.zip` / `List.chunk` | v47.1.0 | 2 リストのペア化・n 要素分割 |
| `List.flat_map` / `List.group_by` / `List.dedupe` | v47.2.0 | flatten+map・グループ化・重複除去 |
| `List.scan` / `List.take_while` / `List.drop_while` | v47.3.0 | 累積値リスト・先頭条件フィルタ |
| `String.pad_left` / `String.trim_start` / `String.repeat` | v47.4.0 | パディング・トリム・繰り返し |
| `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` | v47.5.0 | 浮動小数点・整数拡張 |
| `Option.map` / `Option.unwrap_or` / `Option.and_then` / `Option.is_some` / `Option.is_none` | v47.6.0 | Option コンビネータ |
| `Result.map` / `Result.map_err` / `Result.and_then` / `Result.is_ok` / `Result.is_err` | v47.7.0 | Result コンビネータ |
| `Map.merge` / `Map.filter_values` / `Map.map_values` / `Map.keys` / `Map.values` | v47.8.0 | Map 拡充 |
| stdlib ドキュメント（`float.mdx` / `v2.mdx` / 各 MDX 更新） | v47.9.0 | Standard Library 2.0 全関数索引 |

---

## v47.0.0 — Developer Experience（2026-07-17）

> 「インラインテスト・LSP クイックフィックス・型情報可視化が揃い、
>  Favnir の開発体験が実用水準に達した。
>
>  これが Favnir v47.0 — Developer Experience の姿である。」

v47.0.0 をもって、Favnir の **Developer Experience** を正式に宣言する。

### 達成コンポーネント（v46.1〜v46.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `#[test]` ブロック AST + parser | v46.1 | `FnDef.is_test = true`、`#[test] fn` 解析 |
| `fav test` コマンド実装 | v46.2 | `cmd_test`、`#[test]` 収集と VM 実行ループ |
| assertion 拡充 | v46.3 | `assert_ok` / `assert_err` / `assert_ne` VM primitive |
| LSP inlay hints 強化 | v46.4 | `textDocument/inlayHint`、パイプライン推論型表示 |
| LSP クイックフィックス強化 | v46.5 | E0102 did-you-mean / E0101 引数追加提案 |
| `fav explain` 2.0 Phase 1 | v46.6 | dead path（点線）/ error path（赤）Mermaid 可視化 |
| `fav explain --lineage` 2.0 | v46.7 | `is_dead` フラグ + `--show-dead` CLI |
| `fav explain --types` | v46.8 | ステージ宣言型一覧表示 |
| DX ドキュメント + v47.0 前調整 | v46.9 | `fav-test.mdx` / `developer-experience.mdx` |

---

## v46.0.0 — Language Refinement（2026-07-16）

> 「`return` によるガード節・`match` 完全網羅・型エイリアスの明確な境界・
>  改善されたエラーメッセージが揃い、Favnir の構文が成熟した。
>
>  これが Favnir v46.0 — Language Refinement の姿である。」

v46.0.0 をもって、Favnir の **Language Refinement** を正式に宣言する。

### 達成コンポーネント（v45.1〜v45.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `return` 構文 AST + parser | v45.1 | ReturnStmt ノード・parser 解析 |
| `return` 型チェック + E0415 | v45.2 | 戻り型不一致エラー |
| `return` compiler + VM | v45.3 | Return opcode・早期脱出実行 |
| `match` 網羅性 + W034/E0416 | v45.4 | 非網羅 match の警告・エラー |
| 型エイリアス完全化 | v45.5 | 透過的互換性・opaque 非互換性 |
| エラーメッセージ改善 Phase 1 | v45.6 | E0101〜E0200 suggestion 追加 |
| エラーメッセージ改善 Phase 2 + 数値リテラル `_` | v45.7 | E0201〜E0413 suggestion・`1_000_000` |
| examples 更新 Phase 1 | v45.8 | !Effect 除去確認・return ガード節 |
| examples 更新 Phase 2 + v46.0 前調整 | v45.9 | stage_seq_demo 修正・overview 作成 |

---

## v45.0.0 — Precision & Flow（2026-07-15）

> 「型推論がジェネリクスと戻り値型を補完し、最小限の注釈で安全なコードが書ける。
>  ウィンドウ集計・CEP・Stream join が型安全に記述でき、
>  refinement type と opaque type がデータの意味を型で守る。
>
>  これが Favnir v45.0 — Precision & Flow の姿である。」

v45.0.0 をもって、Favnir の **Precision & Flow** を正式に宣言する。

### 達成コンポーネント（v44.1〜v44.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Refinement type × Streaming 統合 | v44.1 | collect_refinement_stream_bindings |
| CEP × Refinement type | v44.2 | collect_cep_refinement_event_refs |
| Stream join × Opaque type | v44.3 | collect_opaque_alias_groups |
| 型推論 × パイプライン lineage | v44.4 | collect_annotated_lineage_bindings |
| Back-pressure × fav policy 統合 | v44.5 | collect_stage_max_inflight_annotations |
| Precision & Flow E2E デモ | v44.6 | infra/e2e-demo/precision-flow/ |
| ドキュメントサイト概要ページ | v44.7 | precision-and-flow.mdx |
| パフォーマンス最終調整 | v44.8 | collect_bench_stream_notes + CHANGELOG |
| v45.0 前調整・安定化 | v44.9 | precision-and-flow-overview.mdx |

**宣言日**: 2026-07-15

---

## v44.0.0 — Language Expressiveness（2026-07-13）

> 「戻り値型は省略でき、ジェネリクスは呼び出し側から推論される。
>  ラムダ引数はパイプライン上流の型から確定し、
>  `opaque type` で型の境界を守れる。
>
>  これが Favnir v44.0 — Language Expressiveness の姿である。」

v44.0.0 をもって、Favnir の **Language Expressiveness** を正式に宣言する。

### 達成コンポーネント（v43.1〜v43.13）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 戻り値型推論 | v43.1 | Return type omission |
| fav check 統合・E0410/E0411 | v43.2 | 推論失敗エラー |
| ジェネリック型引数推論 | v43.3 | Call-site generic inference |
| E0412 曖昧型変数検出 | v43.4 | Ambiguous type variable |
| ラムダ引数型推論 | v43.5 | Contextual lambda inference |
| パイプライン型伝播 | v43.6 | Pipeline stage typing |
| 構造体リテラル推論 | v43.7 | Structural inference |
| 双方向型推論 | v43.8 | Bidirectional / top-down |
| fav check --show-inference | v43.9 | 推論型の注釈表示 |
| fav check --explain 統合 | v43.10 | 静的解説テキスト |
| opaque type 完全化 | v43.11 | opaque keyword + E0413 |
| W031/W032 lint | v43.12 | 冗長型注釈の警告 |
| Language Expressiveness cookbook | v43.13 | ドキュメント安定化 |

**宣言日**: 2026-07-13

---

## v43.0.0 — Real-Time Power（2026-07-12）

> 「CEP で `seq(Login, Purchase) within 300` が型安全に書ける。
>  Stream join で 2 ストリームを time-window で結合できる。
>  `#[max_inflight]` で Back-pressure を宣言的に制御できる。
>
>  これが Favnir v43.0 — Real-Time Power の姿である。」

v43.0.0 をもって、Favnir の **Real-Time Power** を正式に宣言する。

### 達成コンポーネント（v42.1〜v42.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| CEP DSL 基盤 | v42.1 | `cep pattern` / `within` 構文 |
| CEP パターン: `seq` / `any` / `not` | v42.2 | 3 パターンコンビネータ |
| CEP checker.fav 統合 | v42.3 | `within >= 1` 検証・E0420 |
| Stream join（time-window） | v42.4 | `Stream.join` 2 ストリーム結合 |
| Back-pressure `#[max_inflight]` | v42.5 | parser + AST 宣言 |
| WebSocket Rune | v42.6 | `WebSocket.send` / `WebSocket.broadcast` |
| `fav monitor` | v42.7 | パイプライン監視コマンド stub |
| Real-Time Power cookbook | v42.8 | `cep-login-purchase.mdx` / `stream-join.mdx` |
| v43.0 前調整・安定化 | v42.9 | `real-time-power.mdx` 新規作成 |

**宣言日**: 2026-07-12

---

## v42.0.0 — Type Precision（2026-07-12）

> 「`type Age = Int where (>= 0)` で値の意味を型に刻める。
>  タプルパターンとガード付き match でより精緻な分岐が書ける。
>  Newtype は内側の型の演算を自動継承する。
>
>  これが Favnir v42.0 — Type Precision の姿である。」

v42.0.0 をもって、Favnir の **Type Precision** を正式に宣言する。

### 達成コンポーネント（v41.1〜v41.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Refinement type alias | v41.1 | `type Age = Int where \|v\| v >= 0` |
| Refinement invariant + E0404〜E0406 | v41.2 | fav check 統合 |
| タプルパターン match | v41.3 | `match (status, count) { ... }` |
| ガード付き match | v41.4 | `n if n >= 90 => "A"` |
| Row polymorphism | v41.5 | record spread `{ ..u, active: true }` |
| Newtype 自動 impl | v41.6 | `type Kg(Float)` — 算術演算子自動委譲 |
| W030 lint | v41.7 | 冗長 refinement ガード検出 |
| Type Precision cookbook + docs | v41.8 | refinement-types.mdx 整備 |
| v42.0 前調整・安定化 | v41.9 | type-precision.mdx 新規作成 |

**宣言日**: 2026-07-12

---

## v41.0.0 — Streaming Foundations（2026-07-11）

> 「`tumbling_window` / `sliding_window` / `session_window` でウィンドウ集計を型安全に書ける。
>  `Event<T>` の timestamp と Watermark で out-of-order イベントを制御できる。
>
>  これが Favnir v41.0 — Streaming Foundations の姿である。」

v41.0.0 をもって、Favnir の **Streaming Foundations** を正式に宣言する。

### 達成コンポーネント（v40.1〜v40.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| tumbling_window / sliding_window | v40.1 | 固定幅・スライドウィンドウ |
| session_window | v40.2 | セッションウィンドウ |
| Event\<T\> + timestamp | v40.3 | イベント型に時刻基準フィールド追加 |
| Out-of-order 処理 | v40.4 | late_tolerance / drop / reprocess |
| fav.toml \[stream\] | v40.5 | プロジェクト設定でストリーム設定管理 |
| Kafka / Redis Streams 対応 | v40.6 | consume_windowed 追加 |
| fav bench --stream | v40.7 | ストリームパイプライン計測スタブ |
| Streaming cookbook | v40.8 | window-aggregation / kafka-streaming MDX |
| 安定化 | v40.9 | streaming-foundations.mdx ドキュメント整備 |

**宣言日**: 2026-07-11

---

## v40.0.0 — Enterprise Governance（2026-07-11）

> 「RBAC で実行権限を制御し、Audit Log でパイプラインを追跡できる。
>  `fav policy` で組織ポリシーを宣言的に定義し、
>  `fav policy check --ci` で違反を PR でブロックできる。
>  Secret Rune は Vault / AWS / GCP に対応し、
>  マルチテナント対応で複数チームが安全に使える。
>
>  これが Favnir v40.0 — Enterprise Governance の姿である。」

v40.0.0 をもって、Favnir の **Enterprise Governance** を正式に宣言する。

### 達成コンポーネント（v39.1〜v39.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| RBAC Rune | v39.1 | require_role / check_permission / verify_jwt |
| Audit Log Rune | v39.2 | Audit.log / start_trace / end_trace |
| fav policy | v39.3 | fav policy check / fav policy check --ci（exit 1） |
| Secret Rune 強化 | v39.4 | get_aws / get_vault / get_gcp / get_env |
| マルチテナント | v39.5 | tenant.db_schema / s3_prefix / validate_tenant |
| fav audit | v39.6 | ライセンス一覧 / GPL・CVE 検出 |
| CI/CD ゲート | v39.7 | fav ci init に Policy check ステップ自動含める |
| Governance docs | v39.8 | docs/governance/ 3 件 + cookbook 3 件 |
| 安定化 | v39.9 | enterprise-governance.mdx ドキュメント整備 |

**宣言日**: 2026-07-11

---

## v39.0.0 — Intelligence & Assistance（2026-07-10）

> 「`fav suggest` でエラーから修正案を AI が提案し、
>  `fav generate --from sql` でパイプラインを自動生成し、
>  `fav explain --verbose` でコンテキスト付き解説を受け取れる。
>  Llm Rune はストリーミング・function calling・Embeddings に対応し、
>  RAG パイプラインを `fav new --template rag-pipeline` で即座に生成できる。
>
>  これが Favnir v39.0 — Intelligence & Assistance の姿である。」

v39.0.0 をもって、Favnir の **Intelligence & Assistance** を正式に宣言する。

### 達成コンポーネント（v38.1〜v38.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| fav suggest | v38.1 | エラーコードから修正案を LLM で生成 |
| fav generate --from sql | v38.2 | SQL → Favnir パイプライン自動変換 |
| fav generate --from csv 強化 | v38.3 | schema + expect ブロック出力 |
| LSP AI 補完 | v38.4 | [lsp.ai] enabled = true で LLM rerank |
| fav explain --verbose | v38.5 | コンテキスト付き LLM 解説・修正例 |
| RAG テンプレート | v38.6 | fav new --template rag-pipeline |
| Llm Rune 強化 | v38.7 | stream / function_call / embed 対応 |
| AI 支援 cookbook | v38.8 | sql-to-favnir / rag-pipeline / llm-streaming |
| 安定化 | v38.9 | ai-overview.mdx ドキュメント整備 |

**宣言日**: 2026-07-10

---

## v38.0.0 — Multi-Source ETL Power（2026-07-10）

> 「`List.join_on` で 2 つのリストを型安全に結合し、
>  `List.fan_out` / `List.fan_in` で大規模データを並列処理し、
>  CDC Rune で Debezium イベントをストリーミング処理できる。
>  `fav explain --lineage` でデータフローを DOT/SVG グラフとして可視化し、
>  `fav new --template multi-source` でマルチソース ETL プロジェクトを即座に生成できる。
>
>  これが Favnir v38.0 — Multi-Source ETL Power の姿である。」

v38.0.0 をもって、Favnir の **Multi-Source ETL Power** を正式に宣言する。

### 達成コンポーネント（v37.1〜v37.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス | v37.1 | `T with Serialize/Deserialize` 制約 |
| 行多相実用強化 | v37.2 | ネスト行型 `R with { addr: { city: String, .. } }` |
| List.join_on | v37.3 | left semi-join VM ビルトイン |
| List.fan_out / fan_in | v37.4 | チャンク分散・再集約 VM ビルトイン |
| CDC Rune | v37.5 | Debezium JSON イベント処理 |
| lineage DOT/SVG | v37.6 | `fav explain --lineage --format dot/svg` |
| multi-source テンプレート | v37.7 | `fav new --template multi-source` |
| cookbook 5 本 | v37.8 | join / CDC / fan-out / generics / lineage レシピ |
| 安定化 | v37.9 | lineage サマリー行・Multi-Source ETL ドキュメント |

**宣言日**: 2026-07-10

---

## v37.0.0 — Data Quality First（2026-07-09）

> 「`schema` でテーブル/列の型と制約を宣言し、
>  `expect` でビジネスルールをパイプラインに埋め込み、
>  `fav validate` でデータを検証できる。
>  スキーマ不整合は W025 lint で静的に検出され、
>  違反は E0380〜E0384 として報告される。
>  `fav schema diff` で変更の後方互換性を即座に把握できる。
>
>  これが Favnir v37.0 — Data Quality First の姿である。」

v37.0.0 をもって、Favnir の **Data Quality First** を正式に宣言する。

### 達成コンポーネント（v36.1〜v36.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| schema 定義構文 | v36.1 | `schema Orders { id: Int, ... }` インライン定義 |
| expect ブロック | v36.2 | `expect rows { not_empty, all(...) }` ビジネスルール宣言 |
| W025 lint | v36.3 | `schema_mismatch` — 静的フィールドアクセス検証 |
| fav validate | v36.4 | `fav validate --schema orders.fav data.csv` |
| Data Contract | v36.5 | `contracts/` 規約 + `fav contract check` |
| E0380〜E0384 | v36.6 | スキーマ不整合エラーカタログ |
| GE エクスポート | v36.7 | `--export ge` — Great Expectations 互換出力 |
| fav schema diff | v36.8 | フィールドレベル差分・後方互換性チェック |
| 安定化 | v36.9 | W025↔E0380 連携・validate サマリー・docs 統合 |

**宣言日**: 2026-07-09
**宣言バージョン**: v37.0.0

---

## v36.0.0 — Deployment Story（2026-07-08）

> 「`fav deploy --target lambda` で Lambda に自動デプロイし、
>  `fav deploy --target docker` で Docker イメージを生成し、
>  `fav ci init` で GitHub Actions CI を自動設定できる。
>  `!Effect` 廃止（v35.4〜v35.8）により、すべての API が ctx: AppCtx ベースに統一された。
>
>  これが Favnir v36.0 — Deployment Story の姿である。」

v36.0.0 をもって、Favnir の **Deployment Story** を正式に宣言する。

**宣言日**: 2026-07-08
**宣言バージョン**: v36.0.0

---

## v35.0.0 — Production Ready（2026-07-04）

> 「`fav new --template postgres-etl my-pipeline` で始め、
>  `fav check` で型安全性を確認し、
>  `fav build --target native` でネイティブバイナリを生成し、
>  Lambda にデプロイして実データを処理できる。
>  エラーが起きれば `fav explain` で原因がわかり、
>  `fav test --watch` でリグレッションを防げる。
>
>  これが Favnir v35.0 — Production Ready の姿である。」

v35.0.0 をもって、Favnir の **Production Ready** を正式に宣言する。

実案件デモ / ドキュメントサイト v4 / ベンチマーク公開 / セキュリティ審査 v2 /
エフェクトシステム統一（`!Effect` → ctx）/ 移行ツール整備が v34.x シリーズで完成した。

### 達成コンポーネント（v34.1〜v34.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 実案件デモ | v34.1 | `examples/real-world-etl/`（8 ファイル・5 ステージ）|
| ドキュメントサイト v4 | v34.2 | `/errors/` + cookbook 50 本 + ベンチマーク比較 |
| ベンチマーク公開 | v34.3 | Python pandas / Apache Spark 実測比較 |
| セキュリティ審査 v2 | v34.4 | W021・認証情報・sandbox・OSS ライセンス確認 |
| !Effect 廃止宣言 | v34.5 | W022 / `migration-effects.mdx` / IoCtx |
| ctx Rune 移行 | v34.6 | db / http / stream / io ctx Rune ファイル |
| ドキュメント ctx 移行 | v34.7 | `ctx-syntax-guide.mdx` / `getting-started.mdx` |
| 移行ツール | v34.8 | `MIGRATION.md` / `fav upgrade --from-effects` |
| 移行ドキュメント完全化 | v34.9 | `upgrade-guide.mdx` / ctx_migration フィクスチャ |

**宣言日**: 2026-07-04
**宣言バージョン**: v35.0.0

---

## v34.0.0 — Performance & Tooling（2026-07-04）

> 「`fav build --target native` でネイティブバイナリが生成でき、
>  10GB CSV を定常メモリで処理でき、
>  Lambda コールドスタートが 100ms 以下になること」
> = Performance & Tooling の完成を象徴する定義

v34.0.0 をもって、Favnir の **Performance & Tooling** を正式に宣言する。

AOT ネイティブバイナリ（Cranelift）/ インクリメンタルコンパイル / ストリーミング評価 /
Arrow 列指向統合 / precompiled 起動 / WASM 最適化 / エフェクトシステム移行準備 /
プロファイリング強化 / 並列コンパイルが v33.x シリーズで確認・記録された。

### 達成コンポーネント（v33.1〜v33.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| AOT ネイティブバイナリ | v33.1 | `fav build --target native` / Cranelift バックエンド |
| インクリメンタルコンパイル | v33.2 | `~/.fav/cache/` / SHA256 ハッシュキャッシュ |
| ストリーミング評価 | v33.3 | `#[streaming(chunk_size)]` / 定常メモリ処理 |
| Arrow 列指向統合 | v33.4 | `ArrowBatch` 型 / Parquet ゼロコピー書き込み |
| precompiled 起動 | v33.5 | `fav run --precompiled` / `.favc` アーティファクト |
| WASM 最適化 | v33.6 | DCE / wasm-opt 統合 / `WasmBuildConfig` |
| エフェクトシステム移行準備 | v33.7 | `migrate_effects_in_source` / `resolve_use_effects` |
| プロファイリング強化 | v33.8 | `parse_profile_json` / `to_folded_stacks` |
| 並列コンパイル | v33.9 | `compile_parallel` / `topo_layers` 循環依存検出 |

**宣言日**: 2026-07-04
**宣言バージョン**: v34.0.0

---

## v33.0.0 — Language Power（2026-07-03）

> 「Favnir の型システムを使って、DB スキーマから型を自動生成し、
>  汎用的なレコード変換関数を型安全に書き、
>  コンパイル時に前提条件を保証できること」
> = Language Power の完成を象徴する定義

v33.0.0 をもって、Favnir の **Language Power** を正式に宣言する。

境界付きジェネリクス（`T with Ord`）と行多相（`R with { id: Int }`）により汎用的なレコード変換関数が
型安全に書けるようになった。`where { b != 0 }` で関数引数の前提条件をコンパイル時に保証し、
`type User = schema "postgres:users"` でスキーマから型を自動生成できる。
線形型（E0332/E0333）・分散アノテーション（E0334）・定数ジェネリクス（E0335）が加わり、
型システムが実用的なデータパイプライン設計に耐える水準に達した。

### 達成コンポーネント（v32.1〜v32.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス | v32.1 | `T with Ord` / E0325 制約チェック |
| 行多相 | v32.2 | `R with { id: Int }` / E0337 フィールド不足 |
| where 制約 | v32.3 | `fn f(x: Int where { x > 0 })` / E0331 |
| スキーマ型 | v32.4 | `schema "postgres:users"` パース |
| 線形型 | v32.5 | E0332（二重使用）/ E0333（未使用）|
| 分散アノテーション | v32.6 | `<+T>` / `<-T>` / E0334 |
| 定数ジェネリクス | v32.7 | `<const N: Int where { N > 0 }>` / E0335 |
| 型駆動 API 生成 | v32.8 | `#[api]` / OpenAPI JSON / ルートテーブル |
| エフェクト推論 | v32.9 | `infer_effects_fn` / 推移的推論 |

**宣言日**: 2026-07-03
**宣言バージョン**: v33.0.0

---

## v32.0.0 — Language Polish（2026-07-03）

> 「Favnir を初めて使うデータエンジニアが、エラーメッセージを見て
>  自力でコードを修正し、30 分以内に最初のパイプラインを動かせること」
> = Language Polish の完成を象徴する定義

v32.0.0 をもって、Favnir の **Language Polish** を正式に宣言する。

エラーメッセージが rustc スタイル（`-->` ファイル位置 + `|` ソース行 + `= ヒント:`）に刷新され、
typo 候補（Levenshtein ≤ 2）と全エラーコード URL が付与された。
`fav explain E0001` でエラーの説明・修正例がターミナルで確認できる。
REPL は `:doc` / `:load` / `:history` / `:save` コマンドとタブ補完を備え、
データ探索ツールとして実用レベルに達した。
LSP Inlay Hints により `bind` 変数の型推論結果がエディタでインライン表示される。
`fav test --watch` と `fav check --all` / `fav scaffold` が揃い、
「書いていて気持ちいい」開発体験を達成した。

### 達成コンポーネント（v31.1〜v31.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| エラーメッセージ v2 | v31.1 | rustc スタイル・E0001〜E0021 全件 hint: 付与 |
| typo 候補 + URL | v31.2 | Levenshtein ≤ 2 候補提示・全エラーコード URL |
| fav explain | v31.3 | `fav explain E0001〜E0021` 説明・修正例出力 |
| REPL 品質向上 | v31.4 | :doc / :load / :history / :save / タブ補完 |
| LSP Inlay Hints | v31.5 | bind 変数の型推論結果インライン表示 |
| fav test --watch | v31.6 | ファイル変更で自動テスト再実行 |
| fav check --all | v31.7 | プロジェクト全体クロスファイルチェック |
| fav scaffold | v31.8 | stage / seq スタブを既存プロジェクトに追記 |
| ドッグフード修正 vol.2 | v31.9 | REPL 空行スキップ / check --all 空ディレクトリ警告 |

**宣言日**: 2026-07-03
**宣言バージョン**: v32.0.0

---

## v31.0.0 — Real-World Readiness（2026-07-02）

> 「`fav new --template postgres-etl my-project` で生成されたプロジェクトが、
>  `fav check` / `fav run` / `fav test` すべてで通り、
>  実データ（CSV 1000 行）を Postgres に書き込めること」
> = Real-World Readiness の完成を象徴するデモ

v31.0.0 をもって、Favnir の **Real-World Readiness** を正式に宣言する。

`fav new --template postgres-etl` による 4 ファイル構成テンプレート（types / validators / stages / main）が生成され、
`fav check` / `fav test` / `fav lint` の全コマンドが通過する。
`examples/csv-to-postgres/` に CSV 1000 行 → Postgres の実証パイプラインが実装され、
`fav test`（引数なし）がプロジェクト全体のテストを一括実行できるようになった。

### 達成コンポーネント（v30.1〜v30.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| ビルド軽量化 | v30.1 | `[profile.dev] debug = 0` で target/ 削減 |
| postgres-etl テンプレート v2 | v30.2 | 4 ファイル構成・`fav check` 全通過 |
| マルチファイル E2E | v30.3 | 5 コマンド（check/run/test/lint/fmt）全通過 |
| Rune import マルチファイル | v30.4 | 同一 Rune を複数ファイルから import 可能 |
| ドッグフードサンプル | v30.5 | `examples/csv-to-postgres/` 5 ステージ実装 |
| fav test プロジェクト統合 | v30.6 | 引数なし `fav test` でプロジェクト全体実行 |
| エラー表示改善 | v30.7 | ステージ名・ヒント付きランタイムエラー |
| fav new --list | v30.8 | 8 テンプレートの一覧表示 |
| ドッグフード修正 | v30.9 | `[project]` 解析・import 解決・UX hint |

**宣言日**: 2026-07-02
**宣言バージョン**: v31.0.0

---

## v30.0.0 — Ecosystem Maturity（2026-07-01）

> 「`fav add stripe` で Stripe 連携 Rune が 5 分で動き、
>  コミュニティ投稿 Rune が Registry に 10 本以上存在する」
> = Ecosystem Maturity の完成を象徴するデモ

v30.0.0 をもって、Favnir の **Ecosystem Maturity** を正式に宣言する。

Rune Registry（fav publish / add / search / info）が本番稼働し、
コミュニティ投稿 Rune 10 本（stripe / twilio / notion / linear / airtable /
sendgrid / hubspot / zendesk / shopify / intercom）が `runes/` 下に存在する。
AI/ML Rune 4 本（mlflow / pinecone / vertex-ai / sagemaker）と
VS Code 拡張・ドキュメントサイト v3（cookbook 32 本）が揃い、
「Favnir で書いたパイプラインをコミュニティが Rune で拡張できる」状態を達成した。

### 達成コンポーネント（v29.1〜v29.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| Rune Registry（fav publish / add / search / info） | v29.1 | Lambda + S3 + GitHub OAuth |
| mlflow Rune | v29.2 | start_run / log_metric / log_param / log_artifact / register_model |
| pinecone Rune | v29.3 | upsert / query / delete / fetch / describe_index_stats |
| vertex-ai / sagemaker Rune | v29.4 | predict / batch_predict / invoke / create_endpoint |
| github Rune | v29.5 | create_comment / create_issue / update_issue / list_prs |
| pagerduty Rune | v29.6 | create_incident / resolve / acknowledge / add_note |
| VS Code 拡張 公式リリース | v29.7 | TextMate grammar / LSP クライアント / Task Runner 統合 |
| ドキュメントサイト v3 | v29.8 | cookbook 32 本 / community ページ |
| コミュニティ Rune コンテスト | v29.9 | 10 本スタブ / CONTRIBUTING.md ガイド |

### 残件（v31.x）

- Rune Registry への実際のパッケージアップロード（Lambda 本番稼働後）
- コミュニティ Rune の HTTP 認証ヘッダー対応（HTTP Rune 有効化後）
- VS Code Marketplace への公開（手動）

**宣言日**: 2026-07-01
**宣言バージョン**: v30.0.0

---

**宣言日**: 2026-06-24
**宣言バージョン**: v25.0.0 = v1.0 リリース候補

---

## 宣言

> 「Favnir は Rust の力を借りながら、Rust を使わずに Favnir の世界を記述できる」

v25.0.0 をもって、Favnir の **Practical Self-Hosting** を正式に宣言する。

コンパイラ・型チェッカー・CLI・VM 仕様のすべてが Favnir で実装された。
Rust が担うのは VM の実行基盤（バイトコードディスパッチループ）のみであり、
これは設計上の意図であり制約ではない。

---

## 達成済みコンポーネント

| コンポーネント | ファイル | 実装言語 | 達成バージョン |
|---|---|---|---|
| コンパイラ | compiler.fav | Favnir ✓ | v8.5.0〜 |
| 型チェッカー | checker.fav | Favnir ✓ | v8.1.0〜 |
| CLI | cli.fav | Favnir ✓ | v7.6.0〜 |
| VM 仕様 | vm.fav | Favnir ✓ | v24.0.0〜 |
| VM 実行基盤 | src/backend/vm.rs | Rust（永続・設計上） | — |

### VM エンジンが Rust である理由

バイトコードのディスパッチループ・スタック管理・メモリアロケーションは、
Rust の安全性保証とゼロコスト抽象化が最も価値を発揮する領域です。
**これは Favnir の自己記述能力の欠如ではなく、正しい責任分担の結果です**。
VM の「仕様・動作の記述」は vm.fav（Favnir）が担い、
「実行の実装」は Rust が担う——このハイブリッド戦略こそが Favnir の強みです。

---

## セルフホスト達成の歴史

| バージョン | 達成内容 |
|---|---|
| v7.6.0 | cli.fav: `fav run` / `fav check` / `fav new` をすべて Favnir で実装 |
| v8.1.0 | checker.fav: `fav check` が Favnir 型チェッカー経由で動作 |
| v8.5.0 | compiler.fav: `fav run` がデフォルトで Favnir コンパイラ経由で動作 |
| v9.0.0 | セルフホスト完成宣言（compiler + checker + cli すべて Favnir 経由） |
| v24.0.0 | vm.fav: VM 仕様を Favnir で記述・テスト通過 |
| **v25.0.0** | **Practical Self-Hosting 宣言（本バージョン）** |

---

## 最終テスト（v25.0.0 達成状況）

| # | テスト | 状態 |
|---|---|---|
| 1 | `cargo test --bin fav` — 1974 件全 PASS | ✓ 達成（v25.0.0） |
| 2 | `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` | 延期（v25.x: vm.fav Phase 6） |
| 3 | `fav run --vm=self/vm.fav self/checker.fav` E2E | 延期（v25.x） |
| 4 | `fav run --vm=self/vm.fav self/cli.fav` E2E | 延期（v25.x） |
| 5 | 4-stage bootstrap 全 6 fixture（Stage 4 = vm.fav） | 延期（v25.x） |

テスト 2〜5 は vm.fav Phase 6（ユーザー定義関数ディスパッチ、実装では `CallNamed` opcode として確定）が
未実装のため v25.x に延期。テスト 1 の全件 PASS をもって v25.0.0 の完了条件とする。

---

## v1.x 後方互換性保証

v25.0.0 = v1.0 リリース候補として、後方互換性ポリシーを確定した。
詳細は [STABILITY.md](./STABILITY.md) を参照。

---

## v29.0.0 — Observability First（2026-06-28）

> 「`#[track(latency, error_rate)]` を stage に付けるだけで
>  Grafana ダッシュボードにメトリクスが現れる」
> = Observability First の完成を象徴するデモ

v29.0.0 をもって、Favnir の **Observability First** を正式に宣言する。

prometheus / datadog / sentry / grafana / otel の 5 Rune が揃い、
`#[track]` / `#[trace]` / `#[on_error]` アノテーションと E2E デモ 3 本が
Docker Compose で動作する。パイプラインの内側を型安全に観測できる状態を達成した。

### 達成コンポーネント（v28.1〜v28.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| prometheus Rune | v28.1 | counter / gauge / histogram / push + `#[track]` アノテーション |
| datadog Rune | v28.2 | metric / log / trace / event / service_check |
| OpenTelemetry Rune（otel 強化） | v28.3 | start_span / set_attribute / add_event / end_span + `#[trace]` アノテーション |
| `fav profile` 強化 | v28.4 | `--format flamegraph`（SVG 生成）/ `--compare <version>` |
| sentry Rune | v28.5 | capture_error / capture_message / set_user / set_tag / set_extra + `#[on_error]` アノテーション |
| grafana Rune | v28.6 | create_annotation / push_dashboard / snapshot |
| E2E デモ（prometheus + grafana） | v28.7 | `#[track]` stage → Grafana ダッシュボード自動反映 |
| E2E デモ（datadog APM） | v28.8 | `#[trace]` stage → Datadog サービスマップ・フレームグラフ |
| E2E デモ（sentry アラート） | v28.9 | `#[on_error]` stage → Sentry critical アラート自動送信 |

### 象徴デモ

```favnir
import runes/prometheus
import runes/grafana

// #[track] を付けるだけで Grafana ダッシュボードにメトリクスが現れる
// #[track(latency: true, error_rate: true)]
stage ExtractOrders: Unit -> List<RawOrder> !Db = |_| {
    Postgres.query[RawOrder](conn, "SELECT * FROM orders WHERE status = 'pending'")
}

// #[track(latency: true)]
stage TransformOrders: List<RawOrder> -> List<Order> !Pure = |rows| {
    Result.ok(List.map(rows, parse_order))
}

// #[track(latency: true, error_rate: true)]
stage LoadToWarehouse: List<Order> -> Unit !Db = |orders| {
    Postgres.execute_many(conn, "INSERT INTO warehouse SELECT * FROM ?", orders)
}

seq ObservabilityFirstDemo = ExtractOrders |> TransformOrders |> LoadToWarehouse
```

### v29.x 残件（次フェーズ）

- prometheus / grafana 実メトリクス送信の統合テスト（実際の pushgateway との E2E）
- `#[track]` / `#[trace]` / `#[on_error]` アノテーションのコンパイラ自動挿入実装
- Datadog APM トレース送信の実統合（DSN 本番テスト）
- `fav profile --compare` の stage 別 JSON 比較精度向上

---

## v28.0.0 — Data Lakehouse（2026-06-27）

> 「Delta Lake テーブルを Favnir から型安全に読み書きし、
>  dbt モデルの結果を次のステージに渡す」
> = Data Lakehouse の完成を象徴するデモ

v28.0.0 をもって、Favnir の **Data Lakehouse** を正式に宣言する。

Delta Lake / Iceberg テーブルの読み書き、dbt モデル参照、
主要 DWH 3 本（ClickHouse / BigQuery / Redshift）への接続、
SQLite 組み込み DB が揃い、現代データ基盤アーキテクチャへの完全統合を達成した。

### 達成コンポーネント（v27.1〜v27.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| delta-lake Rune | v27.1 | read / write / merge / history / vacuum / optimize |
| iceberg Rune | v27.2 | read / append / overwrite / time_travel / schema_evolution / list_snapshots |
| clickhouse Rune | v27.3 | connect / query / insert / async_insert |
| bigquery Rune | v27.4 | connect / query / insert / load_from_gcs / create_table |
| redshift Rune | v27.5 | connect / query / execute / copy_from_s3 / unload_to_s3 |
| jsonl Rune | v27.6 | read / write / stream / append |
| `fav infer --from delta/iceberg` | v27.7 | Delta / Iceberg スキーマ → Favnir 型定義自動生成 |
| dbt 連携 Rune | v27.8 | ref / source（manifest.json 解析、`!Db` エフェクト） |
| sqlite Rune | v27.9 | open / open_memory / query / execute / execute_many / close |

### 象徴デモ

```favnir
import rune "delta-lake"
import rune "dbt"
import rune "sqlite"

// Delta Lake からロード → dbt モデル参照 → SQLite に保存
stage LoadFromDelta: Unit -> List<OrderRow> !Io = |_| {
    DeltaLake.read[OrderRow]("s3://my-bucket/orders")
}

stage EnrichWithDbt: List<OrderRow> -> List<EnrichedOrder> !Db = |orders| {
    bind summary <- Dbt.ref(config.dbt, "customer_summary")
    Result.ok(List.map(orders, |o| enrich(o, summary)))
}

stage SaveToSqlite: List<EnrichedOrder> -> Unit !Db = |rows| {
    bind db <- SQLite.open_memory()
    bind _  <- SQLite.execute(db, "CREATE TABLE orders (id INT, amount REAL)", "[]")
    SQLite.execute_many(db, "INSERT INTO orders VALUES (?, ?)", rows)
}

seq DataLakehousePipeline = LoadFromDelta |> EnrichWithDbt |> SaveToSqlite
```

### v28.x 残件（次フェーズ）

- delta-rs 実統合（実際の Delta テーブル読み書き）
- rusqlite 実統合（実際の SQLite 操作）
- dbt manifest.json 実解析と SQL 実行
- Iceberg REST カタログ実統合

---

## v27.0.0 — Streaming Native（2026-06-27）

> 「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
> = Streaming Native の完成を象徴するデモ

v27.0.0 をもって、Favnir の **Streaming Native** を正式に宣言する。

ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）が実質化され、
`Stream.*` 操作 6 関数（map / filter / flat_map / window / merge / split）が使用可能になり、
E2E デモ 3 本（kafka→ES / kinesis→S3 / nats→postgres）が Docker Compose で動作する。

### 達成コンポーネント（v26.1〜v26.9）

| コンポーネント | バージョン | 実装済み関数 |
|---|---|---|
| kinesis Rune | v26.1.0 | connect / put_record / put_records / get_shard_iterator / get_records |
| nats Rune | v26.2.0 | connect / publish / subscribe / jetstream_publish / jetstream_consume |
| rabbitmq Rune | v26.3.0 | connect / declare_exchange / declare_queue / bind_queue / publish / consume |
| Stream.* 操作 6 関数 | v26.4.0 | map / filter / flat_map / window / merge / split |
| E2E デモ: kafka → Elasticsearch | v26.5.0 | `examples/streaming/kafka_to_elasticsearch.fav` |
| E2E デモ: kinesis → S3 | v26.6.0 | `examples/streaming/kinesis_to_s3.fav` |
| E2E デモ: nats → postgres | v26.7.0 | `examples/streaming/nats_to_postgres.fav` |
| sqs Rune | v26.8.0 | send_message / send_message_batch / receive_messages / delete_message / purge / consume |
| pulsar Rune | v26.9.0 | produce / consume / ack / nack（暫定 `!AWS` エフェクト、v27.x で `!Pulsar` へ移行予定） |

### Streaming Native 検証コマンド

```bash
docker compose -f examples/streaming/docker-compose.yml up -d
fav run examples/streaming/kafka_to_elasticsearch.fav
fav run examples/streaming/kinesis_to_s3.fav
fav run examples/streaming/nats_to_postgres.fav
```

### v27.x 残件（次フェーズ）

- kinesis: `Kinesis.consume[T]` 継続消費ループ
- nats: `NATS.request[T]` リクエスト/レスポンス
- rabbitmq: `RabbitMQ.ack` / `RabbitMQ.nack`
- pulsar: Binary Protocol 経由の高速 produce

---

## v26.0.0 — Rune Foundation（2026-06-26）

> 「Favnir で書いたパイプラインが実際の本番データを動かせる」

v26.0.0 をもって、Favnir の **Rune Foundation** を正式に宣言する。

コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が
「動く Rune の 5 条件（connect / read / write / error / test）」をすべてクリアした。
また vm.fav Phase 6（`CallNamed` opcode, 0x56）が完成し、
multi-function Favnir プログラムを vm.fav インタープリター上で実行できるようになった。

### 達成した Rune

| Rune | 条件 | 主要関数 |
|---|---|---|
| postgres | connect / read / write / error / test ✓ | connect / query / execute / execute_many / transaction / Pool |
| s3 | connect / read / write / error / test ✓ | get_object / put_object / list_objects / delete_object / presign_url |
| redis | connect / read / write / error / test ✓ | get / set / del / incr / lpush / rpop / publish / subscribe |
| mysql | connect / read / write / error / test ✓ | connect / query / execute / transaction（DbConn interface 統一） |
| mongodb | connect / read / write / error / test ✓ | find / find_one / insert_one / insert_many / update_one / delete_one / aggregate |
| dynamodb | connect / read / write / error / test ✓ | get_item / put_item / delete_item / query / scan / batch_write / transact_write |
| kafka | connect / read / write / error / test ✓ | produce / consume / consume_batch / commit / seek |
| elasticsearch | connect / read / write / error / test ✓ | index / search / bulk / delete / knn_search / create_index |

### デモ

```bash
# postgres → 集計 → s3 → kafka 通知
fav run examples/full_etl.fav

# postgres ETL
fav run examples/postgres_etl.fav

# s3 CSV → Parquet 変換
fav run examples/s3_csv_to_parquet.fav
```

### vm.fav Phase 6 達成

`CallNamed(name_idx, argc)` opcode (0x56) の実装により、
multi-function Favnir プログラムを vm.fav インタープリター上で実行できるようになった。

```bash
# multi-function プログラムを vm.fav 経由で実行
fav run --vm self/vm.fav --compile hello.fav
```

---

<!-- 以下は v35.0.0 宣言（冒頭）への追記事項 — 重複エントリではなく補足記録 -->
<!-- 正史: このファイル冒頭の "## v35.0.0 — Production Ready（2026-07-04）" が正式宣言 -->

## v35.0.0 追記 — !Effect 廃止完結（2026-07-05〜06）

v35.6.0〜v35.8.0 にて `!Effect` アノテーション構文が言語から**完全に削除**され、
v35.0.0 宣言文の「Lambda にデプロイして実データを処理できる」が完全に充足された。

| カテゴリ | バージョン | 内容 |
|---|---|---|
| E0374 ハードエラー化 | v35.4.0 | `!Effect` を書くとパースエラー |
| Effect enum 完全削除 | v35.5.0 | `ast.rs` 以降 14 ファイルから物理削除（約 380 行） |
| ドキュメント統一 | v35.6.0 | サイト MDX 128 ファイル・317 コードブロックを ctx 構文に変換 |
| LSP / error_catalog / MCP | v35.7.0〜v35.8.0 | 残存 `!Effect` 文字列をすべて除去 |
| 最終テスト数 | v35.8.0 | 2621 tests pass（0 failures）、cargo clippy clean |

**Lambda デプロイの現状**（v35.0 宣言時との差分）:
- v35.0 宣言文「Lambda にデプロイして実データを処理できる」＝ `fav build --target native` 後に**手動**デプロイ
- v36.1〜v37.0（次スプリント）で `fav deploy --target lambda` **CLI 自動化** を実装予定

```bash
# ctx 構文でのパイプライン（v35.0 以降の標準）
fav run examples/postgres_etl.fav

# 旧 !Effect 構文は E0374 でパースエラー
fav check --legacy examples/pipeline/custom_effects.fav  # E0374
```

---

## v36.0 — Deployment Story（2026-07-08）

v35.1〜v35.9 スプリントで実装した機能を統合し、Deployment Story マイルストーンを宣言する。

> 「`fav deploy --target lambda` で Lambda に自動デプロイし、
>  `fav deploy --target docker` で Docker イメージを生成し、
>  `fav ci init` で GitHub Actions CI を自動設定できる。
>  `!Effect` 廃止（v35.4〜v35.8）により、すべての API が ctx: AppCtx ベースに統一された。
>
>  これが Favnir v36.0 — Deployment Story の姿である。」

### 達成コンポーネント（v35.1〜v35.9）

| バージョン | 内容 |
|---|---|
| v35.1.0 | `fav deploy --target lambda` — Lambda 自動デプロイ・bootstrap.zip パッケージング |
| v35.2.0 | `fav deploy --target docker` — Dockerfile 自動生成・`docker build` 実行 |
| v35.3.0 | `fav ci init` — GitHub Actions CI ワークフロー自動生成 |
| v35.4.0 | `!Effect` E0374 ハードエラー化 |
| v35.5.0 | Effect enum・effects フィールド・parse_effects_acc 完全削除 |
| v35.6.0 | ctx 構文統一（MDX 128 件）+ Production Ready 宣言補完 |
| v35.7.0 | `docs_server.rs !Effect` 完全除去 |
| v35.8.0 | LSP / error_catalog / MCP / help !Effect 廃止完結 |
| v35.9.0 | v36.0 前調整・安定化（E2E 確認・lambda-deploy デモ確認） |
