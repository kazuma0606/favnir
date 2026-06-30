# v29.8.0 Tasks — ドキュメントサイト v3

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-07-01

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.7.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2354 passed` を含むこと
- [x] `driver.rs` に `mod v298000_tests` が存在しないこと
- [x] `site/content/cookbook/` のファイル数が 3 本（追加前）であること

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.7.0` → `29.8.0` | [x] |
| T2 | cookbook 29 本追加（`site/content/cookbook/*.mdx`）| [x] |
| T3 | `site/app/community/page.tsx` 作成 | [x] |
| T4 | `CHANGELOG.md` に `[v29.8.0]` セクション追加 | [x] |
| T5 | `benchmarks/v29.8.0.json` 作成（test_count: 2360）| [x] |
| T6 | `driver.rs` に `v298000_tests` 6 件追加 | [x] |
| T7 | `cargo test --bin fav v298000` — 6/6 PASS 確認 | [x] |
| T8 | `cargo test --bin fav` — 2360 tests PASS 確認 | [x] |
| T9 | tasks.md を COMPLETE に更新 | [x] |

---

## T2 追加ファイル一覧（29 本）

### ETL 基礎（4 本）
- `s3-to-parquet.mdx`
- `delta-lake-upsert.mdx`
- `jsonl-processing.mdx`
- `sqlite-etl.mdx`

### ストリーミング（5 本）
- `kafka-consumer.mdx`
- `kinesis-archiver.mdx`
- `nats-iot.mdx`
- `rabbitmq-worker.mdx`
- `sqs-processor.mdx`

### DWH 連携（5 本）
- `bigquery-load.mdx`
- `redshift-copy.mdx`
- `clickhouse-bulk.mdx`
- `dbt-ref.mdx`
- `fav-infer.mdx`

### 可観測性（5 本）
- `prometheus-metrics.mdx`
- `datadog-apm.mdx`
- `sentry-alerts.mdx`
- `grafana-dashboard.mdx`
- `otel-trace.mdx`

### AI/ML（5 本）
- `mlflow-experiment.mdx`
- `pinecone-rag.mdx`
- `vertex-ai-predict.mdx`
- `sagemaker-invoke.mdx`
- `llm-pipeline.mdx`

### 実用（5 本）
- `github-pr-report.mdx`
- `pagerduty-alert.mdx`
- `slack-notify.mdx`
- `email-alert.mdx`
- `multi-cloud-etl.mdx`

---

## テスト詳細（T6）

```rust
// v298000_tests (v29.8.0) -- ドキュメントサイト v3
#[cfg(test)]
mod v298000_tests {
    #[test]
    fn cookbook_kafka_consumer_exists() {
        let src = include_str!("../../site/content/cookbook/kafka-consumer.mdx");
        assert!(
            src.contains("Kafka"),
            "site/content/cookbook/kafka-consumer.mdx must contain 'Kafka'"
        );
    }
    #[test]
    fn cookbook_pinecone_rag_exists() {
        let src = include_str!("../../site/content/cookbook/pinecone-rag.mdx");
        assert!(
            src.contains("Pinecone"),
            "site/content/cookbook/pinecone-rag.mdx must contain 'Pinecone'"
        );
    }
    #[test]
    fn cookbook_pagerduty_alert_exists() {
        let src = include_str!("../../site/content/cookbook/pagerduty-alert.mdx");
        assert!(
            src.contains("PagerDuty"),
            "site/content/cookbook/pagerduty-alert.mdx must contain 'PagerDuty'"
        );
    }
    #[test]
    fn cookbook_prometheus_metrics_exists() {
        let src = include_str!("../../site/content/cookbook/prometheus-metrics.mdx");
        assert!(
            src.contains("Prometheus"),
            "site/content/cookbook/prometheus-metrics.mdx must contain 'Prometheus'"
        );
    }
    #[test]
    fn community_page_exists() {
        let src = include_str!("../../site/app/community/page.tsx");
        assert!(
            src.contains("GitHub Discussions"),
            "site/app/community/page.tsx must contain 'GitHub Discussions'"
        );
    }
    #[test]
    fn changelog_has_v29_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.8.0]") || src.contains("## v29.8.0"),
            "CHANGELOG.md must contain '[v29.8.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.8.0"
- [x] `site/content/cookbook/` が 32 本（既存 3 本 + 追加 29 本、ロードマップ要件「30 本以上」を満たす）
- [x] `site/app/community/page.tsx` が存在する
- [x] `CHANGELOG.md` に `[v29.8.0]` セクションあり
- [x] `benchmarks/v29.8.0.json` 存在（test_count: 2360）
- [x] `cargo test --bin fav v298000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2360 tests PASS
- [x] tasks.md を COMPLETE に更新
