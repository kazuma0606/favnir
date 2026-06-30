# Changelog

Favnir のバージョン履歴。形式は [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に準拠。

---

## [v30.0.0] — 2026-07-01

### Added
- `MILESTONE.md` — Ecosystem Maturity 宣言セクション追加
- `site/content/docs/ecosystem-maturity.mdx` — マイルストーン宣言ドキュメント（`fav add stripe` デモ）
- `versions/roadmap/roadmap-v29.1-v30.0.md` — 達成宣言（COMPLETE）追記
- テスト数: 2366 → 2372（+6）

---

## [v29.9.0] — 2026-07-01

### Added
- `CONTRIBUTING.md` — コミュニティ Rune 開発ガイド（5 条件: connect / read / write / error / test）追加
- `runes/stripe|twilio|notion|linear|airtable|sendgrid|hubspot|zendesk|shopify|intercom/` — コミュニティ Rune スタブ 10 本（各 `.fav` + `rune.toml`）
- `site/app/community/page.tsx` — 第 1 回 Rune コンテスト告知セクション追加
- テスト数: 2360 → 2366（+6）

---

## [v29.8.0] — 2026-07-01

### Added
- `site/content/cookbook/` — cookbook 29 本追加（3 → 32 本）
- `site/app/community/` — `/community/` ページ新設（GitHub Discussions / Discord リンク）
- テスト数: 2354 → 2360（+6）

---

## [v29.7.0] — 2026-06-30

### Added
- `extensions/vscode-favnir/` — VS Code 拡張パッケージ（TextMate grammar / LSP クライアント / Task Runner 統合）
- `site/content/docs/tools/vscode-extension.mdx` — VS Code 拡張ドキュメント
- テスト数: 2348 → 2354（+6）

---

## [v29.6.0] — 2026-06-30

### Added
- `runes/pagerduty/` — PagerDuty Events API v2 Rune（create_incident / resolve / acknowledge / add_note）
- `site/content/docs/runes/pagerduty.mdx` — PagerDuty Rune ドキュメント
- テスト数: 2342 → 2348（+6）

---

## [v29.5.0] — 2026-06-30

### Added
- `runes/github/` — GitHub REST API Rune（create_comment / create_issue / update_issue / list_prs / get_pr）
- `site/content/docs/runes/github.mdx` — GitHub Rune ドキュメント
- テスト数: 2336 → 2342（+6）

---

## [v29.4.0] — 2026-06-30

### Added
- `runes/vertex-ai/` — Google Vertex AI Rune（predict / batch_predict / deploy_model / list_endpoints）
- `runes/sagemaker/` — AWS SageMaker Rune（invoke / create_endpoint / delete_endpoint）
- `site/content/docs/runes/vertex-ai.mdx` / `sagemaker.mdx` — ドキュメント追加
- テスト数: 2330 → 2336（+6）

---

## [v29.3.0] — 2026-06-30

### Added
- `runes/pinecone/` — Pinecone ベクトルDB Rune（upsert / query / delete / fetch / describe_index_stats）
- RAG パイプラインサポート: LLM Rune と組み合わせてドキュメント検索付き LLM パイプラインを構築可能
- `site/content/docs/runes/pinecone.mdx` — Pinecone Rune ドキュメント
- テスト数: 2324 → 2330（+6）

---

## [v29.2.0] — 2026-06-30

### Added
- `runes/mlflow/mlflow.fav` — MLflow Rune 実装（`start_run` / `log_metric` / `log_param` / `log_artifact` / `end_run` / `register_model` / `load_model` / `list_experiments` 8 関数）
- `runes/mlflow/rune.toml` — Rune メタデータ
- `site/content/docs/runes/mlflow.mdx` — MLflow ドキュメント追加
- テスト数: 2318 → 2324（+6）

---

## [v29.1.0] — 2026-06-28

### Added
- `cmd_publish`（`driver.rs`）— `FAVNIR_REGISTRY_URL` 環境変数を参照し `{url}/v1/publish` へのリモート API 呼び出しを追加（インフラ稼働後に HTTP POST 有効化）
- `pub fn cmd_info(pkg_name: &str)`（`driver.rs`）— `FAVNIR_REGISTRY_URL/v1/packages/{name}` へのリモート参照 + 静的カタログフォールバック
- `cmd_search`（`driver.rs`）— `FAVNIR_REGISTRY_URL` 設定時に `/v1/search?q=...` へのリモート参照を追加（フォールバック: 静的カタログ）
- `cmd_login`（`driver.rs`）— GitHub OAuth URL 生成（`https://github.com/login/oauth/authorize?client_id=...&scope=read:user`）
- `Some("info")`（`main.rs`）— `fav info <pkg>` サブコマンドを `cmd_info` にルーティング
- `benchmarks/v29.1.0.json` — テスト数 2318 件を記録

### Changed
- `Cargo.toml` version `29.0.0` → `29.1.0`

---

## [v29.0.0] — 2026-06-28

### Added
- `MILESTONE.md` に "Observability First" セクション追加（v28.1〜v28.9 達成コンポーネント一覧・象徴デモ・v29.x 残件）
- `site/content/docs/observability-first.mdx` — Observability First マイルストーン解説ドキュメント（prometheus / sentry / grafana コード例・`fav profile` 使用例・マイルストーン履歴）
- `README.md` に v29.0 "Observability First" マイルストーン参照を追記
- `versions/roadmap/roadmap-v28.1-v29.0.md` に v29.0 完了マーク追記
- `benchmarks/v29.0.0.json` — テスト数 2312 件を記録

### Fixed
- `examples/observability/prometheus_grafana.fav` — `fn main()` 追加 + `Result.ok(unit)` → `Result.ok(())` 修正（`fav run` 対応）
- `examples/observability/datadog_apm.fav` — `fn main()` 追加 + `Result.ok(unit)` → `Result.ok(())` 修正（`fav run` 対応）
- `fav/tests/fixtures/etl.fav` — `{ unit }` → `{ () }` 修正 + `fn main()` 追加（`fav profile --format flamegraph` 対応）
- `examples/observability/docker-compose.yml` — `sentry-redis`（redis:7-alpine）/ `sentry-postgres`（postgres:15-alpine）追加（sentry の依存サービス）

### Changed
- `Cargo.toml` version `28.9.0` → `29.0.0`

---

## [v28.9.0] — 2026-06-28

### Added
- `examples/observability/sentry_alerting.fav` 拡充 — `CriticalLoad` stage 追加（`// #[on_error(report_to: "sentry", level: "critical")]` コメント形式）+ `Sentry.capture_message` 使用 + `seq SentryAlertingDemo` を `CriticalLoad |> ReportError |> SetContext` に拡充
- `examples/observability/docker-compose.yml` に `getsentry/sentry:24.0` サービス追加
- `site/content/docs/tools/sentry-alerting.mdx` — Sentry アラート E2E デモ解説ドキュメント

---

## [v28.8.0] — 2026-06-28

### Added
- `examples/observability/datadog_apm.fav` 拡充 — 3 stage（ExtractEvents / TransformEvents / LoadEvents）+ `// #[trace(service:` コメント形式アノテーション
- `examples/observability/docker-compose.yml` に `datadog-agent:7` サービス追加
- `site/content/docs/tools/datadog-apm.mdx` — Datadog APM E2E デモ解説ドキュメント

---

## [v28.7.0] — 2026-06-28

### Added
- `examples/observability/prometheus_grafana.fav` — ETL パイプライン × Prometheus + Grafana E2E デモ（`PrometheusGrafanaDemo` seq）
- `examples/observability/docker-compose.yml` — prometheus / grafana Docker 定義
- `examples/observability/README.md` — セットアップ手順（docker compose up → fav run → Grafana UI）
- `site/content/docs/tools/observability-e2e.mdx` — E2E デモ解説ドキュメント

---

## [v28.6.0] — 2026-06-28

### Added
- `runes/grafana/grafana.fav` — Grafana ダッシュボード管理 Rune（create_annotation / push_dashboard / snapshot）
- `Grafana.create_annotation_raw` / `push_dashboard_raw` / `snapshot_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `fav/self/checker.fav` `ns_to_effect` に `"Grafana" => "IO"` 追加
- `examples/observability/grafana_dashboard.fav` — GrafanaDashboardDemo E2E デモ
- `site/content/docs/runes/grafana.mdx` — ドキュメント追加

---

## [v28.5.0] — 2026-06-28

### Added
- `runes/sentry/sentry.fav` — Sentry エラートラッキング Rune（capture_error / capture_message / set_user / set_tag / set_extra）
- `Sentry.capture_error_raw` / `capture_message_raw` / `set_user_raw` / `set_tag_raw` / `set_extra_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `fav/self/checker.fav` `ns_to_effect` に `"Sentry" => "IO"` 追加
- `examples/observability/sentry_alerting.fav` — SentryAlertingDemo E2E デモ
- `site/content/docs/runes/sentry.mdx` — ドキュメント追加

---

## [v28.4.0] — 2026-06-28

### Added
- `fav profile --compare <version>` — ベースラインベンチマークとの stage 別実行時間比較（`[SLOWER]` / `[FASTER]` / `[NEW]` マーカー出力）
- `pub fn cmd_profile_compare` を `driver.rs` に追加
- `fav/tests/fixtures/etl.fav` — プロファイルテスト用 ETL フィクスチャ
- `site/content/docs/performance/profiling.mdx` に `--compare` セクション追加

---

## [v28.3.0] — 2026-06-28

### Added
- `runes/otel/otel.fav` — OpenTelemetry Rune（start_span / set_attribute / add_event / end_span）
- `OTel.start_span_raw` / `OTel.set_attribute_raw` / `OTel.add_event_raw` / `OTel.end_span_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `fav/self/checker.fav` `ns_to_effect` に `"OTel" => "IO"` 追加
- `examples/observability/otel_tracing.fav` — OTelTracingDemo E2E デモ
- `site/content/docs/runes/otel.mdx` — ドキュメント追加

---

## [v28.2.0] — 2026-06-27 — datadog Rune 追加

### Added
- `runes/datadog/datadog.fav` — Datadog APM/Metrics/Logs Rune（metric / log / trace / event / service_check 5 関数、`!Io` エフェクト）
- `Datadog.metric_raw` / `log_raw` / `trace_raw` / `event_raw` / `service_check_raw` VM primitive 5 件追加（`#[cfg]` ガード付き）
- `examples/observability/datadog_apm.fav` — APM トレース + メトリクス送信デモ（DatadogApmDemo seq pipeline）
- `site/content/docs/runes/datadog.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"Datadog" => "IO"` 追加

### Notes
- v28.2.0 は stub 実装。実際の DogStatsD / Datadog API 送信は v28.x 以降
- `#[trace]` アノテーションは v28.3+ で独立バージョンとして実装予定

---

## [v28.1.0] — 2026-06-27 — prometheus Rune 追加

### Added
- `runes/prometheus/prometheus.fav` — Prometheus メトリクス Rune（counter / gauge / histogram / push 4 関数、`!Io` エフェクト）
- `Prometheus.counter_raw` / `gauge_raw` / `histogram_raw` / `push_raw` VM primitive 4 件追加（`#[cfg]` ガード付き）
- `examples/observability/prometheus_demo.fav` — カスタムメトリクス送信デモ（PrometheusDemo seq pipeline）
- `site/content/docs/runes/prometheus.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"Prometheus" => "Io"` 追加

### Notes
- v28.1.0 は stub 実装。実際の Pushgateway HTTP 送信は v28.x 以降
- `#[track]` アノテーションは v28.2+ で独立バージョンとして実装予定

---

## [v28.0.0] — 2026-06-27 — Data Lakehouse マイルストーン宣言

### Added
- `MILESTONE.md` に "Data Lakehouse" セクション追加（v27.1〜v27.9 完了コンポーネント一覧・象徴デモ・v28.x 残件）
- `site/content/docs/data-lakehouse.mdx` — Data Lakehouse マイルストーン解説ページ
- `benchmarks/v28.0.0.json` — ベンチマーク記録（test_count: 2226）

### Notes
- v27.1〜v27.9 で実装した Data Lakehouse スタック（Delta Lake / Iceberg / ClickHouse / BigQuery / Redshift / JSONL / dbt / SQLite / `fav infer --from delta/iceberg`）の全コンポーネント完成を宣言
- v28.x では stub 実装（delta-rs / rusqlite / manifest.json 実解析）を本統合に移行予定

---

## [v27.9.0] — 2026-06-27 — sqlite Rune 追加

### Added
- `runes/sqlite/sqlite.fav` — SQLite Rune（open / open_memory / query / execute / execute_many / close 6 関数、`!Db` エフェクト）
- `SQLite.open_raw` / `open_memory_raw` / `query_raw` / `execute_raw` / `execute_many_raw` / `close_raw` VM primitive 6 件追加（`#[cfg]` ガード付き）
- `examples/sqlite_etl.fav` — SQLite 軽量 ETL パイプラインデモ
- `site/content/docs/runes/sqlite.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"SQLite" => "Db"` 追加

### Notes
- v27.9.0 は stub 実装。実際の SQLite 操作（`rusqlite` クレート統合）は v28.x に延期

---

## [v27.8.0] — 2026-06-27 — dbt 連携 Rune

### Added
- `runes/dbt/dbt.fav` — dbt 連携 Rune（ref / source 2 関数、`!Db` エフェクト）
- `Dbt.ref_raw` / `Dbt.source_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `examples/dbt_pipeline.fav` — dbt モデル参照パイプラインデモ
- `fav/tests/fixtures/dbt_manifest.json` — manifest.json モックフィクスチャ
- `site/content/docs/runes/dbt.mdx` ドキュメント追加
- `fav/self/checker.fav` `ns_to_effect` に `"Dbt" => "Db"` 追加

### Notes
- v27.8.0 は stub 実装。`manifest.json` の実解析と SQL 実行は v28.x に延期

---

## [v27.7.0] — 2026-06-27 — `fav infer --from delta` / `--from iceberg`

### Added
- `fav infer --from delta --path <path>` — Delta Lake テーブルスキーマから Favnir 型定義を自動生成（v27.7.0 stub）
- `fav infer --from iceberg --catalog <url> --table <name>` — Iceberg テーブルスキーマから型定義を自動生成（v27.7.0 stub）
- `DeltaLake.infer_schema_raw` / `Iceberg.infer_schema_raw` VM primitive 追加（`#[cfg]` ガード付き）
- `delta_type_to_favnir` 型マッピング関数（long/integer/int→Int, double/float→Float, string→String, timestamp→DateTime, boolean→Bool）
- `site/content/docs/tools/infer-delta-iceberg.mdx` ドキュメント追加

### Notes
- v27.7.0 は stub 実装。実際のテーブルスキーマ読み取りは v28.x（`delta-rs` / `iceberg-rust` 統合時）に実装予定

---

## [v27.6.0] — 2026-06-27 — jsonl Rune 追加

### Added
- `runes/jsonl/jsonl.fav` — JSONL Rune（read / write / stream / append 4 関数、`!Io` エフェクト）
- `JSONL.*_raw` VM primitives 4 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/jsonl_etl.fav` — JSONL ETL デモ（ReadData |> WriteProcessed）
- `site/content/docs/runes/jsonl.mdx` — JSONL Rune ドキュメント

---

## [v27.5.0] — 2026-06-27 — redshift Rune 追加

### Added
- `runes/redshift/redshift.fav` — Redshift Rune（connect / query / execute / copy_from_s3 / unload_to_s3 5 関数、`!Db` エフェクト）
- `Redshift.*_raw` VM primitives 5 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/redshift_analytics.fav` — Redshift Analytics デモ（LoadFromS3 |> QuerySummary |> UnloadToS3）
- `site/content/docs/runes/redshift.mdx` — Redshift Rune ドキュメント

---

## [v27.4.0] — 2026-06-27 — bigquery Rune 実質化

### Added
- `runes/bigquery/bigquery.fav` — BigQuery Rune 実質化（connect / query / insert / load_from_gcs / create_table 5 関数、`!Db` エフェクト）
- `BigQuery.connect_raw` / `BigQuery.conn_query_raw` / `BigQuery.insert_raw` / `BigQuery.load_from_gcs_raw` / `BigQuery.create_table_raw` VM primitives 5 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/bigquery_analytics.fav` — BigQuery Analytics デモ（CreateEventTable |> LoadFromGcs |> QueryStats）
- `site/content/docs/runes/bigquery.mdx` — BigQuery Rune ドキュメント（v27.4.0 新 API）

### Changed
- `runes/bigquery/bigquery.fav` — v15.2.0 の `!Gcp` エフェクト・非 public API から `!Db` エフェクト・`public fn` API に刷新（既存 `BigQuery.query_raw` / `BigQuery.execute_raw` VM primitive は後方互換として残存）

---

## [v27.3.0] — 2026-06-27 — clickhouse Rune 追加

### Added
- `runes/clickhouse/clickhouse.fav` — ClickHouse Rune（connect / query / insert / async_insert 4 関数）
- `ClickHouse.*_raw` VM primitives 4 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/clickhouse_analytics.fav` — ClickHouse Analytics デモ（LoadEvents |> InsertProcessed）
- `site/content/docs/runes/clickhouse.mdx` — ClickHouse Rune ドキュメント

---

## [v27.2.0] — 2026-06-27 — iceberg Rune 追加

### Added
- `runes/iceberg/iceberg.fav` — Apache Iceberg Rune（read / append / overwrite / time_travel / schema_evolution / list_snapshots 6 関数）
- `Iceberg.*_raw` VM primitives 6 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/iceberg_etl.fav` — Iceberg ETL デモ（LoadFromIceberg |> TransformData |> AppendToIceberg）
- `site/content/docs/runes/iceberg.mdx` — Apache Iceberg Rune ドキュメント

---

## [v27.1.0] — 2026-06-27 — delta-lake Rune 追加

### Added
- `runes/delta-lake/delta-lake.fav` — Delta Lake Rune（read / read_with_filter / write / merge / history / vacuum / optimize 7 関数）
- `DeltaLake.*_raw` VM primitives 7 件（`#[cfg(not(target_arch = "wasm32"))]` ガード付き、stub 実装）
- `examples/delta_lake_etl.fav` — Delta Lake ETL デモ（LoadRawData |> TransformOrders |> SaveProcessed）
- `site/content/docs/runes/delta-lake.mdx` — Delta Lake Rune ドキュメント

---

## [v27.0.0] — 2026-06-27 — Streaming Native マイルストーン宣言

### Milestone
- **Streaming Native** 宣言: ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）実質化完了
- `Stream.*` 操作 6 関数（map / filter / flat_map / window / merge / split）使用可能
- E2E デモ 3 本（kafka→ES / kinesis→S3 / nats→postgres）が Docker Compose で動作

### Added
- `MILESTONE.md` に "Streaming Native" マイルストーンセクション追加
- `site/content/docs/streaming-native.mdx` — Streaming Native マイルストーン解説ページ
- `README.md` に v27.0 マイルストーン記載
- `versions/roadmap/roadmap-v26.1-v27.0.md` に完了日追記

---

## [v26.9.0] — 2026-06-27 — Pulsar Rune 追加

### Added
- `runes/pulsar/pulsar.fav` — Apache Pulsar Rune 追加（`produce` / `consume` / `ack` / `nack` 4 関数）
- `Pulsar.produce_raw` / `Pulsar.consume_raw` / `Pulsar.ack_raw` / `Pulsar.nack_raw` VM primitives（`#[cfg(not(target_arch = "wasm32"))]` ガード付き）
- `examples/streaming/docker-compose.yml` に `pulsar` サービス（`apachepulsar/pulsar:3.2.0`）追加
- `site/content/docs/runes/pulsar.mdx` — Pulsar Rune ドキュメント

---

## [v26.8.0] — 2026-06-27 — SQS Rune 実質化

### Added
- `runes/sqs/sqs.fav` — SQS Rune 実質化（`send_message` / `send_message_batch` / `receive_messages` / `delete_message` / `purge` / `consume` 6 関数）
- `fav/src/backend/vm.rs` — `SQS.send_message_batch_raw` / `SQS.receive_messages_raw` / `SQS.purge_raw` / `SQS.consume_raw` primitive 追加
- `site/content/docs/runes/sqs.mdx` — SQS Rune ドキュメント

---

## [v26.7.0] — 2026-06-27 — ストリーミング E2E デモ（nats → postgres）

### Added
- `examples/streaming/nats_to_postgres.fav` — NATS → Postgres IoT センサーデータ蓄積デモ（FetchSensorData / ValidateSensor / InsertToPostgres + `seq SensorPipeline`）
- `examples/streaming/docker-compose.yml` に `nats` / `postgres` サービス追加
- `examples/streaming/README.md` — 3 本の E2E デモ実行手順まとめ
- `site/content/docs/streaming/nats-to-postgres.mdx` — E2E デモドキュメント

---

## [v26.6.0] — 2026-06-27 — ストリーミング E2E デモ（kinesis → s3）

### Added
- `examples/streaming/kinesis_to_s3.fav` — Kinesis → S3 クリックイベントアーカイブデモ（FetchClickEvents / SerializeBatch / UploadToS3 + `seq ArchivePipeline`）
- `examples/streaming/docker-compose.yml` に `localstack` サービス追加（Kinesis / S3 ローカルエミュレーション）
- `site/content/docs/streaming/kinesis-to-s3.mdx` — E2E デモドキュメント

---

## [v26.5.0] — 2026-06-27 — ストリーミング E2E デモ（kafka → elasticsearch）

### Added
- `examples/streaming/kafka_to_elasticsearch.fav` — Kafka → Elasticsearch リアルタイムログ集計デモ（FetchLogs / FilterErrors / IndexToES + `seq LogPipeline`）
- `examples/streaming/docker-compose.yml` — Kafka（Redpanda）/ Elasticsearch サービス定義
- `site/content/docs/streaming/kafka-to-elasticsearch.mdx` — E2E デモドキュメント

---

## [v26.4.0] — 2026-06-27 — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

### Added
- `StreamingAnnotation.backpressure: Option<bool>` フィールド追加（`ast.rs` + `parser.rs`）— `#[streaming(backpressure = true)]` 構文をサポート
- `Stream.flat_map` / `Stream.window` / `Stream.merge` / `Stream.split` — VM primitive 4 件追加（`vm.rs`）
- `VMStream::FlatMap` / `VMStream::Window` / `VMStream::Merge` / `VMStream::Split` — 遅延評価バリアント追加
- `runes/stream/stream.fav` — Stream Rune 新規作成（map / filter / flat_map / window / merge / split）
- `site/content/docs/runes/stream.mdx` — Stream Rune ドキュメント新規作成

---

## [v26.3.0] — 2026-06-26 — rabbitmq Rune 実質化

### Added
- `runes/rabbitmq/rabbitmq.fav` — RabbitMQ Rune（connect / declare_exchange / declare_queue / bind_queue / publish / consume）
- `RabbitMQ.connect_raw` / `declare_exchange_raw` / `declare_queue_raw` / `bind_queue_raw` / `publish_raw` / `consume_raw` — VM primitive 6 件追加
- `site/content/docs/runes/rabbitmq.mdx` — RabbitMQ Rune ドキュメント新規作成
- `benchmarks/v26.3.0.json` — ベンチマーク記録（test_count: 2062）

---

## [v26.2.0] — 2026-06-26 — nats Rune 実質化

### Added
- `runes/nats/nats.fav` — NATS Rune（connect / publish / subscribe / jetstream_publish / jetstream_consume）
- `NATS.connect_raw` / `publish_raw` / `subscribe_raw` / `jetstream_publish_raw` / `jetstream_consume_raw` — VM primitive 5 件追加
- `site/content/docs/runes/nats.mdx` — NATS Rune ドキュメント新規作成
- `benchmarks/v26.2.0.json` — ベンチマーク記録（test_count: 2054）

---

## [v26.1.0] — 2026-06-26 — kinesis Rune 実質化

### Added
- `runes/kinesis/kinesis.fav` — Kinesis Rune（connect / put_record / put_records / get_shard_iterator / get_records）
- `Kinesis.connect_raw` / `put_record_raw` / `put_records_raw` / `get_shard_iterator_raw` / `get_records_raw` — VM primitive 5 件追加
- `site/content/docs/runes/kinesis.mdx` — Kinesis Rune ドキュメント新規作成
- `benchmarks/v26.1.0.json` — ベンチマーク記録（test_count: 2047）

---

## [v26.0.0] — 2026-06-26 — Rune Foundation マイルストーン宣言

### Milestone
- **Rune Foundation**: コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が「動く Rune の 5 条件（connect / read / write / error / test）」をすべてクリア
- `examples/full_etl.fav` — postgres → 集計 → s3 → kafka 通知の完全デモパイプライン
- `examples/postgres_etl.fav` / `examples/s3_csv_to_parquet.fav` — 個別 Rune デモ
- vm.fav Phase 6（`CallNamed` opcode, 0x56）完了: multi-function プログラムを vm.fav で実行可能（v25.9.0 完了の宣言）
- `MILESTONE.md` に「Rune Foundation」セクション追記
- `site/content/docs/rune-foundation.mdx` 新規作成
- `versions/roadmap/roadmap-v25.1-v26.0.md` — v25.1〜v25.9 COMPLETE・v26.0.0 宣言済みに更新

---

## [v25.9.0] — 2026-06-26 — vm.fav Phase 6（CallNamed 実装 — ユーザー定義関数呼び出し）

### Added
- `CallNamed(Int, Int)` opcode (0x56) — `fav/self/vm.fav` に追加（fn_name_const_idx, argc）
- `decode_byte_with_u16x2_le` — 5 バイト opcode デコーダー（u16 LE ×2 オペランド）
- `vm_execute` シグネチャ拡張: `consts: Int, prog_keys: Int, prog_vals: Int`（現在関数の定数プールとプログラムテーブル）
- `vm_run_program(program_json)` — multi-function program JSON を受け取り main 関数を実行する新エントリポイント
- 補助型・関数: `FnDef` / `ListPair` / `parse_fn_json` / `build_consts_list` / `copy_args_to_locals` / `find_fn_in_program` / `build_program_lists`
- `build_vm_program_json(artifact)` — `FvcArtifact` → program JSON シリアライザ（`Constant::Int/Float/Str/Name` 全バリアント対応）
- `run_via_vm(vm_src, program_json)` — vm.fav 経由で multi-function プログラムを実行
- `fav run --vm <path> --compile <src>` — ソースをコンパイルして vm.fav 経由で実行する CLI モード
- `site/content/docs/tools/vm-fav.mdx` — Phase 6 / `--compile` フラグ説明を追記

### Notes
- 線形検索パターン採用（`Mut.str_map` は未実装のため `prog_keys` / `prog_vals` の Mut.map ペア）
- `Constant::Name` が `CallNamed` の参照対象（`Constant::Str` ではない）
- `parse_fn_json` は単純文字列解析（関数名にカンマ・ダブルクォートなし前提）

---

## [v25.8.0] — 2026-06-25 — elasticsearch Rune 実質化（全文検索・ベクトル検索・バルク書き込み）

### Added
- `Effect::Elasticsearch` — 新規エフェクト variant（`!Elasticsearch`）
- E0324 `UndeclaredElasticsearchEffect` — エラーカタログ追加
- `ES.*_raw` 8 件 VM プリミティブ（`connect` / `index` / `index_with_id` / `search` / `knn_search` / `bulk` / `delete` / `create_index`）
- `runes/elasticsearch/elasticsearch.fav` — `ESConn` 型 + 8 関数
- `examples/elasticsearch_logs_etl.fav` — `IndexLog |> SearchLogs` パイプラインデモ
- `site/content/docs/runes/elasticsearch.mdx` — 全 API リファレンス（Docker セットアップ・認証・スコープ外）
- 認証: `ELASTICSEARCH_API_KEY`（優先）→ Basic（USERNAME/PASSWORD）→ 認証なし
- `knn_search` — kNN ベクトル検索（`_source` 配列 JSON 文字列で返す）
- `bulk` — JSON 配列 → NDJSON 変換 → `POST /_bulk`（一括インデックス）

---

## [v25.7.0] — 2026-06-25 — kafka Rune 実質化（「動く Rune」5 条件達成）

### Added
- `KafkaConn(String)` — Kafka ブローカー接続ラッパー型（`""` → `KAFKA_BOOTSTRAP_BROKERS` 環境変数 → `"localhost:9092"`）
- `Kafka.connect` / `produce` / `consume_one` / `consume_batch` / `create_topic`（5 関数、`KafkaConn` ベース）
- `Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw`（新規 VM primitives 3 件）— 既存 rskafka v0.6 再利用（追加 crate なし）
- `kafka_connect_sync` / `kafka_consume_batch_sync` / `kafka_create_topic_sync` ヘルパー（vm.rs）
- E0319 `UndeclaredStreamEffect` を `error_catalog.rs` に登録（checker.rs には v15.4.0 から存在）
- `examples/kafka_events_etl.fav` — イベント ETL デモ（PublishEvent / ConsumeEvents / EventsETL）
- `site/content/docs/runes/kafka.mdx` — 全 API ドキュメント（Redpanda セットアップ手順含む）

---

## [v25.6.0] — 2026-06-25 — dynamodb Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::DynamoDB` — 新エフェクト（AWS NoSQL KV 専用。E0323 エラーコード追加）
- `DynamoDB.connect` / `get_item` / `put_item` / `delete_item` / `query` / `scan` / `batch_write` / `transact_write`（8 関数）
- `DynamoDB.*_raw` VM primitives 8 件（`vm.rs`）— 既存 `aws_post` / SigV4 インフラ再利用（追加 crate なし）
- `get_dynamo_endpoint` / `json_val_to_dynamo_attr` / `json_to_dynamo_item` / `dynamo_attr_to_json` / `dynamo_item_to_plain_json` ヘルパー（JSON ↔ DynamoDB 属性型変換）
- `examples/dynamodb_session_store.fav` — セッションストア デモ（StoreSession / GetSession / DeleteSession）
- `site/content/docs/runes/dynamodb.mdx` — 全 API ドキュメント（JSON フォーマット・属性型変換説明含む）

---

## [v25.5.0] — 2026-06-25 — mongodb Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::MongoDB` — 新エフェクト（ドキュメント系 NoSQL 専用。E0322 エラーコード追加）
- `Mongo.connect` / `find` / `find_one` / `insert_one` / `insert_many` / `update_one` / `delete_one` / `aggregate`（8 関数）
- `Mongo.*_raw` VM primitives 8 件（`vm.rs`）— tokio `block_on` で async mongodb v3 API を同期化
- `mongodb = { version = "3", features = ["tokio-runtime"] }` を native-only 依存に追加
- `extract_mongo_db_name` / `mongo_bson_to_json` / `mongo_json_to_bson` ヘルパー（ObjectId → `{"$oid": "..."}` 変換対応）
- `examples/mongo_events_etl.fav` — イベント ETL デモ（LoadActiveEvents |> ArchiveEvent）
- `site/content/docs/runes/mongodb.mdx` — 全 API ドキュメント（JSON フォーマット・スコープ外説明含む）

---

## [v25.4.0] — 2026-06-25 — mysql Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::MySQL` — 新エフェクト（`!Postgres` とは独立した外部 MySQL 専用。E0321 エラーコード追加）
- `MySQL.connect` / `MySQL.query` / `MySQL.execute` / `MySQL.transaction_begin/commit/rollback`（6 関数）
- `MySQL.*_raw` VM primitives 6 件（`vm.rs`）— `mysql::prelude::Queryable` トレイト使用
- `mysql = { version = "24", default-features = false }` を native-only 依存に追加
- `examples/mysql_orders_etl.fav` — 注文 ETL デモ（LoadPendingOrders |> MarkProcessed）
- `site/content/docs/runes/mysql.mdx` — 全 API ドキュメント（Postgres との比較表含む）
- `json_to_mysql_value` / `mysql_value_to_json` ヘルパー関数（`vm.rs`）

### Notes
- `transaction_begin/commit/rollback` は VM 制約により各呼び出しで独立接続を使用（擬似実装）。原子性は非保証。v26.x で解決予定
- Postgres と同一シグネチャ（connect/query/execute/transaction）で API を統一。`impl DbConn for MySqlConn` は v26.x で対応予定

---

## [v25.3.0] — 2026-06-25 — redis Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Effect::Redis` — 新エフェクト（`!Cache` インメモリとは独立した外部 Redis 専用。E0320 エラーコード追加）
- `Redis.connect(url)` — RedisConn（接続 URL ラッパー）を返す
- `Redis.get / set / del / incr` — 基本 KV 操作
- `Redis.lpush / rpop` — リスト操作（キュー用途）
- `Redis.publish / subscribe_once` — Pub/Sub（subscribe_once は 30 秒タイムアウト付き 1 件受信）
- `examples/redis_rate_limiter.fav` — Redis を使ったレート制限 E2E デモ
- `v253000_tests`（7 件）: connect / get / set / incr / subscribe_once primitive 存在確認 + example + changelog + Effect::Redis

---

## [v25.2.0] — 2026-06-24 — s3 Rune 実質化（「動く Rune」5 条件達成）

### Added
- `S3.presign_url(bucket, key, ttl_secs)` — 署名付き URL 生成（GET 操作用、自前 SigV4 実装）
- `S3.stream_get(bucket, key)` — 大容量オブジェクトのストリーミング取得（現 v: get_object と同等）
- `examples/s3_csv_to_parquet.fav` — S3 CSV → Parquet 変換 E2E デモ（`import rune "aws"` 使用）
- `v252000_tests`（6 件）: presign_url / stream_get Rune + primitive 存在確認、example 確認、changelog 確認

---

## [v25.1.0] — 2026-06-24 — postgres Rune 実質化（「動く Rune」5 条件達成）

### Added
- `Postgres.connect(config)` — PgConfig（接続文字列ラッパー）から接続オブジェクト（PgConn）を返す
- `Postgres.execute_many(conn, sql, rows)` — バッチ実行（同一 SQL を複数行に適用）
- `Postgres.transaction(conn, fn)` — トランザクション（エラー時自動 ROLLBACK）
- `Postgres.Pool.create(config)` — PoolConfig から接続プールを作成（`pool_create_with_config_raw`）
- `Postgres.Pool.get(pool)` — プールから PgConn を取得
- `Postgres.Pool.release(pool, conn)` — PgConn をプールに返却
- `runes/postgres/db_conn.fav` — `DbConn` interface（query / execute / execute_many / transaction）
- `runes/postgres/types.fav` — `PgConfig` / `PgConn` / `PoolConfig` 型定義
- `examples/postgres_etl.fav` — E2E デモ（connect → execute_many → query → transaction）
- `v251000_tests`（6 件）: connect / execute_many / transaction / Pool.create 存在確認 + example + changelog

### Changed
- `runes/postgres/client.fav` — 上記 6 関数を追加（既存 `execute` / `query<T>` は後方互換として維持）
- `runes/postgres/postgres.fav` — `types` / `db_conn` / 新関数を re-export に追加
- `site/content/docs/runes/postgres.mdx` — 接続オブジェクト API セクションを追加

---

## [v25.0.0] — 2026-06-24 — Practical Self-Hosting マイルストーン宣言（v1.0 リリース候補）

### Added
- `MILESTONE.md` — Practical Self-Hosting 達成宣言ドキュメント（リポジトリルート）
- `site/content/docs/v1-release.mdx` — v1.0 リリースノート（v24.1〜v24.8 機能一覧）
- `v250000_tests`（5 件）: `milestone_md_has_selfhost_declaration` / `readme_mentions_v1_release` / `stability_md_exists` / `site_v1_release_page_exists` / `changelog_has_v25_0_0`

### Changed
- `README.md` — v25.0 / Practical Self-Hosting マイルストーン達成を追記
- `versions/roadmap-v20.1-v25.0.md` — v24.1〜v24.8 を「完了」、v25.0.0 を「宣言済み」に更新

### Milestone
- コンパイラ（compiler.fav）/ 型チェッカー（checker.fav）/ CLI（cli.fav）/ VM 仕様（vm.fav）がすべて Favnir で実装済み
- VM 実行基盤（バイトコードディスパッチ）のみ Rust で永続維持（設計上の意図）
- テスト数: 1974 件（前バージョン比 +5）

---

## [v24.8.0] — 2026-06-24 — `fav new` テンプレートギャラリー

### Added
- `TEMPLATE_GALLERY` 定数（4 テンプレート: etl-csv-to-db / api-gateway / lambda-scheduled / distributed-etl）
- `fav new --template etl-csv-to-db` — CSV → DB ETL スターター（pipeline.fav / fav.toml / README / CI）
- `fav new --template api-gateway` — HTTP API ゲートウェイスターター
- `fav new --template lambda-scheduled` — スケジュール実行ジョブスターター
- `fav new --template distributed-etl` — 分散並列 ETL スターター（par [A,B] |> Merge パターン）
- `site/content/docs/tools/templates.mdx` — テンプレートギャラリードキュメント

### Changed
- `try_cmd_new` エラーメッセージに 4 テンプレート名を追記

---

## [v24.7.0] — 2026-06-23 — ドキュメントサイト v2

### Added
- `site/content/learn/` チュートリアルセクション（getting-started / pipeline-basics / type-system）
- `site/content/cookbook/` レシピ集（etl-csv-to-db / api-gateway / parallel-pipeline）
- `site/app/packages/page.tsx` — Rune レジストリ静的一覧ページ（45 パッケージ）
- `site/content/docs/bench/index.mdx` — ベンチマーク履歴・fav bench コマンド解説
- `site/content/docs/spec/index.mdx` — 形式的仕様書・fav spec コマンド解説

### Changed
- サイト構成を learn / cookbook / spec / bench / packages の 5 軸に拡張

---

## [v24.6.0] — 2026-06-23 — セキュリティ審査（エフェクトシステム形式検証）

### Added
- W021 `pure_fn_calls_effectful` lint ルール — 純粋関数から副作用関数を呼び出す箇所を検出
- `SECURITY.md` — CVE 対応プロセス（security@favnir.dev、90日 responsible disclosure）
- `SECURITY_MODEL.md` — エフェクトシステムの形式的仕様（capability 公理 4 条 + 推論規則）
- `site/content/docs/tools/security.mdx` — セキュリティモデル解説ページ

### Notes
- W021 は `fn` 定義間の呼び出し関係のみ検出。`trf`/`flw` 対応は v24.7+ 予定
- TLA+/Coq による機械検証は v25.0 前後を目標

---

## [v24.5.0] — 2026-06-23 — Rune レジストリ成熟（公式パッケージ 50+）

### Added
- `fav search <query>` — 公式 Rune カタログを検索するトップレベルコマンド
- `OFFICIAL_CATALOG` — 50 パッケージを収録した組み込み公式カタログ（driver.rs）
- 15 新規 Rune スタブ（avro / orc / excel / xml / huggingface / scikit /
  gcs / pubsub / redis / mysql / mongodb / s3 / sqs / dynamodb / azure-servicebus）
- `site/content/docs/runes/catalog.mdx` — 全 50 Rune 公式カタログページ

### Notes
- 新規 Rune は v24.5.0 時点ではスタブ（rune.toml + .fav ヘッダー）。完全実装は v25.x 以降で個別に対応
- `fav search` は OFFICIAL_CATALOG（組み込み）を検索。ローカルインストール済み Rune の検索は `fav registry search <q>` を使用

---

## [v24.4.0] — 2026-06-23 — v1.0 後方互換性ポリシー確定

### Added
- `#[deprecated]` アノテーション — `fn` 定義に付与することで廃止予定を宣言できる
- W020 `deprecated_call` lint ルール — `#[deprecated]` 付き関数の呼び出しを `fav lint` で検出
- `STABILITY.md` — v1.x 後方互換ポリシー・v2.0 破壊的変更ポリシー・SemVer 準拠宣言

### Notes
- `#[deprecated]` は `fn` にのみ対応（`trf`・`flw` は v24.7+ 予定）
- `impl` ブロック内 `fn` への `#[deprecated]` は v24.7+ 予定
- `--legacy` フラグは v2.0 まで維持（STABILITY.md 参照）

---

## [v24.3.0] — 2026-06-23 — 継続的パフォーマンス回帰検知

### Added
- `driver::cmd_bench_compare(baseline_json, current_json, threshold, emit_md) -> (bool, String)` — ベンチマーク JSON 比較の公開 API
- `fav bench --baseline <path> --current <path> [--threshold N] [--emit-md]` CLI サブコマンド（既存 `fav bench` の `--baseline` 検出で自動ディスパッチ）
- `benchmarks/latest.json` — CI 出力テンプレート

### Changed
- `.github/workflows/bench.yml` — baseline を `v24.2.0.json` に更新、threshold を 5% に変更、回帰時 CI fail を有効化（`|| exit 1`）
- `benchmarks/v24.2.0.json` — `metrics` を数値のみに修正（`stage4_deferred` 削除）

### Notes
- 回帰判定式: `(current - baseline) / baseline * 100 > threshold`（増加が劣化）
- `bench.favnir.dev` グラフ公開は v24.7（ドキュメントサイト v2）と同時対応予定

---

## [v24.2.0] — 2026-06-23 — 4-Stage Bootstrap 検証

### Added
- `fav/tests/bootstrap/` — Bootstrap 検証用 fixture 5 件（hello / arithmetic / pattern_match / list_ops / closures）
- `v242000_tests` — Bootstrap fixture コンパイルテスト 7 件（カウント済）
- `bootstrap_stage1_stage3_hello_match` / `bootstrap_stage1_stage3_arithmetic_match` — Stage 1/3 bytecode 比較（`#[ignore]`、低速）

### Notes
- Stage 4（vm.fav + compiler_artifact → bytecode_C）は vm.fav Phase 6（ユーザー定義関数ディスパッチ）完了後に追加予定
- `bytecode_A == bytecode_B` 検証は `cargo test bootstrap_stage1 -- --ignored` で実行
- `type T = A | B` 形式のフィールドなしバリアントと `[h | t]` リストパターンはパーサー非対応のため、pattern_match.fav を Option マッチ、list_ops.fav を多引数算術関数に変更
- 実際のテスト件数: 1940（version_is_24_1_0 削除 -1、新規 +7 = 純増 +6）

---

## [v24.1.0] — 2026-06-23 — 形式的仕様書生成（fav spec）

### Added
- `driver::cmd_spec(format: &str) -> String` — Favnir 言語仕様書を Markdown / HTML で生成する公開 API
- `fav spec [--format markdown|html]` CLI サブコマンド — 型システム・opcode・エフェクト・パターンマッチ規則を仕様書として出力

### Notes
- 仕様書は 4 セクション構成: 型システム（HM 推論規則）/ opcode 動作仕様（31 opcode）/ エフェクトシステム意味論 / パターンマッチ網羅性
- HTML 変換は既存 `md_to_html`（v21.7.0 実装）を再利用
- 既知の制限: HTML 出力のテーブル行は `<p>` タグとして出力（`<table>` 変換は Phase 2 以降）

---

## [v24.0.0] — 2026-06-23 — VM in Favnir マイルストーン宣言

### Added
- `driver::run_with_vm(vm_src, bytecode_hex, globals_entries)` — vm.fav 経由でバイトコードを実行する公開 API
- `fav run --vm <path> --hex <hex>` CLI フラグ — 端末から vm.fav 経由でバイトコードを直接実行

### Notes
- VM in Favnir マイルストーン宣言（v23.1〜v24.0 の達成を宣言）
  - v23.1: Bytes 型 / v23.2: ビット演算 / v23.3: Mut<T>
  - v23.4〜v23.8: vm.fav Phase 1〜5（デコード・実行ループ・制御フロー・builtin・GetField）
- ロードマップ完了条件 1〜3・5 を達成；条件 4（500件超テスト）は Phase 6 以降

---

## [v23.8.0] — 2026-06-22 — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）

### Added
- `vm.fav` Phase 5: GetField・多引数 Call・vmval_display
  - `fn collect_args_rec(stack: Int, n: Int, acc: Int) -> Result<Int, String>` 追加
  - `fn collect_args(stack: Int, n: Int) -> Result<Int, String>` 追加
  - `GetField(idx)` オペコード: namespace VMStr + globals[idx]=field VMStr → push "ns.field" VMStr
  - `Call(argc)` ハンドラを `collect_args` 利用の汎用実装に置換（任意の argc に対応）
  - `fn vmval_display(v: VMVal) -> String` 追加（ユーザー向け表示: VMStr は引用符なし）
  - `call_builtin` に `"String.concat"` 追加（2 引数 builtin・collect_args 引数順実証）

### Notes
- `LoadGlobal + GetField + Call(N)` シーケンス完成: 任意の builtin 呼び出しチェーンが vm.fav 上で動作
- `fav run --vm=<path>` CLI フラグは v24.0 で実装予定

---

## [v23.7.0] — 2026-06-22 — vm.fav Phase 4（stdlib・builtin 呼び出し）

### Added
- `vm.fav` Phase 4: stdlib・builtin 呼び出し
  - `VMVal` に `VMStr(String)` バリアントを追加
  - `fn call_builtin(name: String, args: Int) -> Result<VMVal, String>` 実装（4 builtin: Int.to_string / String.length / String.trim / Math.abs）
  - `LoadGlobal(idx)` オペコード: globals マップから値を lookup してスタックに push
  - `Call(0)` / `Call(1)` オペコード: builtin ディスパッチ（Favnir ↔ Rust の永続的境界）
  - `fn vm_run_named(bytecode: Bytes, globals: Int) -> Result<VMVal, String>` 追加

### Changed
- `fn vm_execute` シグネチャ: `(bytecode, stack, locals, pc)` → `(bytecode, stack, locals, globals, pc)`
- `fn vm_run` が空 globals マップを生成するよう更新

---

## [v23.6.0] — 2026-06-22 — vm.fav Phase 3（制御フロー・ローカル変数）

### Added
- `vm.fav` Phase 3: 制御フロー・ローカル変数
  - `vm_execute` に `locals: Int` パラメータを追加（MutMap による単一フレームのローカル変数）
  - `vm_run` が `Mut.map()` でローカル変数マップを生成
  - 新オペコード 12 件: Jump / JumpIfFalse / LoadLocal / StoreLocal / Ne / Lt / Le / Gt / Ge / And / Or / Div

### Changed
- `fn vm_execute` シグネチャ: `(bytecode, stack, pc)` → `(bytecode, stack, locals, pc)`

---

## [v23.5.0] — 2026-06-22 — vm.fav Phase 2（スタックベース実行ループ）

### Added
- `vm.fav` Phase 2: スタックベース実行ループ
  - `type VMVal` — スタック値 sum type（VMInt / VMBool / VMUnit）
  - `fn vmval_to_string` — デバッグ用文字列化
  - `fn vm_execute(bytecode: Bytes, stack: Int, pc: Int) -> Result<VMVal, String>` — 実行ループ（再帰）
    - 対応オペコード 11 件: ConstUnit / ConstTrue / ConstFalse / Const(n) / Pop / Dup / Return / Add / Sub / Mul / Eq
  - `fn vm_run(bytecode: Bytes) -> Result<VMVal, String>` — エントリポイント

---

## [v23.4.0] — 2026-06-22 — vm.fav Phase 1（バイトコードデコード）

### Added
- `fav/self/vm.fav` — Favnir セルフホスト VM Phase 1（バイトコードデコード）
  - `type Opcode` — 27 バリアント（Const〜Unknown）定義
  - `type DecodeResult` — `{ op: Opcode, next_pc: Int }` レコード型
  - `fn decode_byte_no_operand` / `fn decode_byte_with_u16_le` ヘルパー
  - `fn decode_opcode` — メインデコードエントリポイント
  - `fn opcode_to_string` — デバッグ用文字列変換
- `Bytes.read_u16_le` / `Bytes.read_u24_le` — リトルエンディアン Bytes 読み取り primitive（vm.rs）

---

## [v23.3.0] — 2026-06-22 — 可変コレクション `Mut<T>`

### Added
- `Mut.list()` / `Mut.map()` — 可変コレクション生成（`VMValue::MutList(u64)` / `VMValue::MutMap(u64)` opaque handle）
- `Mut.push` / `Mut.pop` / `Mut.peek` / `Mut.len` / `Mut.set` / `Mut.get` / `Mut.delete` / `Mut.has`
- checker builtin_ret_ty に Mut 5 エントリ追加、compiler builtins リストに `"Mut"` 追加
- 1902 テスト合格

---

## [v23.2.0] — 2026-06-21 — ビット演算

### Added
- `Int.bit_and` / `Int.bit_or` / `Int.bit_xor` / `Int.bit_not` / `Int.shift_left` / `Int.shift_right`
- 16 進数リテラル `0xFF`（lexer `lex_number()` 拡張）
- 1898 テスト合格

---

## [v23.1.0] — 2026-06-21 — `Bytes` 型

### Added
- `VMValue::Bytes(u64)` / `HeapVal::Bytes(u64)` — バイト列 opaque handle（NaN-boxing 準拠）
- `Bytes.from_hex` / `Bytes.to_hex` / `Bytes.from_list` / `Bytes.to_list` / `Bytes.length` / `Bytes.get` / `Bytes.set` / `Bytes.slice` / `Bytes.concat` / `Bytes.read_u8` / `Bytes.read_u16` / `Bytes.read_u32` / `Bytes.write_file` / `Bytes.read_file`
- checker namespace + compiler builtins に `"Bytes"` 追加
- 1894 テスト合格

---

## [v23.0.0] — 2026-06-21 — Distributed Scale マイルストーン宣言

### Added
- Distributed Scale マイルストーン宣言（v22.0.0〜v22.8.0 の実装を集大成）
- `benchmarks/v23.0.0.json` 作成（1887 テスト合格）

---

## [v22.8.0] — 2026-06-21 — `fav deploy` 強化（ECS / K8s / Fly.io 対応）

### Added
- `DeployConfig` 拡張（`platform` フィールド: `"ecs"` / `"k8s"` / `"fly"`）
- `cmd_deploy_ecs` / `cmd_deploy_k8s` / `cmd_deploy_fly` — プラットフォーム別デプロイ
- 1883 テスト合格

---

## [v22.7.0] — 2026-06-21 — OpenTelemetry 統合

### Added
- `fav/src/otel.rs` — OTel スパン生成モジュール新規作成
- `SeqStageEnter` / `SeqStageExit` — stage 境界での自動 span 生成
- `--otel-endpoint` CLI フラグ、`!Otel` エフェクト追加
- 1879 テスト合格

---

## [v22.6.0] — 2026-06-21 — SLA 宣言（タイムアウト・リトライ・サーキットブレーカー）

### Added
- `TimeoutAnnotation` / `RetryAnnotation` / `CircuitBreakerAnnotation` struct（ast.rs）
- `@timeout(ms)` / `@retry(n)` / `@circuit_breaker(threshold)` アノテーション構文
- 1872 テスト合格

---

## [v22.5.0] — 2026-06-21 — Pipeline Orchestration（DAG スケジューリング）

### Added
- `TokenKind::Pipeline` キーワード追加（lexer.rs）
- `pipeline` 宣言構文 — 複数の `seq` / `par` ブロックを DAG として定義
- 1864 テスト合格

---

## [v22.4.0] — 2026-06-21 — Event-driven Pipeline（イベントトリガー）

### Added
- `TriggerAnnotation` struct — `@trigger(kind)` アノテーション、`FlwDef.trigger` フィールド追加
- `!Event` エフェクト追加、`Trigger.sqs` / `Trigger.http` / `Trigger.schedule` Rune
- 1860 テスト合格

---

## [v22.3.0] — 2026-06-21 — Pipeline State Rune（分散状態管理）

### Added
- `Effect::PipelineState` 追加（ast.rs）
- `PipelineState` Rune（`get` / `set` / `delete` / `list_keys` primitives）
- 1855 テスト合格

---

## [v22.2.0] — 2026-06-21 — Distributed `par`（複数 Worker への分散）

### Added
- `FlwStep::ParDistributed { stages, workers }` — 複数 Worker への分散実行
- `--workers N` CLI フラグ
- 1850 テスト合格

---

## [v22.1.0] — 2026-06-21 — Checkpoint / Resume（パイプライン永続化）

### Added
- `TrfDef.checkpoint: bool` フィールド追加、`@checkpoint` アノテーション構文
- checkpoint 書き込み / 読み取り VM primitive（`.fav-checkpoint/` ディレクトリ）
- `fav run --resume` フラグ — チェックポイントから再開
- 1846 テスト合格

---

## [v22.0.0] — 2026-06-21 — Developer Tooling Complete マイルストーン宣言

### Added
- Developer Tooling Complete マイルストーン宣言（v21.0.0〜v21.8.0 の実装を集大成）
- `benchmarks/v22.0.0.json` 作成、README Developer Tooling セクション追加
- 1842 テスト合格

---

## [v21.8.0] — 2026-06-20 — `fav migrate` 強化

### Added
- `migrate_fav_toml_source` — `fav.toml` マイグレーション（v13→v14 等）
- `fav migrate --from v13 --to v14` — バージョン指定移行、`fav migrate --check` — 確認モード
- 1831+ テスト合格

---

## [v21.7.0] — 2026-06-20 — `fav doc` サイト生成（docsite）

### Added
- `fav doc --format site src/ --out docs/` — 静的 HTML ドキュメントサイト生成（ダークテーマ）
- `fav doc --serve src/` — ローカルプレビューサーバー（`TcpListener`、デフォルト port 8080）
- `html_escape` / `inline_md` / `md_to_html`、`site/content/docs/tools/doc-site.mdx` 新規作成
- 1831 テスト合格

---

## [v21.6.0] — 2026-06-20 — Playground v2（共有・テンプレート・ライブ統計）

### Added
- `site/lib/share-url.ts` — gzip+base64url URL エンコード/デコード
- `site/lib/playground-templates.ts` — 6 テンプレート、`site/app/playground/share-api.ts` — Lambda API クライアント
- Playground 共有ボタン・テンプレートドロップダウン・実行統計・URL 復元
- `infra/share/` — AWS Lambda 共有 API（Terraform + handlers/share.js）
- 1824 テスト合格

---

## [v21.5.0] — 2026-06-20 — LSP コードアクション強化

### Added
- `CheckedDoc.program: Option<Program>` フィールド追加（document_store.rs）
- `lsp/references.rs` / `lsp/rename.rs` / `lsp/code_action.rs` 新規作成
- LSP capabilities に `codeActionProvider` / `renameProvider` / `referencesProvider` 追加
- 1817 テスト合格

---

## [v21.4.0] — 2026-06-20 — `fav lint` 強化（W010〜W019）

### Added
- W010〜W019 lint ルール追加（stage_too_large / effectless_io_call / unused_type / map_filter_chain / redundant_result_ok / rebind_in_block / wildcard_only_match / deep_nesting / magic_number / string_concat_chain）
- `partial_flw_warnings` を W020 に改名（W011 との衝突回避）
- 1806 テスト合格

---


## [v21.3.0] — 2026-06-20 — テストカバレッジ HTML / LCOV 出力

`fav test --coverage --html` で HTML カバレッジレポート、
`--lcov` で LCOV 形式ファイルを生成できるようになった。

### Added
- `fav test --coverage --html --coverage-report <dir>` — HTML レポート（index.html）生成（行ハイライト・ファイル一覧テーブル付き）
- `fav test --coverage --lcov --coverage-report <dir>` — LCOV 形式（lcov.info）出力（coveralls / codecov 連携用）
- コンソールサマリーをファイル別 ✓/✗ 形式に改善（`Coverage: XX.X% (N/M lines)`）
- `fav/src/coverage/mod.rs` 新規作成（`CoverageFileStat` / `CoverageSummary` / `generate_coverage_html` / `generate_lcov` / `format_coverage_summary_console`）
- `is_executable_line` を `pub(crate)` に昇格（coverage モジュールからの利用を可能に）
- `site/content/docs/tools/coverage.mdx` — 使い方ドキュメント

---

## [v21.2.0] — 2026-06-20 — fav explain 可視化強化

`fav explain --lineage` の出力形式を Mermaid / D2 に拡張。
GitHub / Notion / Obsidian でそのままレンダリングできる依存グラフを生成できる。

### Added
- `fav explain --lineage --format mermaid` — Mermaid `flowchart LR` 形式でパイプライングラフを stdout に出力
- `fav explain --lineage --format d2` — D2 diagram 形式でパイプライングラフを stdout に出力
- `render_lineage_mermaid(report: &LineageReport) -> String` を `lineage.rs` に追加
- `render_lineage_d2(report: &LineageReport) -> String` を `lineage.rs` に追加
- `sanitize_mermaid_id` ヘルパー（ノード ID を英数字 + `_` のみに変換）
- `site/content/docs/tools/lineage.mdx` — 可視化出力の使い方ドキュメント（4形式の例含む）

---

## [v21.1.0] — 2026-06-20 — DAP デバッガー

VS Code / Neovim / Emacs から Favnir パイプラインをステップ実行できる
DAP（Debug Adapter Protocol）サーバーを実装。

### Added
- `fav dap [--port 5678]` — DAP サーバー起動コマンド。TCP ポートでリッスンし DAP クライアントの接続を待ち受ける
- `fav run --debug [--dap-port N] <file>` — デバッグモード実行。VM に DAP フックを挿入して実行する
- `fav/src/dap/` モジュール（`protocol` / `session` / `adapter` / `server`）
- DAP サポートリクエスト: `initialize` / `launch` / `setBreakpoints` / `configurationDone` / `threads` / `stackTrace` / `scopes` / `variables` / `next` / `stepIn` / `continue` / `disconnect`（計12コマンド）
- `VM::debug_mode` / `VM::dap_adapter` フィールド（`--debug` なし実行はブランチが最適化で除去されゼロコスト）
- `DapSession.event_queue` — VM フックから `stopped` イベントを DAP クライアントへプッシュする仕組み
- VS Code `launch.json` 設定例（`site/content/docs/tools/dap.mdx`）

---

## [v21.0.0] — 2026-06-20 — Runtime Excellence マイルストーン宣言

v20.1.0〜v20.8.0 で達成した VM 実行性能最適化の集大成。
全 5 SLO（`cold_start_precompiled < 10ms` / `csv_throughput > 1 GB/s` / `tight_loop < 30ms` /
`record_transform < 80ms` / `duckdb_query pushdown 委譲`）を達成。

### Milestone
- **SLO 達成**: cold_start 18ms → **8ms**、csv 340 MB/s → **1.2 GB/s**、tight_loop 85ms → **26ms**、record_transform 210ms → **72ms**、duckdb_query（集計）VM 実行 → **DuckDB pushdown（3ms）**
- `benchmarks/v21.0.0.json` — SLO 達成値記録
- `site/content/docs/performance/runtime-excellence.mdx` — マイルストーン概要ページ
- `site/content/docs/performance/nan-boxing.mdx` / `pushdown.mdx` — 各最適化解説

---

## [v20.8.0] — 2026-06-20 — DB コネクションプール統合

### Added
- `PgPool` — `tokio_postgres::Client` の Vec プール（`fav/src/backend/pg_pool.rs`）
- `PgPoolStats` struct — `borrow_count` / `miss_count` / `return_count` / `error_count` / `idle_count`
- `pg_pool_runtime()` — プール専用長寿命 tokio runtime（`new_multi_thread`、`worker_threads(2)`）
- `VMValue::PgPool(u64)` — opaque handle（`HeapVal::PgPool` と対応、exhaustive match 全 6 箇所更新）
- Primitives: `Postgres.Pool.create` / `query` / `execute` / `stats` / `close`（`vm_call_builtin` に追加）
- `fav.toml` の `[postgres]` セクションに `pool_size` / `min_idle` フィールド追加

### Performance（期待値）
- `pg_stage_first_call_ms`: -45ms 削減（プール再利用時の接続コストゼロ化）
- `pg_pipeline_10stage_ms`: +5〜10x 改善（10 stage × 接続確立 → プール再利用）
- `pg_pool_reuse_rate_pct`: >95%（実 DB 環境でのプール hit 率）

---

## [v20.7.0] — 2026-06-20 — Arena アロケータ（GC なし高速アロケーション）

### Added
- `ChunkArena` struct（`fav/src/arena/mod.rs`）— `bumpalo::Bump` + `Vec<Vec<VMValue>>` pool を組み合わせたアリーナアロケータ
  - `acquire(capacity)` — pool hit 時は既存 Vec を再利用、miss 時のみ `Vec::with_capacity` を呼ぶ
  - `release(buf)` — Vec を pool に返却し `peak_capacity` を更新
  - `end_chunk(result_val, out)` — chunk 結果を out に追加し `bump.reset()` でチャンク境界をリセット
  - `start_chunk()` — 将来の文字列インターン用マーカー（現在 no-op）
  - `reset_bump()` — chunk ループ後の一括リセット
- `ArenaStats` struct — `acquire_count` / `alloc_count` / `reset_count` / `peak_capacity` フィールド
- `ChunkArena::new_with_enabled(bool)` — テスト用コンストラクタ（`std::env::set_var` 不要）
- `Arena.stats() -> Record` VM primitive — `call_builtin`（`&mut self` メソッド）に追加
  - 返却フィールド: `acquire_count`, `alloc_count`, `reset_count`, `peak_capacity`（すべて `Int`）
  - WASM では `err_vm("Arena.stats: not supported on wasm32")` を返す
- `bumpalo = "3"` 依存クレート追加（`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`）
- `FAV_ARENA_ENABLED=0` 環境変数で arena を無効化可能（デバッグ用）

### Changed
- `__streaming_pipeline` — chunk ごとの `Vec::new()` を `ChunkArena::acquire/end_chunk` で置き換え（malloc/free 削減）
  - `#[cfg(not(target_arch = "wasm32"))]` で arena パス、`#[cfg(target_arch = "wasm32")]` で従来パスを使用
- `FavList::to_vec` の可視性を `fn` → `pub(crate) fn` に変更（`arena/mod.rs` からアクセスするため）
- `is_known_builtin_namespace` に `"Arena"` を追加（`vm.rs`）
- `compiler.rs` builtin 一覧に `"Arena.stats"` を追加
- `checker.rs` builtin namespace 一覧に `"Arena"` を追加

### Performance（期待値、v20.6.0 比）
- `record_transform_1m_ms`: +20〜40% 改善（ストリーミングパイプライン Vec pool 再利用）
- `streaming_peak_memory_mb`: -20% 削減（chunk 境界での Vec 一括返却）
- `chunk_alloc_overhead_ms`: +2〜3x 改善（malloc/free ラウンドトリップ削減）
- 実測は `benchmarks/v20.7.0.json` 参照

---

## [v20.6.0] — 2026-06-20 — io_uring 非同期 I/O（Linux）

### Added
- `IO.read_files_batch(paths: List<String>) -> List<String>` — 複数ファイル並列読み込み
  - Linux（カーネル 5.1+）: `tokio-uring` (io_uring) によるゼロコンテキストスイッチ非同期 I/O
  - Windows / macOS: `rayon` 並列 `read_to_string` フォールバック
  - WASM: `err_vm` を返す（非対応、`#[cfg(target_arch = "wasm32")]` ガード）
  - 結果は入力パスと同じ順序を保証（rayon も順序を保持）
  - いずれか 1 ファイル失敗で全体が `Err` を返す（fail-fast）
- `read_files_batch_impl` ヘルパー関数（`pub(crate)`）— Linux / 非Linux で cfg 分岐
- `read_one_uring` async fn（Linux のみ）— `tokio_uring::fs::File::read_at` でバッファ所有権移転
- `tokio-uring = "0.4"` 依存クレート追加（`[target.'cfg(target_os = "linux")'.dependencies]`）
- `futures = "0.3"` 依存クレート追加（`try_join_all` による並列 await）

### Performance（Linux 本番環境、期待値）
- `io_batch_100_files_ms`: +2〜4x 改善（io_uring 並列 vs 逐次 read）
- `io_batch_1000_files_ms`: +3〜5x 改善
- `io_db_file_mixed_ms`: +1.5〜2x 改善
- 実測は `benchmarks/v20.6.0.json` 参照

---

## [v20.5.0] — 2026-06-20 — mmap + SIMD CSV パーサー

### Added
- `ArrowBatch.from_csv(path: String) -> ArrowBatch` — mmap ゼロコピー + arrow-csv 列指向パース
  - `memmap2::MmapOptions` でファイルをゼロコピーマッピング（`read()` syscall 削減）
  - `arrow::csv::reader::Format::infer_schema` で先頭 1000 行からスキーマ自動推論
  - `arrow::csv::ReaderBuilder` で列指向 CSV パース（batch_size 65536）
  - 複数チャンクは `arrow::compute::concat_batches` で単一 `RecordBatch` に結合
  - WASM では常に `Err`（`#[cfg(not(target_arch = "wasm32"))]`）
- `read_csv_mmap` ヘルパー関数（`pub(crate)`）— v20.4.0 の DuckDB プッシュダウンと自動連携
- `memmap2 = "0.9"` 依存クレート追加（native-only）

### Changed
- `arrow = { version = "52", features = ["ipc", "csv"] }` — `"csv"` feature 追加

### Performance
- `csv_10gb_throughput_mb_s`: +3〜5x 改善（期待値: > 1 GB/s）
- `peak_memory_csv_1gb_mb`: -40% 削減（中間 `Vec<String>` アロケーション排除）
- `csv_row_alloc_1m_ms`: +2〜3x 改善（行単位 `HashMap` 生成の排除）
- 実測は `benchmarks/v20.5.0.json` 参照

---

## [v20.4.0] — 2026-06-19 — DuckDB プッシュダウン最適化パス

### Added
- `fav/src/pushdown/` モジュール — コンパイル時 AST パターン検出 + SQL 生成
  - `mod.rs`: `FilterExpr / CmpOp / SqlLiteral / PushdownOp / PushdownPlan` 型定義、`detect_pushdown` エントリポイント
  - `pattern.rs`: `List.filter / map / group_by / sum_by / length` パターンマッチャー（5 種）
  - `sql_builder.rs`: `?pushdown_table?` プレースホルダーを使った SQL テンプレート生成
- `__duckdb_push` VM ビルトイン — ArrowBatch 入力時に DuckDB へ SQL を委譲
  - 非 ArrowBatch 入力またはクエリ失敗時は元のステージ関数にフォールバック
  - WASM では常にフォールバック（`#[cfg(not(target_arch = "wasm32"))]`）
- `fav run --explain-pushdown` フラグ — プッシュダウン適用状況を stderr に出力
- `PUSHDOWN_EXPLAIN_ENABLED` / `PUSHDOWN_LOG` thread-local（ログ管理）

### Changed
- `Item::TrfDef` コンパイルアーム — `detect_pushdown` 呼び出しを統合
  - プッシュダウン対象ステージは自動的にラッパー関数に変換（元ステージはフォールバック）
  - 非対象ステージは従来通りコンパイル（変更なし）

### Performance
- DuckDB 委譲成功時: `duckdb_query_sum_1m_ms` が v20.3.0 比 +10x 改善（期待値）
- プッシュダウン非対象時: オーバーヘッド < 1μs（detect_pushdown は AST 解析のみ）
- 実測は `benchmarks/v20.4.0.json` 参照

---

## [v20.3.0] — 2026-06-19 — NaN-boxing（VMValue の圧縮）

### Changed
- `VMValue` enum（32〜40 bytes/値）を `NanVal`（8 bytes/値）に置き換え
  - Int/Bool/Float/Unit はインライン格納（ヒープ割り当て不要）
  - Str/List/Record/その他ヒープ型は `Arc<T>` 経由でポインタ格納
  - `VMStream` / `FavList` 内部は v20.3.0 スコープ外（将来最適化）
- `VMValue` / `VMStream` / `FavList` を `pub(crate)` に昇格（crate 内参照のため）

### Added
- `fav/src/backend/nan_val.rs` — `NanVal` 型、8 タグ定数、encode/decode、Clone/Drop（Arc refcount 管理）
- `fav/src/backend/heap_val.rs` — `HeapVal` enum（Variant/Closure/Stream/BigInt 等）
- `fav run --legacy-value-repr` — 旧 VMValue 表現へのフォールバックフラグ（v21 以降削除予定）

### Performance
- `tight_loop_10m_iter`: スタックサイズ 40 bytes → 8 bytes によりキャッシュヒット率改善
- `record_transform_1m`: 同上（実測は `benchmarks/v20.3.0.json` 参照）

---

## [v20.2.0] — 2026-06-19 — スーパー命令（Superinstruction）

### Added
- `Opcode::AddLL / SubLL / MulLL / AddLC / SubLC / LeLC / LtLC / EqLC / GetFieldL / MoveLocal`
  (0xA0〜0xA9) — IR レベルスーパー命令 10 種
- `emit_expr / emit_stmt` が Local×Local・Local×Int リテラルのパターンで自動融合
- `GetFieldL` が `FieldAccess(Local(a), field)` を 6→5 bytes に圧縮
- `MoveLocal` が `Bind(dst, Local(src))` を 6→5 bytes に圧縮

### Performance
- `tight_loop_10m_iter`: ディスパッチ回数削減（+20〜30% 期待）
- `record_transform_1m`: フィールドアクセスパターン改善（+10〜15% 期待）

---

## [v20.1.0] — 2026-06-18 — ベンチマーク基盤整備

### Added
- `benchmarks/suite/` に 8 計測スクリプトを追加（01_cold_start.sh〜08_concurrent_stages.fav）
- `benchmarks/compare.fav` — ベースライン比較ツール（threshold 超えで非ゼロ終了）
- `.github/workflows/bench.yml` — master push ごとに自動計測・回帰検出
- `benchmarks/v20.0.0.json` — v20.0.0 ベースライン参考値（CI が実測値で更新）

---

## [v20.0.0] — 2026-06-17 — Production Performance マイルストーン宣言

### Added
- v19.x シリーズ集大成：遅延評価パイプライン / AOT コンパイル / インクリメンタルコンパイル / 並列コンパイル / Apache Arrow 統合 / WASM 最適化 / 事前コンパイル `.favc` / フレームグラフプロファイリングが揃い Production Performance を宣言
- `benchmarks/` ディレクトリ（`10gb_csv.fav` / `lambda_coldstart.sh` / `results.md`）
- `site/content/docs/performance/` ドキュメント（6 ファイル）
- `CHANGELOG.md` / `README.md` 全面更新（v19.1.0〜v20.0.0）

### Internal
- Cargo.toml version: `20.0.0`
- `v200000_tests`: 5 件追加

---

## [v19.8.0] — 2026-06-17 — プロファイリング強化（フレームグラフ）

### Added
- `fav profile --format=flamegraph` — `inferno` crate による SVG フレームグラフ生成
- `fav profile --format=text` — HOT PATH マーカー付きテキストレポート
- `fav profile --format=json` — `pct` フィールド付き JSON 出力
- `--runs=N` — N 回実行の平均プロファイル
- `--stage=<name>` — 特定 stage のみ表示
- `--out=<path>` — 出力先パス指定（flamegraph 向け）
- `site/content/docs/tools/profiling.mdx` 新規作成

### Internal
- `fav/Cargo.toml`: `inferno = "0.11"` を native-only deps に追加
- `src/profiler/` モジュール新規作成（`collector.rs` / `flamegraph.rs` / `report.rs`）
- `src/driver.rs`: `cmd_profile` シグネチャ拡張
- Cargo.toml version: `19.8.0`
- `v198000_tests`: 5 件追加

---

## [v19.7.0] — 2026-06-17 — 事前コンパイル（`.favc`）

### Added
- `fav compile <src.fav>` — `.favc` バイナリアーティファクト生成（SHA-256 ハッシュ + タイムスタンプ埋め込み）
- `fav run --precompiled <src.favc>` — 型チェック・コンパイルなしで直接実行（Lambda コールドスタート削減）
- `FavcMeta` 構造体（`source_hash` / `compiled_at` / `compiler_ver`）META セクション
- `site/content/docs/tools/precompiled.mdx` 新規作成

### Internal
- `src/backend/artifact.rs`: `FavcMeta` + `write_meta_section` / `read_meta_section`
- `src/driver.rs`: `cmd_compile` / `cmd_compile_to_bytes` / `cmd_run_precompiled`
- `src/main.rs`: `Some("compile")` ブランチ + `--precompiled` フラグ
- Cargo.toml version: `19.7.0`
- `v197000_tests`: 5 件追加

---

## [v19.6.0] — 2026-06-17 — WASM 最適化

### Added
- WASM バイナリサイズ削減（デッドコード除去・未使用 import 削減）
- WASM ビルドプロセス改善
- `site/content/docs/performance/wasm.mdx` 新規作成

### Internal
- Cargo.toml version: `19.6.0`
- `v196000_tests`: 5 件追加

---

## [v19.5.0] — 2026-06-17 — Apache Arrow 統合

### Added
- `VMValue::ArrowBatch(u64)` — opaque Arrow RecordBatch ハンドル
- `ArrowBatch.from_list` / `ArrowBatch.to_list` — VMValue List との相互変換
- `ArrowBatch.write_parquet` / `ArrowBatch.read_parquet` — Parquet ファイル I/O
- `#[arrow]` stage アノテーション（Arrow バッチパイプライン最適化）
- `site/content/docs/runes/arrow.mdx` 新規作成

### Internal
- `src/vm.rs`: `ARROW_BATCHES` thread-local + Arrow primitives
- `arrow = { version = "52", features = ["ipc"] }` / `parquet = "52"` を native-only deps に追加
- Cargo.toml version: `19.5.0`
- `v195000_tests`: 5 件追加

---

## [v19.4.0] — 2026-06-17 — 並列コンパイル

### Added
- `fav build --parallel` — Rayon + petgraph によるトポロジカル並列コンパイル
- `src/parallel/` モジュール（`topo.rs` / `compiler.rs`）

### Internal
- `rayon = "1"` / `petgraph = "0.6"` を native-only deps に追加
- Cargo.toml version: `19.4.0`
- `v194000_tests`: 5 件追加

---

## [v19.3.0] — 2026-06-17 — インクリメンタルコンパイル

### Added
- SHA-256 フィンガープリントベースのインクリメンタルコンパイル
- `.fav_cache/` ディレクトリへのアーティファクトキャッシュ
- `FAV_NO_CACHE` / `FAV_EXPLAIN_CACHE` / `FAV_CACHE_DIR` 環境変数

### Internal
- `src/incremental/` モジュール（`fingerprint.rs` / `dep_graph.rs` / `cache.rs`）
- Cargo.toml version: `19.3.0`
- `v193000_tests`: 5 件追加

---

## [v19.2.0] — 2026-06-17 — AOT コンパイル（Cranelift バックエンド）

### Added
- `fav build --target native` — Cranelift バックエンドによるネイティブバイナリ生成
- `src/backend/cranelift_aot.rs` — `CraneliftBackend::compile_to_binary`

### Internal
- `cranelift-codegen / cranelift-frontend / cranelift-module / cranelift-object / cranelift-native 0.117` を native-only deps に追加
- Cargo.toml version: `19.2.0`
- `v192000_tests`: 5 件追加

---

## [v19.1.0] — 2026-06-17 — 遅延評価パイプライン（`#[streaming]`）

### Added
- `#[streaming(chunk_size = N)]` / `#[streaming]` stage アノテーション — 定常メモリで大規模データを処理
- `#[stateful]` アノテーション — チャンク間状態保持
- `compile_streaming_pipeline` — chunk 単位の VM opcode 生成

### Internal
- `src/vm.rs`: `__streaming_pipeline` builtin ハンドラ追加
- Cargo.toml version: `19.1.0`
- `v191000_tests`: 5 件追加

---

## [v19.0.0] — 2026-06-16 — Type System Maturity マイルストーン宣言

### Added
- v18.x シリーズ集大成：エフェクト推論 / 行多相 / Refinement Types / スキーマ型 / 線形型 / 共変・反変アノテーション / Const Generics / 型駆動 API 生成が揃い Type System Maturity を宣言
- `CHANGELOG.md` / `README.md` 全面更新（v18.1.0〜v19.0.0）

### Internal
- Cargo.toml version: `19.0.0`
- `v190000_tests`: 5 件追加

---

## [v18.8.0] — 2026-06-16 — 型駆動 API 生成

### Added
- `#[api(method = "GET", path = "/users/:id")]` アノテーション構文
- `fav generate api` — OpenAPI 3.0 JSON/YAML と GraphQL SDL の自動生成
- `fav api-serve` — 開発用 HTTP サーバー（TcpListener ベース）
- `site/content/docs/api/generate.mdx` / `serve.mdx` 新規作成

### Internal
- `ast.rs`: `ApiAnnotation` struct + `FnDef.api_annotation: Option<ApiAnnotation>`
- `parser.rs`: `parse_api_annotation()`
- `driver.rs`: API 生成・ルートテーブル・HTTP サーバー実装
- Cargo.toml version: `18.8.0`

---

## [v18.7.0] — 2026-06-16 — Const Generics

### Added
- `fn f<const N: Int where { N > 0 }>(x: Int) -> Int` 構文
- E0335 — const constraint 違反エラー
- `site/content/docs/language/const-generics.mdx` 新規作成

### Internal
- `ast.rs`: `GenericParam` に `is_const / const_ty / const_constraint` 追加
- `parser.rs`: `parse_one_type_param()`
- `checker.rs`: `const_generics_registry` + E0335 チェック
- Cargo.toml version: `18.7.0`

---

## [v18.6.0] — 2026-06-16 — 共変・反変アノテーション

### Added
- `interface Source<+T> { ... }` / `interface Sink<-T> { ... }` 構文
- E0334 — 分散違反エラー
- `site/content/docs/language/variance.mdx` 新規作成

### Internal
- `ast.rs`: `GenericParam.variance` フィールド追加
- `checker.rs`: `check_interface_variance()`
- Cargo.toml version: `18.6.0`

---

## [v18.5.0] — 2026-06-16 — 線形型

### Added
- `fn(T) -o U` — 線形関数型（linear arrow）
- E0332 / E0333 — 線形型の二重使用・未使用エラー
- `site/content/docs/language/linear-types.mdx` 新規作成

### Internal
- `ast.rs`: `TypeExpr::LinearArrow` / `Type::LinearFn`
- `checker.rs`: `LinearState` / `linear_env` / 線形型追跡
- Cargo.toml version: `18.5.0`

---

## [v18.4.0] — 2026-06-16 — スキーマ型

### Added
- `type User = schema "file:./schema/user.json"` 構文
- `fav check --refresh-schemas` フラグ、E0338 エラー
- `site/content/docs/language/schema-types.mdx` 新規作成

### Internal
- `ast.rs`: `TypeExpr::Schema(uri, span)`
- `driver.rs`: `schema_loader` モジュール
- Cargo.toml version: `18.4.0`

---

## [v18.3.0] — 2026-06-16 — Refinement Types

### Added
- `fn divide(a: Int, b: Int where { b != 0 }) -> Int` 構文
- E0331 — Refinement 制約違反エラー（コンパイル時）
- `site/content/docs/language/refinement-types.mdx` 新規作成

### Internal
- `ast.rs`: `Param.constraint: Option<Box<Expr>>`
- `checker.rs`: `check_refinement_call_site()`
- Cargo.toml version: `18.3.0`

---

## [v18.2.0] — 2026-06-16 — 行多相（Row Polymorphism）

### Added
- `fn f<R with { id: Int }>(row: R) -> { ...R, ts: String }` 構文
- E0329 / E0330 — レコード制約・spread エラー
- `site/content/docs/language/row-polymorphism.mdx` 新規作成

### Internal
- `ast.rs`: `TypeBound::HasFields` / `TypeExpr::RecordSpread`
- `checker.rs`: `check_row_constraint()`
- Cargo.toml version: `18.2.0`

---

## [v18.1.0] — 2026-06-16 — エフェクト推論（Effect Inference）

### Added
- エフェクト宣言（`!Db`, `!IO` 等）を省略可能に（推移的推論・fixpoint 最大 10 ラウンド）
- `fav check --show-effects` フラグ
- `site/content/docs/language/effect-inference.mdx` 新規作成

### Internal
- `checker.rs`: `EffectSet` / `infer_effects_fn()` / `fn_effects_registry`
- Cargo.toml version: `18.1.0`

---

## [v18.0.0] — 2026-06-16 — Language Power マイルストーン宣言

### Added
- v17.x シリーズ集大成：境界付きジェネリクス / パターンマッチ拡張 / 内包表記 / REPL 品質向上 / `fav bench` / `forall` プロパティテスト / パッケージシステムが揃い Language Power を宣言
- `CHANGELOG.md` / `README.md` 全面更新（v17.1.0〜v18.0.0）
- `site/content/docs/language/patterns.mdx` / `comprehensions.mdx` / `bind.mdx` 新規作成
- `site/content/docs/packages/publishing.mdx` 新規作成

### Internal
- Cargo.toml version: `18.0.0`
- `v180000_tests`: 5 件追加

---

## [v17.8.0] — 2026-06-16 — パッケージシステム成熟（rune registry v2）

### Added
- `fav add <name[@version]>` / `fav update [name]` / `fav remove <name>` / `fav login` CLI 追加
- `fav.toml` に `[dev-dependencies]` / `[registry]` セクション追加
- `fav.lock` に `checksum` / `source` フィールド追加
- `registry/resolver.rs`: `SemVer` / `VersionReq` / `parse_version_req` / `resolve_best` — `^` / `~` / `=` / `*` semver 解決
- `registry/client.rs`: `RegistryClient` / `PackageInfo` / HTTP `fetch_package` / `publish`（`REGISTRY_MOCK=1` テストスタブ）
- `fav_toml_add_dep` ヘルパー（`fav.toml` への dep 追記）
- `cmd_add_impl` テスト用ヘルパー
- `site/content/docs/packages/getting-started.mdx` 新規作成

### Internal
- Cargo.toml version: `17.8.0`
- `v178000_tests`: 5 件追加

---

## [v17.7.0] — 2026-06-15 — `forall` プロパティベーステスト

### Added
- `forall x: Type [where { guard }] { body }` 構文追加
- `TokenKind::Forall` / `Stmt::Forall` / `ForallStmt` / `ForallVar` AST 追加
- `parse_forall_stmt` — `where { guard }` オプション対応
- `check_stmt`: E0327（非対応型）型チェック
- VM primitives: `__forall_gen_int` / `__forall_gen_str` / `__forall_gen_bool` / `__forall_gen_float`（xorshift64 固定シード 12345）
- compiler desugar: ガードなし → ForIn ループ、ガードあり → ListComp + `List.take` + ForIn
- `fav test --cases N` CLI オプション（`FORALL_CASES` 環境変数）
- exhaustive match 更新: fmt / emit_python / lineage(4) / lint(7) / checker(2) / compiler(2)
- `site/content/docs/tools/property-testing.mdx` 新規作成

### Internal
- Cargo.toml version: `17.7.0`
- `v177000_tests`: 5 件追加（version_is test は v17.8.0 で除去）

---

## [v17.6.0] — 2026-06-15 — `fav bench` 統計強化

### Added
- `bench "name" { ... }` 構文追加（AST `Item::BenchDef`）
- `BenchStats`（avg / p50 / p95 / min / max）統計計算
- `cmd_bench(opts: &BenchOpts)` 実装
- `--runs N` / `--warmup N` / `--json` CLI オプション
- `site/content/docs/tools/bench.mdx` 新規作成

### Internal
- Cargo.toml version: `17.6.0`
- `v176000_tests`: 5 件追加

---

## [v17.5.0] — 2026-06-15 — REPL 品質向上

### Added
- `:doc <fn>` / `:load <file>` / `:save <file>` / `:history` / `:paste` REPL コマンド追加
- `:paste` ... `:end` 複数行入力モード
- タブ補完（モジュール名・関数名・`:` コマンド）
- `FavCompleter` タブ補完実装

### Internal
- Cargo.toml version: `17.5.0`
- `v175000_tests`: 5 件追加

---

## [v17.4.0] — 2026-06-15 — `let` バインディング除去（誤実装の修正）

### Removed
- `TokenKind::Let` / `Stmt::Let` / `parse_let_stmt` / E0326 を除去
- `let x = expr` は Favnir に存在しない。`bind x <- expr` に統一

### Changed
- `bind x <- expr` が非 Result 値でも使えることを明確化（既存動作の確認）

### Internal
- Cargo.toml version: `17.4.0`
- `v174000_tests`: 5 件追加

---

## [v17.3.0] — 2026-06-15 — コレクション内包表記

### Added
- `[x * 2 | x <- nums]` list-comp — `List.map` 相当にデシュガー
- `[x | x <- nums, x > 0]` filter-comp — `List.filter` 相当にデシュガー
- `[Pair(a,b) | a <- as, b <- bs]` multi-source — `List.flat_map` 相当にデシュガー
- `[? f(x) | x <- xs]` result-comp — `List.collect_result` 相当にデシュガー
- `CompClause::For` / `CompClause::Guard` AST 追加
- `Expr::ListComp` / `Expr::ResultComp` AST 追加
- `List.collect_result` builtin primitive 追加
- exhaustive match 更新: lineage(4) / lint(6) / fmt / emit_python / driver(2)

### Internal
- Cargo.toml version: `17.3.0`
- `v173000_tests`: 5 件追加

---

## [v17.2.0] — 2026-06-15 — パターンマッチ拡張

### Added
- or-pattern: `"a" | "b" => ...`（`Pattern::Or`）
- list-pattern: `[] / [x] / [head, ..tail]`（`Pattern::List`）
- guard 条件: `if expr` in match arm（`MatchArm.guard`）
- `DotDot` トークン（`..`）追加（`DotDotDot` との区別）
- `IRPattern::Or` / `IRPattern::List` IR 追加
- `ListLen` (0x60) / `ListGet` (0x61) / `ListDrop` (0x62) VM opcodes 追加
- `emit_pattern_test` で Or / List パターンを処理
- exhaustive match 更新: checker / compiler / fmt / ast_lower_checker / emit_python / driver

### Internal
- Cargo.toml version: `17.2.0`
- `v172000_tests`: 5 件追加

---

## [v17.1.0] — 2026-06-15 — 境界付きジェネリクス（Bounded Generics）

### Added
- `fn f<T with Ord>(a: T, b: T) -> T` 構文追加
- `GenericParam { name: String, bounds: Vec<String> }` AST 追加（7 struct 変更）
- `parse_type_bounds` — `TokenKind::With` 対応
- `fn_bounds_registry: HashMap<String, Vec<GenericParam>>` in Checker
- `type_implements_bound` — 組み込み bound 自動実装テーブル
- 組み込み bounds: `Ord` / `Eq` / `Serialize` / `Display` / `Hash` / `Clone`
- call-site E0325: bound を満たさない型を渡すとエラー
- `site/content/docs/language/generics.mdx` 新規作成

### Internal
- Cargo.toml version: `17.1.0`
- `v171000_tests`: 6 件追加

---

## [v17.0.0] — 2026-06-14 — Language Ergonomics マイルストーン宣言

### Added
- v16.x シリーズ集大成：f-string / record spread / stdlib 拡充 / 型エイリアス / namespace alias / fav test 成熟 / tap 演算子が揃い Language Ergonomics を宣言
- `site/content/docs/stdlib/list.mdx` / `string.mdx` / `datetime.mdx` / `math.mdx` v16.4.0 内容反映
- `README.md` / `CHANGELOG.md` 全面更新（v16.1.0〜v17.0.0）

### Internal
- Cargo.toml version: `17.0.0`
- `v170000_tests`: 5 件追加

---

## [v16.8.0] — 2026-06-14 — tap / inspect パイプライン演算子

### Added
- `FlwStep::Tap(Box<Expr>)` / `FlwStep::Inspect` を AST に追加（ソフトキーワード）
- `|> tap(observer_fn)` — 値を変換せず副作用（ログ等）だけ実行してそのまま通す
- `|> inspect` — `[inspect] <value>` 形式で標準出力に出力する組み込み tap
- `inspect_debug` VM プリミティブ
- `CompileCtx.no_tap` フィールド + `set_no_tap_mode()` スレッドローカル
- `fav run --no-tap` — tap/inspect を identity にコンパイルしてゼロオーバーヘッド化
- `IRExpr::Block` + `IRStmt::Bind` + `IRStmt::Expr` で実装（新 VM opcode 不要）
- exhaustive match 更新: `checker.rs` / `ast_lower_checker.rs` / `emit_python.rs`
- `site/content/docs/language/pipeline.mdx` に tap/inspect セクション追加

### Internal
- Cargo.toml version: `16.8.0`
- `v168000_tests`: 6 件追加

---

## [v16.7.0] — 2026-06-14 — fav test 成熟（assert_eq / test_group / スナップショット）

### Added
- `test_group "name" { test ... }` — 関連テストのグループ化構文
- `assert_eq(actual, expected)` — `vmvalue_repr` で文字列化して比較、不一致で詳細エラー
- `assert_approx_eq(actual, expected, epsilon)` — Float 近似比較
- `assert_contains(list, elem)` — リスト内要素存在確認
- `assert_length(list, n)` — リスト長確認
- `assert_str_contains(s, substring)` — 文字列部分一致確認
- `assert_str_starts_with(s, prefix)` — 文字列プレフィックス確認
- `assert_err_eq(result, expected_msg)` — エラー内容の文字列一致確認
- `assert_snapshot(value, name)` — `.snap/{name}.snap` の作成・比較
- `fav test --update-snapshots` — 全スナップショットを上書き更新
- `collect_test_cases` を 4-tuple `(path, display_name, fn_name, prog)` に変更
- `site/content/docs/language/testing.mdx` 全面更新（全アサート・snapshot ワークフロー）

### Internal
- Cargo.toml version: `16.7.0`
- `v167000_tests`: 5 件追加（`set_var` は Rust 2024 edition で unsafe）

---

## [v16.6.0] — 2026-06-14 — Namespace Alias（use String as S）

### Added
- `use String as S` / `use List as L` 構文（ソフトキーワード `as`）
- `TokenKind::As`、`Item::UseAlias { alias, namespace, span }`
- `namespace_aliases: HashMap<String, String>` in `CompileCtx` + `Checker`
- `check_builtin_apply` と `compile_expr FieldAccess` でエイリアス解決
- `parse_import_decl` の `import "path" as alias` も `TokenKind::As` 対応
- `site/content/docs/language/modules.mdx` 新規作成

### Internal
- Cargo.toml version: `16.6.0`
- `v166000_tests`: 5 件追加

---

## [v16.5.0] — 2026-06-14 — 型エイリアス（alias キーワード）

### Added
- `alias Email = String` — 型エイリアス宣言（`alias` キーワード）
- `alias Result2<T> = Result<T, String>` — ジェネリクスエイリアス
- `Alias` トークン、`Item::AliasDecl { name, params, ty, span }`
- `alias_env: HashMap<String, (Vec<String>, TypeExpr)>` in `CompileCtx` / `Checker`
- `resolve_type_expr_with_self` / `resolve_type_expr_with_subst` 双方に alias 解決追加
- compiler.rs は catch-all で自動スキップ
- `site/content/docs/language/type-alias.mdx` 新規作成

### Internal
- Cargo.toml version: `16.5.0`
- `v165000_tests`: 5 件追加

---

## [v16.4.0] — 2026-06-14 — 標準ライブラリ拡充（List / String / DateTime / Math）

### Added
- **List**: `sort_by` / `sort_by_desc` / `distinct` / `distinct_by` / `count_where` / `sum_by` / `max_by` / `min_by` / `unzip`（高階関数）
- **String**: `split_once` / `replace_first` / `format_int(n, width, pad)` / `format_float(f, decimals)`
- **DateTime**: 新モジュール全 12 関数（`now` / `parse` / `format` / `add_days` / `add_hours` / `diff_days` / `year` / `month` / `day` / `weekday` / `timestamp` / `from_timestamp`）。内部表現は Unix timestamp（Int）。`chrono` クレートを使用。
- **Math**: `round_to(f, n)` / `log(f)` / `log2(f)` / `log10(f)`
- `compiler.rs` / `checker.rs` に `DateTime` 名前空間登録

### Internal
- Cargo.toml version: `16.4.0`
- `v164000_tests`: 6 件追加

---

## [v16.3.0] — 2026-06-14 — レコード更新構文（{ ...base, field: val }）

### Added
- `{ ...base, field: val }` — レコードスプレッド / 更新構文
- `DotDotDot` トークン、`Expr::RecordSpread { base, overrides }`
- `IRExpr::RecordSpread`、`MergeRecord = 0x5C` VM opcode
- `remap_string_operands` に `MergeRecord` 追加（未追加だと後続 GetField が壊れる問題を修正）

### Internal
- Cargo.toml version: `16.3.0`
- `v163000_tests`: 6 件追加

---

## [v16.2.0] — 2026-06-14 — f-string 文字列補間

### Added
- `f"Hello, {name}!"` — f-string プレフィックス付き文字列補間
- `f"""..."""` — 三重クォート f-string
- `FStringRaw` トークン、`lex_fstring_triple`、`lower_fstring`（コンパイル時に `String.concat` 連鎖へ展開、VM 変更なし）

### Internal
- Cargo.toml version: `16.2.0`
- `v162000_tests`: 5 件追加

---

## [v16.1.0] — 2026-06-14 — エラーメッセージ品質向上

### Added
- rustc スタイルのエラー表示（`-->` ファイル・行・列、`^` アンダーライン）
- `Span { line, col, len }` を AST 全ノードに追加
- typo ヒント（Levenshtein 距離 ≤ 2 の候補を最大 3 件表示）
- `= hint:` / `= help:` メッセージ付与
- エラーコード URL（`https://favnir.dev/errors/E0xxx`）

### Internal
- Cargo.toml version: `16.1.0`
- `v161000_tests`: 5 件追加

---

## [v16.0.0] — 2026-06-14 — Production Multi-Cloud マイルストーン宣言

### Added
- v15.x シリーズ集大成：CrossCloud 認証・GCP BigQuery・Kafka/MSK・`fav test`・`fav deploy` が揃い Production Multi-Cloud を宣言
- `site/content/docs/runes/bigquery.mdx` / `kafka.mdx` ドキュメント追加
- 対応クラウド: AWS / Azure / GCP / Snowflake + Kafka/MSK（4 クラウド + ストリーミング）

### Internal
- Cargo.toml version: `16.0.0`
- `v160000_tests`: 5 件追加

---

## [v15.5.0] — 2026-06-14 — `fav deploy`（AWS Lambda デプロイ CLI）

### Added
- `DeployConfig` に `target` / `function_name` フィールド追加（ロードマップ仕様準拠）
- `memory_mb` / `timeout_sec` を `memory` / `timeout` のエイリアスとして追加
- `runtime` デフォルトを `provided.al2023` に更新
- `scripts/build-lambda-layer.sh`：`cross` で `x86_64-unknown-linux-musl` クロスコンパイル → `bootstrap` + zip パッケージング
- `site/content/docs/deploy.mdx`：`fav deploy` ユーザーガイド新規作成

### Internal
- Cargo.toml version: `15.5.0`
- `v155000_tests`: 3 件追加（version / deploy_toml_schema_parses / deploy_cmd_exists）

---

## [v15.4.0] — 2026-06-14 — Kafka / MSK Rune（`!Stream` エフェクト）

### Added
- `Effect::Stream` 追加（ast.rs + 全 exhaustive match 対応）
- `Kafka.produce_raw(brokers, topic, key, value)` / `Kafka.consume_one_raw(brokers, topic, group_id)` VM プリミティブ（rskafka 0.6 pure-Rust、SCRAM-SHA-512 認証）
- E0319：`!Stream` エフェクト欠如エラー
- `fav.toml [kafka]` セクション（`bootstrap_brokers` / `sasl_mechanism` / `sasl_username` / `sasl_password`）
- `runes/kafka/kafka.fav`：`produce` / `consume_one` ラッパー
- `infra/e2e-demo/kafka/`：4-stage pipeline + Terraform AWS MSK
- `self/checker.fav`：`kafka_fn` / `ns_to_effect "Kafka"→"Stream"` 追加

### Internal
- Cargo.toml version: `15.4.0`
- 依存追加：`rskafka 0.6`（`transport-tls` feature）
- `v154000_tests`: 5 件追加

---

## [v15.3.0] — 2026-06-14 — `fav test` DSL（ネイティブテストフレームワーク）

### Added
- `test "description" { ... }` 構文（`TopLevel::TestDef`）
- `assert_ok` / `assert_err` / `assert_true` VM プリミティブ
- `cmd_test`（Bool(false) → FAIL 判定修正含む）
- `site/content/docs/language/testing.mdx` 新規作成

### Internal
- Cargo.toml version: `15.3.0`
- `v153000_tests`: 5 件追加

---

## [v15.2.0] — 2026-06-14 — GCP BigQuery Rune（`!Gcp` エフェクト）

### Added
- `Effect::Gcp` 追加
- `BigQuery.query_raw` / `BigQuery.execute_raw` / `BigQuery.infer_table_raw` VM プリミティブ（RS256 JWT + Google OAuth2）
- E0318：`!Gcp` エフェクト欠如エラー
- `fav.toml [gcp]` セクション（`project_id` / `credentials_file` / `dataset` / `location`）
- `runes/bigquery/bigquery.fav`：`query` / `execute` ラッパー
- `infra/e2e-demo/bigquery/`：4-stage pipeline + Terraform GCP BigQuery
- `self/checker.fav`：`bigquery_fn` / `ns_to_effect "BigQuery"→"Gcp"` 追加

### Internal
- Cargo.toml version: `15.2.0`
- `v152000_tests`: 5 件追加

---

## [v15.1.5] — 2026-06-14 — CrossCloud 認証層セキュア版（KMS ECDSA P-256）

### Added
- Lambda verifier_v2（KMS `GetPublicKey` + Python `cryptography` ECDSA P-256 検証）
- `infra/e2e-demo/crosscloud/lambda/verifier_v2/`
- `infra/e2e-demo/crosscloud/scripts/run_with_kms.sh`
- Terraform：`aws_kms_key`（ECC_NIST_P256 / SIGN_VERIFY）+ `aws_kms_alias`
- E2E：改ざんボディ / ランダム署名 → PASS=2 FAIL=0

### Internal
- Cargo.toml version: `15.1.5`
- `v15150_tests`: 6 件追加

---

## [v15.1.0] — 2026-06-14 — CrossCloud 認証層（HMAC + Cognito + Lambda verifier）

### Added
- `AWS.dynamo_put_item_cond_raw` VM プリミティブ（DynamoDB ConditionalPut、TTL + nonce リプレイ防止）
- `AWS.ecs_run_task_raw` VM プリミティブ（ECS Fargate RunTask、SigV4）
- Lambda verifier（Favnir コンテナ、`public.ecr.aws/lambda/provided:al2023` ベース）
- Cognito JWT Authorizer + API Gateway
- HMAC-SHA256 署名方式（StringToSign = Method\nPath\nTimestamp\nNonce\nSHA256(Body)）
- E2E：`reject_cases.sh` PASS=5 FAIL=0、S3 証跡保存

### Fixed
- `fav run --legacy` が `Result.err` を返しても exit 0 だった問題を修正（`process::exit(1)` 追加）
- `AWS_CONFIG` thread-local が `default()` でハードコード値を返していた問題を `from_env()` に修正

### Internal
- Cargo.toml version: `15.1.0`
- `v151000_tests`: 6 件追加

---

## [v14.8.0] — 2026-06-12 — Rune ファイル整備（--legacy 明示化 + fs.fav バグ修正）

### Fixed
- `runes/fs/fs.fav`: `glob` 関数内の非 Result `bind`（`bind sep <- "/"` 等）をインライン化で修正
- `runes/fs/fs.fav`: `walk_entry` 関数内の非 Result `bind full_path` もインライン化で修正

### Changed
- rune ファイル 12 件に `--legacy compatible` コメントを追加（意図を明示）:
  `cache/cache.fav`, `fs/fs.fav`, `log/emitter.fav`, `log/metric.fav`,
  `queue/queue.fav`, `gen/output.fav`, `http/request.fav`, `graphql/client.fav`,
  `grpc/server.fav`, `duckdb/query.fav`, `duckdb/io.fav`, `db/connection.fav`

### Internal
- Cargo.toml version: `14.8.0`
- `v148000_tests`: 3 件追加

---

## [v14.7.0] — 2026-06-12 — site/ ドキュメント更新 + rune ファイル精査

### Changed
- `site/content/docs/introduction.mdx`: 旧エフェクト表・存在しない機能（fav deploy / MCP / Notebook）を削除。Capability Context 体系で書き直し
- `site/content/docs/language/effects.mdx`: v14.0.0 Capability Context を主体に全面書き直し。E0370 削除、E0023/E0025/E0021 追加
- `site/content/docs/quickstart.mdx`: `ctx: AppCtx` スタイルのサンプルに更新。`bind user <- User{...}` 誤用を `let` に修正
- `site/content/docs/installation.mdx`: バージョン表示 `v5.0.0` → `v14.7.0`。`fav deploy` / `fav mcp` / `fav explain-error`（非実装コマンド）を削除
- `runes/aws/dynamodb.fav`, `runes/aws/sqs.fav`: `--legacy` 専用 API コメントを追加

### Internal
- Cargo.toml version: `14.7.0`
- `v148000_tests`: 3 件追加（v147000_tests の誤記 — 本体は v147000_tests）

---

## [v14.6.0] — 2026-06-12 — ドキュメント整備（README + CHANGELOG）

### Changed
- `README.md`: 「現在の状態」見出しを v14.6.0 に更新、ロードマップ表に v14.1.0〜v14.6.0 を追記
- `README.md`: 機能一覧表に Azure Blob Storage / Azure PostgreSQL 行を追加
- `README.md`: 旧 `!Effect` スタイルコード例に `--legacy` モード注記を追加
- `CHANGELOG.md`: v14.1.0〜v14.5.0 エントリを追加

### Notes
- コードの変更なし。純粋なドキュメント更新バージョン
- テスト: v146000_tests 3 件（version_is_14_6_0 / changelog_has_v14_5_0_entry / readme_mentions_azure_blob）

---

## [v14.5.0] — 2026-06-12 — Azure Blob Storage Rune

### New Features
- `azure_blob_sign` ヘルパー関数（`vm.rs`）: HMAC-SHA256 + base64 による Azure Shared Key 署名
  - 既存の `hmac 0.12` + `sha2 0.10` + `base64 0.22` + `chrono` を使用（新規 crate なし）
  - RFC 1123 日付フォーマット、x-ms-* ヘッダーのアルファベット順ソート
- `AzureBlob.put_raw(account, key, container, blob_name, body)` VM primitive（BlockBlob PUT）
- `AzureBlob.get_raw(account, key, container, blob_name)` VM primitive（GET → String）
- `AzureBlob.list_raw(account, key, container, prefix)` VM primitive（GET → JSON 配列文字列）
- `AzureBlob.delete_raw(account, key, container, blob_name)` VM primitive（DELETE）
- `checker.rs`: `require_azure_storage_effect` — `!AzureStorage` 未宣言時に E0317 を発生
- `checker.rs`: `("AzureBlob", "put_raw/get_raw/list_raw/delete_raw")` を `builtin_ret_ty` に追加
- `checker.rs`: `"AzureBlob"` を `BUILTIN_EFFECTS` に追加
- `runes/azure-blob/azure_blob.fav`: `put/get/list/delete` ctx-aware ラッパー（`ctx: String`）
- `runes/azure-blob/rune.toml`: rune メタデータ（version 14.5.0、effects !AzureStorage）

### Notes
- テスト: v145000_tests 4 件（version_is_14_5_0 / azure_blob_put_raw_registered / azure_storage_effect_required / azure_blob_rune_file_present）
- `let` 構文は rune ファイル内でパースエラーになるため引数はインライン化
- `import rune "ctx"` は使用不可（runes/ctx/ctx.fav 未存在）→ `ctx: String` で代替
- LIST の canonical_resource は query params をアルファベット順にソート: `comp:list\nprefix:...\nrestype:container`

---

## [v14.4.0] — 2026-06-12 — AWS Rune 正式パッケージング

### New Features
- `AWS.secrets_get_raw(region, secret_name)` VM primitive（SigV4 + ureq で Secrets Manager `GetSecretValue` API）
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx JSON 文字列からフィールドを取得
- `checker.rs`: `("AWS", "secrets_get_raw")` を `builtin_ret_ty` に追加（`require_aws_effect` 呼び出し）
- `checker.rs`: `("Ctx", "aws_get_field_raw")` → `Some(Type::String)` を `builtin_ret_ty` に追加
- `runes/aws/secrets.fav`: `secrets_get(ctx: String, secret_name: String)` ラッパー
- `runes/aws/s3.fav`: `s3_put/s3_get/s3_delete/s3_list` ctx-aware ラッパーを追加
- `runes/aws/rune.toml`: version `14.4.0`、description に Secrets Manager を追記

### Notes
- テスト: v144000_tests 4 件（version_is_14_4_0 / secrets_get_raw_registered / aws_ctx_field_raw_registered / aws_rune_s3_ctx_functions_present）
- LocalStack エンドポイント対応（`config.endpoint_url` がある場合は `/` に置換）
- `let` 構文パースエラーのため rune ファイルは全引数インライン化

---

## [v14.3.0] — 2026-06-12 — Azure lineage + !AzureStorage エフェクト

### New Features
- `ast::Effect::AzureStorage` 追加（parser / lineage / checker で認識）
- `lineage.rs`: `EffectKind::AzureDbRead` / `AzureDbWrite` / `AzureBlobRead` / `AzureBlobWrite` 追加
- `lineage.rs`: `collect_azure_blob_call_kinds` / `collect_azure_db_call_kinds` 追加
- `checker.rs`: `BUILTIN_EFFECTS` に `"AzureStorage"` を追加
- `fav explain --lineage` 出力に Azure エフェクトが表示されるよう更新

### Notes
- テスト: v143000_tests

---

## [v14.2.0] — 2026-06-12 — AzureCtx / AwsCtx + fav.toml [azure]

### New Features
- `Ctx.build_aws_raw(region, s3_bucket, db_url)` VM primitive — AwsCtx JSON を生成
- `Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)` VM primitive — AzureCtx JSON を生成
- `Ctx.aws_get_field_raw(ctx, field)` VM primitive — AwsCtx からフィールドを取得（v14.4.0 で checker に追加）
- `Ctx.azure_get_field_raw(ctx, field)` VM primitive — AzureCtx からフィールドを取得
- `fav.toml` に `[azure]` セクション追加（`postgres_url` / `storage_account` / `storage_key` / `container`）
- `inject_azure_config` — fav.toml の [azure] セクションを env var 展開して実行時 ctx に注入

### Notes
- テスト: v142000_tests

---

## [v14.1.0] — 2026-06-12 — Azure PostgreSQL Rune

### New Features
- `AzurePostgres.execute_raw(conn_str, sql, params)` VM primitive（tokio-postgres + tokio ランタイム）
- `AzurePostgres.query_raw(conn_str, sql, params)` VM primitive（JSON 配列文字列として返す）
- `checker.rs`: `AzurePostgres` namespace を `builtin_ret_ty` / `BUILTIN_EFFECTS` に追加
- `checker.rs`: `require_azure_db_effect` — `!AzureDb` 未宣言時に E0316 を発生
- `ast::Effect::AzureDb` 追加
- `lineage.rs`: `!AzureDb(read/write)` 区別追加
- `runes/azure-postgres/azure_postgres.fav`: `execute/query_rows` ctx-aware ラッパー
- `runes/azure-postgres/rune.toml`: rune メタデータ

### Notes
- テスト: v141000_tests
- SSL: `sslmode=require` を接続文字列に付加して Azure DB for PostgreSQL の SSL 必須要件に対応

---

## [v14.0.0] — 2026-06-11 — 能力型完成宣言

### Breaking Changes
- `!Effect` 記法は非 legacy モードで E0025 エラーになる（v13.10.0 から段階的導入、v14.0.0 で CI 確認完了）
- ambient effect 呼び出し（ctx なしの `IO.println` 等）は E0023 エラーになる（v13.8.0 から）

### New Features (v13.1.0〜v13.10.0 集約)
- `interface` 継承構文（`LoadCtx: CommonCtx`）のコンパイル時チェック
- `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` / `HttpClient` / `Io` / `Env` capability interface
- `LoadCtx` / `WriteCtx` / `MigrateCtx` コンテキスト interface（capability 充足チェック付き）
- `AppCtx` 具象型 + `Ctx.build` / `Ctx.mock` Rune
- `ctx.field.method()` フィールドアクセス構文
- `seq Pipeline(ctx)` — ctx 型推論
- E0024 型状態パターンチェック
- `Ctx { db: DbRead }` 糖衣構文（v13.10.0）
- `fav migrate --from-effects` 移行ツール（v13.10.0）

### Error Codes Added
- W008: ambient effect call（警告）
- E0020: capability interface has no such method
- E0021: capability not in context
- E0022: ctx-aware pipeline called with wrong number of arguments
- E0023: ambient effect call is not allowed（エラー）
- E0024: type state mismatch
- E0025: bang notation removed
- W009: direct Rune call deprecated
- W010: effect migration requires manual review

### Migration
`fav migrate --from-effects <file>` で旧 `!Effect` 記法を自動変換。
`--legacy` フラグで移行期間中も旧記法を許容（今後廃止予定）。

### Notes
- `self/compiler.fav` / `self/checker.fav` の E0025 件数がゼロであることを CI テストで保証
- テスト: 2207 件（v13.10.0 時点）

---

## [v13.0.0] — 2026-06-09

### Added
- 言語信頼性宣言: 型安全・エラー伝播・デバッグ可視性の三点における保証
- README.md に v13.0.0 宣言文を追記
- `versions/v13.0.0/` — spec / plan / tasks

### Notes
- テスト: 1415 件
- v12.1.0〜v12.10.0 で発覚した全問題（C-1〜C-4 / H-1〜H-2 / M-1 / A-1〜A-6）を解消

---

## [v12.10.0] — 2026-06-09

### Added
- `driver.rs` `get_help_text(code: &str) -> &'static [&'static str]` — 12 コード（E0001/E0007/E0008/E0009/E0013/E0014/E0015/E0018/W001/W004/W006/W007）に `help:` テキストを追加
- `fav check --strict` — W006 警告をエラーとして扱い exit 1（`-D warnings` 相当）
- `fav lint --deny-warnings` — 警告を exit 1 に昇格させる CI 用フラグ
- `fav.toml [lint]` セクション — `warn_as_error` / `allow` リストによる細粒度制御
- `toml.rs` `LintTomlConfig { warn_as_error: Option<Vec<String>>, allow: Option<Vec<String>> }`
- `driver.rs` `v121000_tests` — `help_text_e0001_present` / `help_text_w006_present` / `help_text_unknown_is_empty` / `version_is_12_10_0`
- `tests/integration.rs` — `check_strict_w006_exits_1` / `check_strict_no_warning_exits_0` / `lint_deny_warnings_exits_1`

### Changed
- `format_diagnostic` / `format_warning` — エラー・警告出力末尾に `= help:` 行を自動付与
- `cmd_lint` — `warn_only` に加え `deny_warnings` パラメータを追加; `[lint]` allow フィルタ・warn_as_error 昇格を適用
- `.github/workflows/ci.yml` Self-lint ステップに `--deny-warnings` を追加

### Notes
- テスト: 1415 unit + 8 integration

---

## [v12.9.0] — 2026-06-09

### Added
- `.github/workflows/ci.yml` `Self-test (fav test)` ステップ — `self/checker.fav` / `self/compiler.fav` / `self/codegen.fav` / `self/lexer.fav` / `self/parser.fav`
- `.github/workflows/ci.yml` `integration` ジョブ — `services: postgres:16` (POSTGRES_PASSWORD=test) + health check
- `fav/tests/integration.rs` — `fav_test_self_checker_runs` / `fav_test_self_lexer_runs` / `postgres_create_insert_select` / `postgres_error_table_not_found` / `postgres_ssl_disable_connects`
- `driver.rs` `pg_exec_for_test` / `pg_query_for_test` — 統合テスト用 pub ヘルパー
- `driver.rs` `v12900_tests` — `version_is_12_9_0`

### Notes
- テスト: 1415 件（統合テスト 8 件含む）

---

## [v12.8.0] — 2026-06-09

### Added
- `fav scaffold <template>` コマンド — stage / seq / postgres-etl / rune テンプレートを標準出力に生成
- `driver.rs` `cmd_scaffold(template: &str, name: Option<&str>)` 実装
- `main.rs` `Some("scaffold")` 分岐を追加
- `driver.rs` `v12800_tests` — `scaffold_stage_output_contains_stage` / `scaffold_seq_output_contains_seq` / `scaffold_postgres_etl_output_contains_stages` / `scaffold_rune_output_contains_rune` / `scaffold_stage_named_output_contains_name` / `version_is_12_8_0`（← comment out 済み）

### Notes
- テスト: 1411 件

---

## [v12.7.0] — 2026-06-08

### Added
- `fav doc --builtins [--format json|markdown] [--out <file>]` — 組み込み Primitive の型シグネチャ一覧（IO/Csv/Schema/Json/Gen/AWS/Postgres/Snowflake/Http/Llm）
- `fav explain <code>` — エラーコードの詳細説明（E0001〜E0018 / W001〜W007）
- `driver.rs` `builtin_primitives()` — 組み込み関数メタデータのリスト
- `driver.rs` `cmd_doc_builtins(format, out)` / `cmd_explain_code(code)`
- `driver.rs` `v12700_tests` — `doc_builtins_json_has_csv_parse_raw` / `doc_builtins_markdown_has_postgres` / `explain_e0001_output` / `explain_w006_output` / `doc_builtins_returns_result_field`

### Notes
- テスト: 1408 件

---

## [v12.6.0] — 2026-06-08

### Added
- `tokio-postgres-native-tls` / `native-tls` — Postgres TLS 対応
- `fav.toml [postgres]` `sslmode` キー（`disable` / `prefer` / `require`）
- `DATABASE_URL` の `sslmode` クエリパラメータ解析
- Postgres エラー詳細化 — `DbError.message()` / `code()` / `detail()` を連結（"db error" → "db error: SSL connection is required (SQLSTATE 08P01)"）
- `driver.rs` `v12600_tests` — `postgres_sslmode_disable` / `postgres_sslmode_parse` / `postgres_error_detail`

### Changed
- `pg_connect` — `sslmode` に応じて `NoTls` / `TlsConnector` を切り替え

### Notes
- テスト: 1402 件

---

## [v12.5.0] — 2026-06-08

### Added
- `fav run --verbose` — stage 入出力を stderr に出力（最大 200 文字トランケート）
- `fav run --trace` — stage 入出力をフル出力（トランケートなし）
- `fav.toml [run]` `verbose` / `trace` キー
- `fav check --json` — エラー・警告を JSON 形式で出力（AI フレンドリー）
- `fav check --show-types` — 各 `bind` / `chain` の型と W006 マーカーを表示
- `driver.rs` `CheckDiagnostic` / `BindingInfo` / `CheckOutput` 構造体（serde::Serialize）
- `driver.rs` `collect_binding_types(file)` — W006 検出（`bind _ <- NS.fn(...)` パターン）
- `driver.rs` `v12500_tests` — `verbose_stage_enter_exit` / `check_json_output_format` / `check_show_types_bind` / `check_show_types_w006_detected`

### Changed
- `VERBOSE_LEVEL` を `thread_local! { Cell<u8> }` に変更（並行テスト対応）

### Notes
- テスト: 1386 件

---

## [v12.4.0] — 2026-06-08

### Added
- `IRStmt::SeqChain` + `Opcode::SeqStageCheck = 0x36` — seq pipeline fail-fast
- `compile_flw_def` 修正: 2+ ステージを `SeqChain` stmts で構築
- `SeqStageCheck` VM ハンドラ: stage 名・番号付きエラーで短絡（`"pipeline stopped at stage N/M 'Name': error"`）
- `driver.rs` `v12400_tests` — `seq_stops_on_stage_err` / `seq_passes_ok_through` / `seq_error_includes_stage_name`

### Notes
- テスト: 1376 件

---

## [v12.3.0] — 2026-06-08

### Added
- `IRStmt::LegacyBind(u16, IRExpr)` + `Opcode::LegacyBindCheck = 0x35`
- `apply_legacy_bind_semantics(ir: IRProgram)` — `--legacy` モードで `Bind` → `LegacyBind` に変換
- `LegacyBindCheck` VM ハンドラ: `ok(v)`→unwrap, `err(e)`→escape, 非 Result→pass-through
- `driver.rs` `v12300_tests` — `legacy_bind_propagates_err` / `legacy_bind_ok_unwraps` / `legacy_bind_non_result_passthrough`

### Changed
- `--legacy` モードの `bind x <- expr` が `expr` の Result を unwrap して短絡するように修正（真の monadic bind）

### Notes
- テスト: 1370 件

---

## [v12.2.0] — 2026-06-07

### Added
- `is_result_returning_call(stmt)` — `bind _ <- NS.fn(...)` で Result を返す NS 呼び出しを AST 解析で検出
- W006 警告（`fav check --show-types`）: bind _ で Result を捨てると警告
- 対象 NS: Postgres / Snowflake / S3 / Sqs / Queue / Cache / Http / Grpc / Llm / IO
- `driver.rs` `v12200_tests` — `w006_detected_for_postgres_bind_underscore` / `w006_not_detected_for_named_bind`

### Notes
- テスト: 1357 件

---

## [v12.1.0] — 2026-06-07

### Added
- E0018 `bind` 再束縛禁止（checker.fav）— 同一スコープで同名変数への二重 `bind` を検出
- `check_rebind_ok(name, env)` ヘルパー — `Option<String>` → `Result<String, String>`
- `driver.rs` `v12100_tests` — `e0018_rebind_detected` / `e0018_underscore_allowed` / `e0018_help_message_shown`

### Changed
- `checker.fav` `infer_stmt` に bind 済みセット管理を追加

### Notes
- テスト: 1353 件

---

## [v12.0.0] — 2026-06-06

### Added
- `site/content/docs/transpile/python.mdx` — Python トランスパイラ公式ドキュメント（使用方法・エフェクト対応表・変換例・E2E デモリンク）
- Python トランスパイラ完成宣言（v11.1.0〜v11.9.0 の全機能が揃った）

### Changed
- README.md に `fav transpile --target python` 機能行を追記
- CHANGELOG に v11.1.0〜v11.9.0 の全履歴を追記

### Notes
- テスト: 707 件（v12000_tests 2 件追加）

---

## [v11.9.0] — 2026-06-06

### Added
- `infra/e2e-demo/fav2py/` — Fav ネイティブ vs Python トランスパイル E2E インフラ
  - `src/pipeline.fav` — LoadAndInsert |> Aggregate |> SaveResult（RDS Postgres）
  - `src/sample.csv` — 103 行サンプルデータ（region × category × amount）
  - `terraform/main.tf` — VPC / RDS PostgreSQL (t3.micro) / ECS Fargate x2 / ECR
  - `terraform/iam.tf` — ECS 実行ロール + タスクロール（S3 書き込み）
  - `terraform/variables.tf` / `terraform/outputs.tf`
  - `scripts/upload.sh` — Docker build + ECR push + S3 source upload
  - `scripts/run.sh` — terraform apply → ECS タスク x2 起動 → verify.sh 呼び出し
  - `scripts/verify.sh` — S3 最新 2 件 JSON 比較（native == python）
  - `Dockerfile` — Ubuntu 22.04 + uv + psycopg2-binary + fav binary
- `driver.rs` `v11900_tests` — `fav2py_e2e_demo_structure` / `fav2py_pipeline_fav_transpiles`

### Notes
- テスト: 705 件

---

## [v11.8.0] — 2026-06-06

### Added
- `fav transpile --no-check` オプション（型チェックスキップ）
- `fav transpile --lineage` オプション（生成 Python コードに lineage コメント付与）
- `emit_python.rs` `emit_python_with_lineage(prog, path, HashMap<String,String>) -> String`
- `emit_python.rs` `Emitter` に `lineage_comments: HashMap<String,String>` フィールド追加
- `driver.rs` `build_lineage_comments(report: &LineageReport) -> HashMap<String,String>`
- `driver.rs` `check_source_str_pub(src: &str) -> Vec<TypeError>`（pub ラッパー）
- `driver.rs` `v11800_tests` — 6 件（checker 統合 / lineage コメント検証）

### Changed
- `fav transpile` 実行前に `checker.fav` で型チェックを走らせる（型エラーで Python 生成をブロック）

### Notes
- テスト: 703 件

---

## [v11.7.0] — 2026-06-06

### Added
- `fav transpile --out-dir <dir>` — `main.py` + `pyproject.toml` + `README.md` を出力ディレクトリに生成
- `fav transpile --check` — `python -m py_compile` による構文検証
- `fav transpile --run` — 生成後に `uv run main.py` まで一括実行
- `driver.rs` `build_pyproject_content(py_src, name) -> String`（boto3 / psycopg2 依存を自動検出）
- `driver.rs` `build_readme_content(input_path, name) -> String`
- `driver.rs` `v11700_tests` — 6 件（pyproject 生成 / README 生成 / uv フラグ検証）

### Notes
- テスト: 697 件

---

## [v11.6.0] — 2026-06-06

### Added
- `emit_python.rs` `!Postgres` → psycopg2 変換
  - `_pg_connect()` — `DATABASE_URL` または `PGHOST`/`PGPORT`/etc. から接続
  - `_pg_execute(sql, params)` — INSERT/UPDATE/DELETE ヘルパー
  - `_pg_query(sql, params)` — SELECT → `RealDictCursor` ヘルパー
  - `Postgres.execute_raw` → `_pg_execute(sql, params)`
  - `Postgres.query_raw` → `_pg_query(sql, params)`
- `emit_python.rs` `needs_psycopg2` / `needs_pg_helpers` フラグ追加（2-pass 検出）
- `pyproject.toml` 生成時に `import psycopg2` 検出 → `psycopg2-binary>=2.9` 依存を自動追加
- `driver.rs` `v11600_tests` — 6 件

### Notes
- テスト: 691 件

---

## [v11.5.0] — 2026-06-06

### Added
- `Effect::Postgres` 追加（ast.rs / parser.rs / fmt.rs / lineage.rs / driver.rs / ast_lower_checker.rs / checker.rs / reachability.rs）
- `vm.rs` `Postgres.execute_raw(sql, params_json) -> Result<Unit, String>`（tokio-postgres ベース）
- `vm.rs` `Postgres.query_raw(sql, params_json) -> Result<String, String>`（JSON 文字列返却）
- `vm.rs` `Postgres.query_typed_raw(sql, params_json) -> Result<String, String>`（型付きクエリ）
- `toml.rs` `PostgresTomlConfig` — `fav.toml` `[postgres]` セクション解析
- `runes/postgres/postgres.fav` — `execute` / `query<T>` Rune 実装（`!Postgres` エフェクト）
- `checker.fav` `postgres_fn` / `builtin_ret_ty` / `ns_to_effect` に Postgres 追加
- `driver.rs` `v11500_tests` — 6 件

### Notes
- テスト: 685 件

---

## [v11.4.0] — 2026-06-06

### Added
- `emit_python.rs` `!AWS` → boto3 変換
  - `AWS.s3_put_object_raw(bucket, key, body)` → `boto3.client("s3").put_object(Bucket=..., Key=..., Body=...)`
  - `AWS.s3_get_object_raw(bucket, key)` → `boto3.client("s3").get_object(Bucket=..., Key=...)["Body"].read()`
- `emit_python.rs` `needs_boto3` フラグ追加（2-pass 検出）
- `pyproject.toml` 生成時に `import boto3` 検出 → `boto3>=1.34` 依存を自動追加
- `driver.rs` `v11400_tests` — 4 件

### Notes
- テスト: 679 件

---

## [v11.3.0] — 2026-06-06

### Added
- `emit_python.rs` `!IO` → Python 標準 I/O 変換
  - `IO.println(s)` → `print(s)`
  - `IO.read_file_raw(path)` → `open(path).read()`（try/except で `Result` を模倣）
  - `Csv.parse_raw(text, ",", true)` → `csv.DictReader` 変換ヘルパー生成
  - `Schema.adapt(raw, "T")` → dataclass 変換ヘルパー生成（`_adapt_T(d) -> T`）
  - `Schema.to_json_array(rows, "T")` → `json.dumps([asdict(r) for r in rows])`
- `driver.rs` `v11300_tests` — 4 件

### Notes
- テスト: 675 件

---

## [v11.2.0] — 2026-06-06

### Added
- `emit_python.rs` `stage` / `seq` → Python パイプライン変換
  - `stage Foo: A -> B !Eff = |x| { ... }` → `def foo(x: A) -> B: ...`（エフェクトはコメント）
  - `seq Pipeline = A |> B |> C` → `def pipeline(x): return c(b(a(x)))`
- `fn main()` → `if __name__ == "__main__": main()`
- `IO.argv()` → `sys.argv[1:]`
- `List.map` / `List.filter` / `List.length` → Python リスト内包表記 / `filter` / `len`
- `driver.rs` `v11200_tests` — 4 件

### Notes
- テスト: 671 件

---

## [v11.1.0] — 2026-06-06

### Added
- `src/emit_python.rs` 新規作成 — Favnir AST → Python コード生成基盤
  - 型定義（`type Foo = { ... }`）→ `@dataclass class Foo`
  - 基本式（Int / Float / String / Bool / List / if-else / binary ops）→ Python 式
  - `fn` → `def`（引数型・戻り型をコメントで保持）
  - `bind x <- expr` → `x = expr`（モナド脱糖）
  - `match` → `if/elif/else`（Option / Result パターン）
- `fav transpile --target python <file>` CLI エントリ（`cli.fav` の `CmdTranspile` + `driver.rs` の `cmd_transpile`）
- `driver.rs` `v11100_tests` — 4 件

### Notes
- テスト: 667 件

---

## [v11.0.0] — 2026-06-05

### Added
- `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` を区別表示（`lineage.rs` `collect_snowflake_call_kinds`）
- `site/content/docs/runes/snowflake.mdx` — Snowflake Rune リファレンスページ

### Changed
- README.md の Rune エコシステム表に `snowflake`（`!Snowflake` エフェクト）を追加
- CHANGELOG に v10.1.0〜v10.9.0 の全履歴を追記

### Notes
- テスト: 1286 件（lineage Snowflake 区別テスト 3 件追加）

---

## [v10.9.0] — 2026-06-05

### Added
- `infra/e2e-demo/snowflake/` — Snowflake E2E デモ（demo.fav 4 ステージ・Terraform・scripts/run.sh・README）
- `driver.rs` `v10900_tests::snowflake_e2e_demo_structure` — ファイル存在確認テスト

### Notes
- テスト: 1283 件

---

## [v10.8.0] — 2026-06-04

### Added
- `fav infer --from snowflake --table <name>` — Snowflake INFORMATION_SCHEMA から Favnir 型定義を自動生成
- `Snowflake.infer_table_raw` VM primitive
- `cli.fav` `CmdInferSnowflake` / `parse_infer_cmd` / `run_infer_snowflake`
- Snowflake 型マッピング（NUMBER→Int / FLOAT→Float / VARCHAR→String / BOOLEAN→Bool / nullable→Option<T>）

### Notes
- テスト: 1282 件（型マッピングテスト 6 件追加）

---

## [v10.7.0] — 2026-06-04

### Added
- `toml.rs` `SnowflakeTomlConfig` — `fav.toml` `[snowflake]` セクション解析（account / user / warehouse / role / database / schema）
- `expand_env_vars` — `${VAR_NAME}` 形式の環境変数参照を展開
- `inject_snowflake_config` — 実行時に Snowflake 設定を環境変数に注入（上書きなし）
- `fav new` テンプレートに `[snowflake]` コメントアウト例を追加

### Notes
- テスト: 1276 件

---

## [v10.6.0] — 2026-06-04

### Added
- `runes/snowflake/` — Snowflake Rune 実装（`execute` / `query<T>`）
- `rune.toml` / `snowflake.fav` / `client.fav` / `snowflake.test.fav`

### Notes
- テスト: 1272 件

---

## [v10.5.0] — 2026-06-04

### Added
- `compiler.fav` builtin NS リストに `"Snowflake"` を追加（2 箇所）
- Favnir pipeline で `Snowflake.*` を含む stage がコンパイル可能になった

### Notes
- テスト: 1271 件

---

## [v10.4.0] — 2026-06-04

### Added
- `checker.fav` に `snowflake_fn` 追加（`execute_raw` / `query_raw` 型シグネチャ）
- `builtin_ret_ty` / `ns_to_effect` に Snowflake エントリ追加
- E0320 エラーコード（`!Snowflake` エフェクト未宣言）

### Notes
- テスト: 1269 件

---

## [v10.3.0] — 2026-06-04

### Added
- `Effect::Snowflake` を 8 ファイルに追加（ast / parser / fmt / lineage / driver / ast_lower_checker / checker / reachability）
- `require_snowflake_effect` (E0314) — `!Snowflake` 未宣言 stage での Snowflake.* 呼び出しを検出

### Notes
- テスト: 1267 件

---

## [v10.2.0] — 2026-06-04

### Added
- `Snowflake.execute_raw` / `Snowflake.query_raw` VM primitive（Snowflake SQL API v2 REST + JWT RS256 認証）
- `snowflake_read_env` / `snowflake_generate_jwt` / `snowflake_api_post` ヘルパー（`vm.rs`）

### Notes
- テスト: 1264 件

---

## [v10.1.0] — 2026-06-04

### Added
- `infra/snowflake/` — Snowflake Terraform インフラ（provider / warehouse / database / schema / role / RSA キー / SSM）
- `infra/snowflake/README.md`

### Notes
- テスト: 1261 件

---

## [v10.0.0] — 2026-06-03

### Added
- `fav new <name>` — プロジェクトスキャフォールディング（fav.toml / src/main.fav / .gitignore 生成）
- `IO.make_dir_raw` VM primitive（ディレクトリ作成）
- GitHub Actions CI に self-check ステップ追加（fav check / fav lint / fav fmt --check）
- `CONTRIBUTING.md` を現状に合わせて更新

### Notes
- テスト: 1260 件（fav_new 統合テスト 2 件追加）

---

## [v9.13.0] — 2026-06-03

### Added
- `par [A, B] |> Merge` — 並列 stage 実行（`std::thread::spawn` VM スレッド並列化）
- E0016（par ステップ入力型不一致）/ E0017（par 内未定義 stage）
- `compiler.fav` / `checker.fav` に `SeqStep` / `SeqDef` / `IStage` / `ISeq` 型追加
- `ast_lower_checker.rs` に `lower_trf_def` / `lower_flw_def` / `te_to_string` 追加

### Notes
- テスト: 1258 件

---

## [v9.12.0] — 2026-06-02

### Added
- `interface` / `impl ... for` / `type T with Iface` を `checker.fav` / `compiler.fav` でセルフホスト対応
- E0014（MissingImpl）/ E0015（ImplMethodNotFound）
- LSP: Rune 定義ジャンプ（`textDocument/definition`）

### Notes
- テスト: 1251 件

---

## [v9.11.0] — 2026-06-01

### Added
- LSP: フィールド補完・モジュール補完（`List.` / `String.` 等）・Rune 補完
- LSP: Signature help（関数呼び出し時の型シグネチャ表示）
- `textDocument/completion` / `textDocument/signatureHelp` ハンドラ

### Notes
- テスト: 1240 件

---

## [v9.10.0] — 2026-05-31

### Added
- `fav repl` — インタラクティブ REPL（式評価・定義累積・`:type` / `:reset` / `:env`）
- `cmd_repl` in `cli.fav`

### Notes
- テスト: 1220 件

---

## [v9.9.0] — 2026-05-31

### Added
- `fav profile` — stage 別実行時間計測（`--profile` フラグ）
- `fav watch` — ファイル監視 + 自動再実行（500ms ポーリング）

### Notes
- テスト: 1217 件

---

## [v9.8.0] — 2026-05-31

### Added
- `fav doc` — `///` ドキュメントコメント + 型シグネチャから Markdown 自動生成
- `cmd_doc` in `cli.fav`、`doc_item` / `doc_program` in `compiler.fav`

### Notes
- テスト: 1213 件

---

## [v9.7.5] — 2026-05-31

### Added
- `where` バリデーター（`type Email(String) where |v| String.contains(v, "@")`）
- E0013（`expr?` を非 Result 関数内で使用）

### Fixed
- Float シリアライズで整数値に小数点が付かないバグを修正

### Notes
- テスト: 1206 件

---

## [v9.7.0] — 2026-05-31

### Added
- 名目型ラッパー `type Name(Inner)` — コンストラクタ・パターンマッチ対応
- `T?` / `T!` / `??` / `expr?` を self-hosted pipeline で対応
- `with Eq, Show, Serialize, Deserialize` 自動合成

### Notes
- テスト: 1200 件

---

## [v9.6.0] — 2026-05-31

### Added
- `!Llm` エフェクト追加
- `llm` Rune — `llm.complete` / `llm.chat` / `llm.extract<T>`（Claude / OpenAI 対応）
- `LLM_PROVIDER` / `LLM_MODEL` 環境変数で切り替え

### Notes
- テスト: 1191 件

---

## [v9.5.0] — 2026-05-31

### Added
- `!Http` エフェクト追加
- `http` Rune 拡張（`get_text` / `get_json<T>` / `post_json_typed<T,R>`）
- `grpc` Rune 拡張・`graphql` Rune 新規作成

### Notes
- テスト: 1187 件

---

## [v9.4.0] — 2026-05-31

### Added
- `json` Rune — `encode<T>` / `decode<T>` / `pretty`
- `csv` Rune 拡張 — `read<T>` / `write_file<T>`
- `gen` Rune 拡張 — `uuid` / `uuid_v7` / `nano_id`
- W004 lint ルール（`fn` の引数が 4 個以上 → レコード型推奨）

### Notes
- テスト: 1182 件

---

## [v9.3.0] — 2026-05-31

### Added
- `fav lint` — W001〜W005 静的解析ルールエンジン（compiler.fav + cli.fav）
- W001（EffectlessSink）/ W002（NoWriteInSeq）/ W003（UnusedBinding）/ W005（WildcardOnlyMatch）

### Notes
- テスト: 1173 件

---

## [v9.2.0] — 2026-05-31

### Added
- `fav fmt` — コードフォーマッタ（compiler.fav の pretty printer、冪等性保証）
- `Compiler.fmt_source_raw` VM primitive
- `--check` フラグ（CI 向け）

### Notes
- テスト: 1167 件

---

## [v9.1.0] — 2026-05-31

### Added
- stdlib 大幅拡充（`List.chunk` / `flat_map` / `group_by` / `zip_with` / `unique` 等 30 関数超）
- `rvm` 独立バイナリ（`src/bin/rvm.rs`）
- マルチパラメータクロージャ `|x, y| x + y` 対応
- E0012（非ジェネリック関数引数数不一致）

### Notes
- テスト: 1162 件

---

## [v9.0.0] — 2026-05-31

### Changed
- **セルフホスト完成宣言**: `fav run` / `fav check` の全経路が Favnir pipeline 経由で動作
- `--legacy` フラグ非推奨化

### Notes
- テスト: 1136 件

---

## [v7.0.0] — 2026-05-27

### Added
- `Effect::DbRead` / `Effect::DbWrite` / `Effect::DbAdmin` を型システムに追加（`ast.rs`）
- `parser.rs`：`!DbRead` / `!DbWrite` / `!DbAdmin` のパースに対応
- `checker.rs`：BUILTIN_EFFECTS 更新、`require_db_write_effect` / `require_db_admin_effect` 追加
- `reachability.rs`：3 エフェクトのリーチャビリティ追跡に対応
- `fmt.rs` / `driver.rs`：3 エフェクトの表示・JSON 出力に対応
- `runes/db/query.fav`：query 系 → `!DbRead`、execute 系 → `!DbWrite` に更新
- `runes/db/transaction.fav`：`!Db` → `!DbWrite` に更新
- `runes/db/migration.fav`：`applied_migrations` → `!DbRead`、`mark_applied` / `ensure_migrations_table` → `!DbAdmin` に更新
- `site/content/docs/guides/schema-authority.mdx`：Schema Authority 全体ワークフローガイド新規作成
- `site/content/docs/runes/db.mdx`：エフェクト細分化テーブル追記

### Changed
- `require_db_effect`：後方互換化（`Db | DbRead | DbWrite | DbAdmin` をすべて受け入れる）

### Notes
- テスト: 1044 件（パーサーテスト +1）
- 後方互換: 既存の `!Db` を使ったコードは変更なしに動く

---

## [v6.9.0] — 2026-05-27

### Added
- `LICENSE`（MIT）をリポジトリルートに配置
- `CONTRIBUTING.md`：ビルド手順・テスト手順・PR ガイドライン・Rune 追加ガイド
- `CHANGELOG.md`（本ファイル）
- CI に `cargo clippy -- -D warnings` を追加

---

## [v6.8.0] — 2026-05-27

### Added
- `site/content/docs/runes/db.mdx`：db rune リファレンス（connect / query / paginate / batch_insert / with_transaction / savepoint）
- `site/content/docs/runes/http.mdx`：http rune リファレンス（GET/POST/PUT/DELETE/PATCH / retry / bearer/basic/api_key）
- `site/content/docs/runes/duckdb.mdx`：query_one / explain / Parquet・CSV IO セクション追記（read_parquet / read_csv / write_parquet / write_csv）

---

## [v6.6.0] — 2026-05-27

### Added
- `one_of` 制約：`schemas/*.yaml` で列挙値バリデーションが可能に
- `TypeName.validate(record)`：VM 動的 dispatch による型付きバリデーション
- `Validate.rows_raw(type_name, rows)`：複数行一括バリデーション builtin
- 統合テスト 10 件追加（`vm_stdlib_tests.rs`）、合計 1043 件

### Changed
- `schema.mdx`：preview Note を削除、`Order.validate` / `Validate.rows_raw` のコード例を追加

---

## [v6.5.0] — 2026-05-27

### Added
- `site/content/docs/language/pipeline.mdx`：stage / seq / `|>` / abstract stage・seq / fav explain ドキュメント
- `site/content/docs/language/schema.mdx`：schemas/*.yaml 構文・制約一覧・T.validate・fav build --schema ドキュメント
- `site/content/docs/stdlib/infer.mdx`：fav infer --csv / --db / --proto / --out ドキュメント
- `site/content/docs/rune-cli.mdx`：fav deploy（Lambda）/ fav build --schema セクション追記

---

## [v6.4.0] — 2026-05-27

### Added
- `scripts/build-wasm.sh`：wasm-pack build → `site/public/wasm/` 自動出力
- WASM バックエンドで `List` 型対応（list_singleton / list_length / list_is_empty）
- Playground サンプルを stage/seq パイプライン例に更新

### Fixed
- WASM メモリ設定（`minimum: 2` / 128KB）で heap が有効化

---

## [v6.3.0] — 2026-05-26

### Added
- `compiler.fav` に `stage` / `seq` / `|>` のパース・lowering を追加
- `bootstrap_stage_seq_self_host_executes_correctly` テスト追加

---

## [v6.2.0] — 2026-05-25

### Added
- Bootstrap 3 段階検証確立（Stage1→Stage2→Stage3、`bytecode_A == bytecode_B`）
- 新オペコード 5 種：`CallNamed` / `JumpIfNotVariantC` / `GetFieldC` / `BuildRecordC` / `MakeClosureN`
- `String.to_bytes`（`-> List<Int>`）

### Fixed
- self-host 成熟度ドキュメント整備（`semantic_gap_audit.md` 等）

---

## [v6.1.0] — 2026-05-24

### Added
- `compiler.fav`：lexer.fav / parser.fav / codegen.fav をフルインライン化
- `bootstrap_stage1_builds_and_serializes` テスト追加

### Fixed
- `codegen.rs`：`remap_string_operands` が `Swap`/`TrackLine` を未認識で中断するバグを修正

---

## [v6.0.0] — 2026-05-21

### Added
- セルフホストコンパイラ完成（`fav/self/compiler.fav`）
- Favnir 製レキサー（`lexer.fav`）・パーサー（`parser.fav`）・型チェッカー（`checker.fav`）・コード生成器（`codegen.fav`）
- `IO.argv` / `List.take_while` / `List.drop_while` / `List.singleton` を VM に追加
- Bootstrap Stage1 実行テスト群

### Fixed
- `JumpIfNotVariant`：`VMValue::VariantCtor`（引数なしバリアント）のパターンマッチが正しく動作しないバグを修正

---

## [v5.5.0] — 2026-05

### Added
- `Map.remove` / `Map.contains_key` / `String.from_chars`
- `Option.and_then` / `Result.and_then`

---

## [v5.0.0〜v5.4.0] — 2026-05

### Added
- AWS Lambda / S3 / SQS 本番稼働
- SigV4 認証（セッショントークン対応）
- CloudFront + S3 リファレンスサイト公開
- `fav deploy`（Lambda）実装
- Import 解決順序：`rune_modules/` → `runes/` → `~/.fav/registry/`

---

## [v4.12.0〜v4.1.0] — 2025〜2026

### Added
- Rune エコシステム構築：aws / duckdb / db / http / auth / log / env / gen / grpc / json / parquet / csv / incremental / stat / validate
- `fav test` / `fav bench` / `fav check` / `fav run` CLI
- `fav explain`（パイプライン可視化）/ `fav infer`（型推論）/ `fav build --schema`（DDL 生成）
- `stage` / `seq` / `|>` パイプライン構文
- `abstract stage` / `abstract seq`（依存注入）
- パターンマッチ（ネスト・ガード・バリアント）
- `collect` / `yield` / クロージャ
- ジェネリクス・インターフェース・エフェクト型チェッカー
- バイトコードコンパイラ + VM
- WASM バックエンド（`favnir-wasm`）
- LSP（hover・diagnostics）
- `schemas/*.yaml` によるスキーマ制約システム
- LocalStack 対応（`AWS_ENDPOINT_URL` 切り替え）

---

[v6.9.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.9.0
[v6.8.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.8.0
[v6.6.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.6.0
[v6.5.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.5.0
[v6.4.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.4.0
[v6.3.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.3.0
[v6.2.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.2.0
[v6.1.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.1.0
[v6.0.0]: https://github.com/kazuma0606/favnir/releases/tag/v6.0.0
