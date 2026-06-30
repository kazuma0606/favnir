# v29.8.0 Plan — ドキュメントサイト v3

**バージョン**: 29.8.0
**日付**: 2026-06-30
**前バージョン**: v29.7.0 (VS Code 拡張 公式リリース)

---

## 実装手順

### T1: Cargo.toml version 更新

```toml
version = "29.8.0"
```

### T2: cookbook 29 本追加（site/content/cookbook/）

各ファイルは「タイトル・概要・コード例・関連 Rune リンク」の最小構成で作成する。

#### ETL 基礎（4 本）

**s3-to-parquet.mdx**
```markdown
# S3 CSV → Parquet 変換

S3 上の CSV ファイルを Parquet 形式に変換して再保存します。

...（コード例）...
```

**delta-lake-upsert.mdx** / **jsonl-processing.mdx** / **sqlite-etl.mdx** — 同形式

各ファイルの最小要件: タイトル（`# ファイル概要`）+ 対応サービス名（大文字、英語）を本文に含む + `import runes/XXX` コード例

#### ストリーミング（5 本）

**kafka-consumer.mdx** — 本文に `Kafka` を含む（テスト対象）
**kinesis-archiver.mdx** — 本文に `Kinesis` を含む
**nats-iot.mdx** — 本文に `NATS` を含む
**rabbitmq-worker.mdx** — 本文に `RabbitMQ` を含む
**sqs-processor.mdx** — 本文に `SQS` を含む

#### DWH 連携（5 本）

**bigquery-load.mdx** / **redshift-copy.mdx** / **clickhouse-bulk.mdx** / **dbt-ref.mdx** / **fav-infer.mdx**

#### 可観測性（5 本）

**prometheus-metrics.mdx** / **datadog-apm.mdx** / **sentry-alerts.mdx** / **grafana-dashboard.mdx** / **otel-trace.mdx**

#### AI/ML（5 本）

**mlflow-experiment.mdx** / **pinecone-rag.mdx** / **vertex-ai-predict.mdx** / **sagemaker-invoke.mdx** / **llm-pipeline.mdx**

#### 実用（5 本）

**github-pr-report.mdx** / **pagerduty-alert.mdx** / **slack-notify.mdx** / **email-alert.mdx** / **multi-cloud-etl.mdx**

### T3: site/app/community/page.tsx 作成

```tsx
export default function CommunityPage() {
  return (
    <main>
      <h1>Favnir Community</h1>
      <p>Join the Favnir community. We discuss ideas, share Rune recipes, and support each other.</p>
      <ul>
        <li>
          <a href="https://github.com/favnir/favnir/discussions">GitHub Discussions</a>
          {' '}— Q&amp;A, feature requests, and announcements
        </li>
        <li>
          <a href="#">Discord（coming soon）</a>
          {' '}— real-time chat
        </li>
        <li>
          <a href="https://github.com/favnir/favnir/blob/main/CONTRIBUTING.md">Contributing Guide</a>
          {' '}— how to contribute Runes and fixes
        </li>
      </ul>
    </main>
  );
}
```

### T4: CHANGELOG.md に [v29.8.0] セクション追加

```markdown
## [v29.8.0] — 2026-06-30

### Added
- `site/content/cookbook/` — cookbook 29 本追加（3 → 32 本）
- `site/app/community/` — `/community/` ページ新設（GitHub Discussions / Discord リンク）
- テスト数: 2354 → 2360（+6）
```

### T5: benchmarks/v29.8.0.json 作成

```json
{
  "version": "29.8.0",
  "date": "2026-06-30",
  "milestone": "Ecosystem Maturity (phase 8)",
  "test_count": 2360,
  "metrics": {
    "compile_hello_ms": 12,
    "compile_etl_ms": 38,
    "typecheck_ms": 9,
    "vm_run_ms": 4
  }
}
```

### T6: driver.rs に v298000_tests 6 件追加

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

### T7: cargo test --bin fav v298000 — 6/6 PASS 確認

### T8: cargo test --bin fav — 2360 tests PASS 確認

### T9: tasks.md を COMPLETE に更新

---

## テスト数カウント

| バージョン | テスト数 |
|---|---|
| v29.7.0 | 2354 |
| v29.8.0 | **2360** (+6) |
