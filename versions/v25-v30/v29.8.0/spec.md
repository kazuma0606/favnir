# v29.8.0 Spec — ドキュメントサイト v3

**バージョン**: 29.8.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 8)
**前バージョン**: v29.7.0 (VS Code 拡張 公式リリース)

---

## 概要

v24.7.0 で構築したドキュメントサイト v2 を採用フォーカスに再構築する（ドキュメントサイト v3）。
cookbook を現在の 3 本から **30 本以上**に拡充し、`/community/` ページを新設する。

> **ポジショニング**: VS Code 拡張（v29.7）でエディタ体験が整った。
> 次は「Favnir を見つけた人が 30 分で動かせる」体験を提供するドキュメントを揃える。
> cookbook 30 本は「Favnir で何ができるか」を具体例で網羅する最重要コンテンツ。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `site/content/cookbook/*.mdx` | cookbook 29 本追加（3 → 32 本）|
| `site/app/community/page.tsx` | `/community/` ページ新設 |
| `fav/Cargo.toml` | version 29.7.0 → 29.8.0 |
| `CHANGELOG.md` | `[v29.8.0]` セクション追加 |
| `benchmarks/v29.8.0.json` | ベンチマーク記録 |
| `fav/src/driver.rs` | `v298000_tests` 6 件追加 |

---

## サイト構成（v3 完成形）

```
favnir.dev/
├── /                  ← ランディング（既存）
├── /learn/            ← チュートリアル（既存: 3 本）
├── /cookbook/         ← 実用レシピ 32 本（本バージョンで 29 本追加）
├── /runes/            ← Rune ドキュメント（既存）
├── /playground/       ← WASM Playground（既存）
├── /packages/         ← Rune Registry（既存）
├── /bench/            ← ベンチマーク（既存）
├── /spec/             ← 形式的仕様書（既存）
└── /community/        ← GitHub Discussions / Discord（本バージョンで新設）
```

---

## cookbook 追加内訳（29 本）

### ETL 基礎（4 本 — etl-csv-to-db は既存）

| ファイル | 内容 |
|---|---|
| `s3-to-parquet.mdx` | S3 CSV → Parquet 変換 |
| `delta-lake-upsert.mdx` | Delta Lake upsert パイプライン |
| `jsonl-processing.mdx` | JSON Lines ファイル処理 |
| `sqlite-etl.mdx` | SQLite ETL（ローカル開発用）|

### ストリーミング（5 本）

| ファイル | 内容 |
|---|---|
| `kafka-consumer.mdx` | Kafka コンシューマーパイプライン |
| `kinesis-archiver.mdx` | Kinesis → S3 アーカイバー |
| `nats-iot.mdx` | NATS IoT データ収集 |
| `rabbitmq-worker.mdx` | RabbitMQ ワーカーパイプライン |
| `sqs-processor.mdx` | SQS メッセージ処理 |

### DWH 連携（5 本）

| ファイル | 内容 |
|---|---|
| `bigquery-load.mdx` | BigQuery データロード |
| `redshift-copy.mdx` | Redshift COPY コマンド連携 |
| `clickhouse-bulk.mdx` | ClickHouse バルクインサート |
| `dbt-ref.mdx` | dbt モデル参照 |
| `fav-infer.mdx` | `fav infer` でスキーマ自動推論 |

### 可観測性（5 本）

| ファイル | 内容 |
|---|---|
| `prometheus-metrics.mdx` | Prometheus メトリクス送信 |
| `datadog-apm.mdx` | Datadog APM トレース |
| `sentry-alerts.mdx` | Sentry エラーレポート |
| `grafana-dashboard.mdx` | Grafana ダッシュボード連携 |
| `otel-trace.mdx` | OpenTelemetry トレーシング |

### AI/ML（5 本）

