# v28.8.0 Plan — オブザーバビリティ E2E デモ（datadog APM）

## 実装順序

```
T1: Cargo.toml version bump (28.7.0 → 28.8.0)
T2: examples/observability/datadog_apm.fav 更新（3 stage + // #[trace] コメント）
T3: examples/observability/docker-compose.yml 更新（datadog-agent 追加）
T4: site/content/docs/tools/datadog-apm.mdx 新規作成
T5: CHANGELOG.md に [v28.8.0] セクション追加
T6: benchmarks/v28.8.0.json 新規作成
T7: driver.rs に v288000_tests 8 件追加
T8: cargo test --bin fav v288000 — 8/8 PASS 確認
T8.5: cargo test --bin fav datadog — 既存テスト含め PASS 確認（datadog_example_has_pipeline 維持）
T9: cargo test --bin fav 全体 — 2297 PASS 確認
T10: tasks.md を COMPLETE に更新
```

---

## T2: datadog_apm.fav の更新内容

既存ファイル（v28.2.0 stub）を以下に置き換える。
**`seq DatadogApmDemo` は維持**（`datadog_example_has_pipeline` テストが `DatadogApmDemo` をチェック）。

```favnir
import runes/datadog

// #[trace(service: "etl-extractor")] — v29.0+ でコンパイラが自動挿入予定
stage ExtractEvents: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Datadog.metric("etl.extract.rows", 200.0, "pipeline:etl,stage:extract")
    bind _ <- Datadog.trace("etl-extractor", "extract_events")
    Result.ok(unit)
}

// #[trace(service: "etl-transformer")]
stage TransformEvents: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Datadog.metric("etl.transform.duration_ms", 38.5, "pipeline:etl,stage:transform")
    bind _ <- Datadog.log("info", "Transform complete", "{\"rows\": 200}")
    Result.ok(unit)
}

// #[trace(service: "etl-loader")]
stage LoadEvents: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Datadog.metric("etl.load.rows", 200.0, "pipeline:etl,stage:load")
    bind _ <- Datadog.event("ETL Complete", "All stages finished successfully", "pipeline:etl")
    bind _ <- Datadog.service_check("etl.pipeline", "OK")
    Result.ok(unit)
}

seq DatadogApmDemo = ExtractEvents |> TransformEvents |> LoadEvents
```

**注意**: 既存の `import rune "datadog"` 構文を `import runes/datadog` に修正する。
`seq DatadogApmDemo` を確認し維持する（`seq DatadogAPMDemo` への変更は不可）。

---

## T3: docker-compose.yml 更新

既存の grafana サービスの後に追記する:

```yaml
  datadog-agent:
    image: datadog/agent:7
    environment:
      - DD_API_KEY=${DD_API_KEY:-dummy-key-for-local-mode}
      - DD_SITE=datadoghq.com
      - DD_APM_ENABLED=true
      - DD_LOGS_ENABLED=true
      - DD_DOGSTATSD_NON_LOCAL_TRAFFIC=true
    ports:
      - "8125:8125/udp"
      - "8126:8126"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
```

`DD_API_KEY` はデフォルト値 `dummy-key-for-local-mode` を使用し、
本番では `.env` ファイルや環境変数で上書きする設計。ハードコードなし。

---

## T4: site/content/docs/tools/datadog-apm.mdx

- frontmatter: `title: Datadog APM Demo`, `description: Datadog APM による ETL トレース E2E デモ（v28.8.0）`
- `DatadogApmDemo` seq のコード例掲載
- `// #[trace(service: "...")]` の将来バージョン（v29.0+）での自動化について
- Docker Compose セットアップ手順（`DD_API_KEY` 環境変数の設定方法含む）
- Datadog APM URL（`https://app.datadoghq.com/apm/traces`）への言及

---

## T7: driver.rs — v288000_tests

```rust
// ── v288000_tests (v28.8.0) — datadog APM E2E デモ ────────────────────────────
#[cfg(test)]
mod v288000_tests {
    // include_str! のみ使用のため use super::* 不要
    #[test]
    fn datadog_apm_example_has_extract_events_stage() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("stage ExtractEvents"), "datadog_apm.fav must define stage ExtractEvents");
    }
    #[test]
    fn datadog_apm_example_has_transform_events_stage() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("stage TransformEvents"), "datadog_apm.fav must define stage TransformEvents");
    }
    #[test]
    fn datadog_apm_example_has_load_events_stage() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("stage LoadEvents"), "datadog_apm.fav must define stage LoadEvents");
    }
    #[test]
    fn datadog_apm_example_has_trace_annotation() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("// #[trace(service:"), "datadog_apm.fav must contain // #[trace(service: annotation in comment form");
    }
    #[test]
    fn datadog_apm_example_uses_datadog_metric() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("Datadog.metric"), "datadog_apm.fav must use Datadog.metric");
    }
    #[test]
    fn docker_compose_has_datadog_agent() {
        let src = include_str!("../../examples/observability/docker-compose.yml");
        assert!(src.contains("datadog-agent"), "docker-compose.yml must define datadog-agent service");
    }
    #[test]
    fn datadog_apm_doc_exists() {
        let src = include_str!("../../site/content/docs/tools/datadog-apm.mdx");
        assert!(src.contains("DatadogApmDemo"), "datadog-apm.mdx must mention DatadogApmDemo");
    }
    #[test]
    fn changelog_has_v28_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.8.0]") || src.contains("## v28.8.0"), "CHANGELOG.md must contain '[v28.8.0]'");
    }
}
```

`include_str!` パス一覧:
- `../../examples/observability/datadog_apm.fav`
- `../../examples/observability/docker-compose.yml`
- `../../site/content/docs/tools/datadog-apm.mdx`
- `../../CHANGELOG.md`

---

## 既存テスト保護（T8.5）

`cargo test --bin fav datadog` を実行し、v282000_tests の以下が引き続き PASS することを確認:
- `datadog_example_has_pipeline`（`DatadogApmDemo` seq 維持により PASS）
- その他 `datadog_*` テスト群
