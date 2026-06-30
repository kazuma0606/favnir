# v28.8.0 Spec — オブザーバビリティ E2E デモ（datadog APM）

## 概要

マイクロサービス連携パイプラインのトレースを Datadog APM で可視化する E2E デモを整備する。

`examples/observability/datadog_apm.fav` は v28.2.0 で stub として作成済み。
v28.8.0 では以下を実施する:
1. `datadog_apm.fav` を 3 ステージ構成（ExtractEvents / TransformEvents / LoadEvents）に拡充
2. `// #[trace(service: "...")]` アノテーションをコメント形式で追加
3. `docker-compose.yml` に Datadog Agent サービスを追加
4. `site/content/docs/tools/datadog-apm.mdx` 新規作成

**新規コンパイラ機能なし**。`checker.fav` 更新なし。

> **`#[trace]` の扱い**: v28.3.0 で OTel Rune は実装済みだが `#[trace]` のコンパイラ自動挿入は
> v29.0+ 予定。v28.8.0 では `// #[trace(service: "...")]` のコメント形式で記述する。
> （v28.7.0 の `#[track]` と同じ方針）

---

## 既存ファイルとの整合

`examples/observability/datadog_apm.fav` の v28.2.0 時点の内容:
- `seq DatadogApmDemo`（既存 `datadog_example_has_pipeline` テストが確認）
- 2 stage: `TraceExtract` / `ReportToDatadog`

v28.8.0 での変更:
- `seq DatadogApmDemo` は **維持**（既存テスト破壊防止）
- stage を **3 段構成**（ExtractEvents / TransformEvents / LoadEvents）に置き換え
- `import rune "datadog"` → `import runes/datadog` に修正
- 各 stage に `// #[trace(service: "...")]` コメントを追加

---

## ステージ名について

ロードマップ v28.8 のサンプルコードは `Extract / Transform / Load`（型: `Config -> List<Event> !Db` 等）を使用しているが、
v28.8.0 はコンパイラ機能なしのデモのため、stage 名を `ExtractEvents / TransformEvents / LoadEvents` に変更し
型を `Unit -> Result<Unit, String> !Io` に統一する。
（ロードマップのサンプルは概念的例示であり、ステージ名・型は実装で変更可）

## datadog_apm.fav の新設計

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

---

## docker-compose.yml 更新

`examples/observability/docker-compose.yml` の既存 grafana サービスの後に Datadog Agent を追加:

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

> **セキュリティ注記**: `DD_API_KEY=${DD_API_KEY:-dummy-key-for-local-mode}` はシェルの `:-` 演算子でデモ用ローカルモード専用のデフォルト値を設定している。本番環境では `DD_API_KEY` 環境変数または `.env` ファイルで上書きすること。
> 既存の `GF_SECURITY_ADMIN_PASSWORD=admin`（v28.7.0 で追加）はデモ専用のまま維持し、本バージョンでは変更しない。

---

## 追加ファイル

| ファイル | 内容 |
|---|---|
| `examples/observability/datadog_apm.fav` | **更新**（3 stage + `// #[trace]` コメント）|
| `examples/observability/docker-compose.yml` | **更新**（datadog-agent サービス追加）|
| `site/content/docs/tools/datadog-apm.mdx` | 新規作成（APM デモ解説）|
| `benchmarks/v28.8.0.json` | 新規作成（test_count: 2297）|
| `CHANGELOG.md` | `[v28.8.0]` セクション追加 |

---

## テスト一覧（driver.rs v288000_tests）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `datadog_apm_example_has_extract_events_stage` | `datadog_apm.fav` に `stage ExtractEvents` を含む |
| 2 | `datadog_apm_example_has_transform_events_stage` | `datadog_apm.fav` に `stage TransformEvents` を含む |
| 3 | `datadog_apm_example_has_load_events_stage` | `datadog_apm.fav` に `stage LoadEvents` を含む |
| 4 | `datadog_apm_example_has_trace_annotation` | `datadog_apm.fav` に `// #[trace` を含む |
| 5 | `datadog_apm_example_uses_datadog_metric` | `datadog_apm.fav` に `Datadog.metric` を含む |
| 6 | `docker_compose_has_datadog_agent` | `docker-compose.yml` に `datadog-agent` を含む |
| 7 | `datadog_apm_doc_exists` | `site/content/docs/tools/datadog-apm.mdx` に `DatadogApmDemo` を含む |
| 8 | `changelog_has_v28_8_0` | `CHANGELOG.md` に `[v28.8.0]` または `## v28.8.0` を含む |

合計 8 テスト。test_count: **2297**（2289 + 8）

---

## 完了条件チェックリスト

- [ ] `Cargo.toml` version = `28.8.0`
- [ ] `examples/observability/datadog_apm.fav` 更新（3 stage、`// #[trace(service:` コメント、`import runes/datadog`、`seq DatadogApmDemo` 維持）
- [ ] `examples/observability/docker-compose.yml` 更新（`datadog-agent` サービス追加）
- [ ] `site/content/docs/tools/datadog-apm.mdx` 存在（`DatadogApmDemo` 言及）
- [ ] `CHANGELOG.md` に `[v28.8.0]` セクションあり
- [ ] `benchmarks/v28.8.0.json` 存在（test_count: 2297）
- [ ] `cargo test --bin fav v288000` — 8/8 PASS
- [ ] `cargo test --bin fav datadog` — 既存テスト含め PASS（`datadog_example_has_pipeline` 維持確認）
- [ ] `cargo test --bin fav` — 2297 tests PASS
- [ ] （手動確認）`DD_API_KEY=xxx docker compose up -d` で datadog-agent 起動
- [ ] （手動確認）`fav run examples/observability/datadog_apm.fav` がエラーなく実行