| ファイル | 内容 |
|---|---|
| `mlflow-experiment.mdx` | MLflow 実験管理 |
| `pinecone-rag.mdx` | Pinecone RAG パイプライン |
| `vertex-ai-predict.mdx` | Vertex AI オンライン推論 |
| `sagemaker-invoke.mdx` | SageMaker エンドポイント推論 |
| `llm-pipeline.mdx` | LLM テキスト生成パイプライン |

### 実用（5 本）

| ファイル | 内容 |
|---|---|
| `github-pr-report.mdx` | GitHub PR にデータ品質レポートを投稿 |
| `pagerduty-alert.mdx` | PagerDuty アラート自動化 |
| `slack-notify.mdx` | Slack 通知パイプライン |
| `email-alert.mdx` | メールアラート送信 |
| `multi-cloud-etl.mdx` | マルチクラウド ETL（S3 + GCS + Azure）|

---

## テスト戦略

### v298000_tests（6 件）

| テスト名 | 検証内容 | カテゴリ |
|---|---|---|
| `cookbook_kafka_consumer_exists` | `site/content/cookbook/kafka-consumer.mdx` が存在し `Kafka` を含む | ストリーミング |
| `cookbook_pinecone_rag_exists` | `site/content/cookbook/pinecone-rag.mdx` が存在し `Pinecone` を含む | AI/ML |
| `cookbook_pagerduty_alert_exists` | `site/content/cookbook/pagerduty-alert.mdx` が存在し `PagerDuty` を含む | 実用 |
| `cookbook_prometheus_metrics_exists` | `site/content/cookbook/prometheus-metrics.mdx` が存在し `Prometheus` を含む | 可観測性 |
| `community_page_exists` | `site/app/community/page.tsx` が存在し `GitHub Discussions` を含む | /community/ |
| `changelog_has_v29_8_0` | `CHANGELOG.md` に `[v29.8.0]` が存在する | — |

ストリーミング・AI/ML・実用・可観測性の 4 カテゴリを直接テストにより確認。
ETL 基礎・DWH 連携の 2 カテゴリは ファイル作成によって存在確認できるが、テストスロットを community と changelog に割り当てる。

テスト数: 2354 → **2360**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "29.8.0"
- [ ] `site/content/cookbook/` が 32 本（既存 3 本 + 追加 29 本、ロードマップ要件「30 本以上」を満たす）
- [ ] `site/app/community/page.tsx` が存在する
- [ ] `CHANGELOG.md` に `[v29.8.0]` セクションあり
- [ ] `benchmarks/v29.8.0.json` 存在（test_count: 2360）
- [ ] `cargo test --bin fav v298000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2360 tests PASS

---

## ロードマップとの差異

### 「8 ページ存在」条件について

ロードマップ v29.8 完了条件の「8 ページがすべて存在し公開済み」のうち、
`/`・`/learn/`・`/runes/`・`/playground/`・`/packages/`・`/bench/`・`/spec/` の 7 ページは
v24.7.0 以前に作成・確認済みのため、本バージョンのテストでは新設の `/community/` のみ確認する。

### cookbook カウントについて

ロードマップ v29.8 は「ETL 基礎（5 本）」と記載しているが、うち 1 本（`etl-csv-to-db.mdx`）は既存。
本バージョンでは新規追加 4 本として計上し、合計 32 本（= 既存 3 + 新規 29）となる。

### cookbook ナビゲーションについて

`/cookbook/` ページのナビゲーションは既存 `site/app/` のルーティングで自動的に解決されるため、
追加の index ページは不要。

## スコープ外

- cookbook の実際の動作確認（`fav run` による E2E 実行）— コンテンツ検証のみ
- `/community/` ページへの実際の Discussions / Discord リンク埋め込み（URL 確定後に更新）
- ランディングページのデモ動画埋め込み — 動画制作が別途必要
- Playground（WASM）のインタラクティブ化強化 — v30.x+ で対応
- cookbook の自動生成 / CMS 連携 — 手動 MDX 管理を継続
