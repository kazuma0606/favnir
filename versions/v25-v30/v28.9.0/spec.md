# v28.9.0 Spec — オブザーバビリティ E2E デモ（sentry アラート）

## 概要

パイプライン失敗時に自動的に Sentry でアラートを受け取る E2E デモを整備する。

`examples/observability/sentry_alerting.fav` は v28.5.0 で stub として作成済み。
v28.9.0 では以下を実施する:
1. `sentry_alerting.fav` に `CriticalLoad` stage を追加（`// #[on_error]` コメント形式）
2. `Sentry.capture_message` の使用例を追加
3. `seq SentryAlertingDemo` を `CriticalLoad |> ReportError |> SetContext` に拡充
4. `docker-compose.yml` に Sentry サービスを追加
5. `site/content/docs/tools/sentry-alerting.mdx` 新規作成

**新規コンパイラ機能なし**。`checker.fav` 更新なし。

> **`#[on_error]` の扱い**: `#[on_error(report_to: "sentry", level: "critical")]` の
> コンパイラ自動挿入は v29.0+ 予定。v28.9.0 では `// #[on_error(...)]` のコメント形式で記述する。
> （v28.7.0 の `#[track]`・v28.8.0 の `#[trace]` と同じ方針）

---

## 既存ファイルとの整合

`examples/observability/sentry_alerting.fav` の v28.5.0 時点の内容:
- `import runes/sentry`（正しい構文）
- `seq SentryAlertingDemo = ReportError |> SetContext`
- 2 stage: `ReportError`（`capture_error` / `set_tag`）、`SetContext`（`set_user` / `set_extra`）

既存 `v285000_tests::sentry_example_has_pipeline` が `SentryAlertingDemo` を確認。

v28.9.0 での変更:
- `seq SentryAlertingDemo` は **`CriticalLoad |> ReportError |> SetContext` に更新**（`SentryAlertingDemo` 文字列は維持）
- `CriticalLoad` stage を先頭に追加（`// #[on_error]` コメント + `capture_message` 使用）
- 既存 `ReportError` / `SetContext` は維持

> **型シグネチャについて**: ロードマップ v28.9 では `stage CriticalLoad: Unit -> List<Order> !Db`
> と記載されているが、本実装では `Unit -> Result<Unit, String> !Io` を採用する。
> 理由: (1) Sentry アラートデモでは DB アクセスは不要（`!Db` は過剰なエフェクト）、
> (2) 他の observability E2E デモ（v28.7/v28.8）と同じシグネチャに統一し整合性を保つ、
> (3) `List<Order>` は Sentry アラートのデモスコープに不適切（外部 API 呼び出し結果を返す必要はない）。
> v29.0+ の `#[on_error]` コンパイラ実装時に正式シグネチャを確定する予定。

---

## sentry_alerting.fav の新設計

```favnir
import runes/sentry

// #[on_error(report_to: "sentry", level: "critical")] — v29.0+ でコンパイラが自動挿入予定
stage CriticalLoad: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Sentry.capture_error("critical stage failed: connection timeout")
    bind _ <- Sentry.capture_message("critical", "pipeline critical failure detected")
    Result.ok(unit)
}

stage ReportError: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Sentry.capture_error("pipeline execution failed: connection timeout")
    bind _ <- Sentry.set_tag("pipeline", "etl")
    Result.ok(unit)
}

stage SetContext: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Sentry.set_user("user-001", "ops@example.com")
    bind _ <- Sentry.set_extra("pipeline_version", "28.9.0")
    Result.ok(unit)
}

seq SentryAlertingDemo = CriticalLoad |> ReportError |> SetContext
```

---

## docker-compose.yml 更新

既存の `datadog-agent` サービスの後に Sentry を追加:

```yaml
  sentry:
    image: getsentry/sentry:24.0
    ports:
      - "9000:9000"
    environment:
      - SENTRY_SECRET_KEY=${SENTRY_SECRET_KEY:-dummy-secret-key-for-local-mode}
```

> **注記**: 本番環境の Sentry セルフホストは redis / postgres 等の追加サービスが必要。
> v28.9.0 のデモ用途では単体起動の簡略構成を採用し、`SENTRY_SECRET_KEY` は
> `${SENTRY_SECRET_KEY:-dummy-secret-key-for-local-mode}` でデモ用デフォルト値を設定。
> 本番では `SENTRY_SECRET_KEY` 環境変数で上書きすること。

---

## 追加ファイル

| ファイル | 内容 |
|---|---|
| `examples/observability/sentry_alerting.fav` | **更新**（`CriticalLoad` 追加、seq 拡充）|
| `examples/observability/docker-compose.yml` | **更新**（`sentry` サービス追加）|
| `site/content/docs/tools/sentry-alerting.mdx` | 新規作成（Sentry アラートデモ解説）|
| `benchmarks/v28.9.0.json` | 新規作成（test_count: 2305）|
| `CHANGELOG.md` | `[v28.9.0]` セクション追加 |

---

## テスト一覧（driver.rs v289000_tests）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `sentry_alerting_example_has_critical_load_stage` | `sentry_alerting.fav` に `stage CriticalLoad` を含む |
| 2 | `sentry_alerting_example_has_on_error_annotation` | `sentry_alerting.fav` に `// #[on_error(report_to:` を含む |
| 3 | `sentry_alerting_example_has_capture_message` | `sentry_alerting.fav` に `Sentry.capture_message` を含む |
| 4 | `sentry_alerting_seq_includes_critical_load` | `sentry_alerting.fav` に `CriticalLoad \|>` を含む |
| 5 | `docker_compose_has_sentry` | `docker-compose.yml` に `sentry` を含む |
| 6 | `sentry_alerting_doc_exists` | `site/content/docs/tools/sentry-alerting.mdx` に `SentryAlertingDemo` を含む |
| 7 | `sentry_alerting_example_has_import_runes_sentry` | `sentry_alerting.fav` に `import runes/sentry` を含む |
| 8 | `changelog_has_v28_9_0` | `CHANGELOG.md` に `[v28.9.0]` または `## v28.9.0` を含む |

合計 8 テスト。test_count: **2305**（2297 + 8）

---

## 完了条件チェックリスト

- [ ] `Cargo.toml` version = `28.9.0`
- [ ] `examples/observability/sentry_alerting.fav` 更新（`CriticalLoad` stage 追加、`// #[on_error(report_to:` コメント、`seq SentryAlertingDemo` 維持）
- [ ] `examples/observability/docker-compose.yml` 更新（`sentry` サービス追加）
- [ ] `site/content/docs/tools/sentry-alerting.mdx` 存在（`SentryAlertingDemo` 言及）
- [ ] `CHANGELOG.md` に `[v28.9.0]` セクションあり
- [ ] `benchmarks/v28.9.0.json` 存在（test_count: 2305）
- [ ] `cargo test --bin fav v289000` — 8/8 PASS
- [ ] `cargo test --bin fav sentry` — 既存テスト `sentry_example_has_pipeline` 含め PASS
- [ ] `cargo test --bin fav` — 2305 tests PASS
- [ ] （手動確認）`docker compose up -d` で sentry サービス起動
- [ ] （手動確認）`fav run examples/observability/sentry_alerting.fav` がエラーなく実行
