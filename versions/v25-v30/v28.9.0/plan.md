# v28.9.0 Plan — オブザーバビリティ E2E デモ（sentry アラート）

## 実装順序

```
T1: Cargo.toml version bump (28.8.0 → 28.9.0)
T2: examples/observability/sentry_alerting.fav 更新（CriticalLoad 追加、seq 拡充）
T3: examples/observability/docker-compose.yml 更新（sentry サービス追加）
T4: site/content/docs/tools/sentry-alerting.mdx 新規作成
T5: CHANGELOG.md に [v28.9.0] セクション追加
T6: benchmarks/v28.9.0.json 新規作成
T7: driver.rs に v289000_tests 8 件追加
T8: cargo test --bin fav v289000 — 8/8 PASS 確認
T8.5: cargo test --bin fav sentry — 既存 sentry_example_has_pipeline 含め PASS 確認
T9: cargo test --bin fav 全体 — 2305 PASS 確認
T10: tasks.md を COMPLETE に更新
```

---

## T2: sentry_alerting.fav の更新内容

既存ファイル（v28.5.0 stub）に `CriticalLoad` stage を**先頭に追加**し、seq を更新する。
**`seq SentryAlertingDemo` は維持**（`sentry_example_has_pipeline` テストが `SentryAlertingDemo` をチェック）。

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

**注意**:
- `import runes/sentry` は既存のまま維持（変更不要）
- `// #[on_error(report_to: "sentry", level: "critical")]` はコメント形式（`// ` 前置き）
- テスト `sentry_alerting_example_has_on_error_annotation` は `// #[on_error(report_to:` を検索

---

## T3: docker-compose.yml 更新

既存の `datadog-agent` サービスの後に追記:

```yaml
  sentry:
    image: getsentry/sentry:24.0
    ports:
      - "9000:9000"
    environment:
      - SENTRY_SECRET_KEY=${SENTRY_SECRET_KEY:-dummy-secret-key-for-local-mode}
```

`SENTRY_SECRET_KEY` はシェル `:-` 演算子でデモ用デフォルト値を設定。
本番環境では `SENTRY_SECRET_KEY` 環境変数で上書きする設計（ハードコードなし）。

---

## T4: site/content/docs/tools/sentry-alerting.mdx

- frontmatter: `title: Sentry Alerting Demo`, `description: Sentry アラート E2E デモ（v28.9.0）`
- `SentryAlertingDemo` seq のコード例掲載
- `// #[on_error(report_to: "sentry", level: "critical")]` の将来バージョン（v29.0+）での自動化について
- Docker Compose セットアップ手順（`SENTRY_SECRET_KEY` 環境変数の設定方法含む）
- `http://localhost:9000` での Sentry UI 確認手順

---

## T7: driver.rs — v289000_tests

```rust
// ── v289000_tests (v28.9.0) — sentry アラート E2E デモ ────────────────────────────
#[cfg(test)]
mod v289000_tests {
    // include_str! のみ使用のため use super::* 不要
    #[test]
    fn sentry_alerting_example_has_critical_load_stage() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("stage CriticalLoad"), "sentry_alerting.fav must define stage CriticalLoad");
    }
    #[test]
    fn sentry_alerting_example_has_on_error_annotation() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("// #[on_error(report_to:"), "sentry_alerting.fav must contain // #[on_error(report_to: annotation in comment form");
    }
    #[test]
    fn sentry_alerting_example_has_capture_message() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("Sentry.capture_message"), "sentry_alerting.fav must use Sentry.capture_message");
    }
    #[test]
    fn sentry_alerting_seq_includes_critical_load() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("CriticalLoad |>"), "sentry_alerting.fav seq must include CriticalLoad |>");
    }
    #[test]
    fn docker_compose_has_sentry() {
        let src = include_str!("../../examples/observability/docker-compose.yml");
        assert!(src.contains("getsentry/sentry"), "docker-compose.yml must define sentry service (getsentry/sentry image)");
    }
    #[test]
    fn sentry_alerting_doc_exists() {
        let src = include_str!("../../site/content/docs/tools/sentry-alerting.mdx");
        assert!(src.contains("SentryAlertingDemo"), "sentry-alerting.mdx must mention SentryAlertingDemo");
    }
    #[test]
    fn sentry_alerting_example_has_import_runes_sentry() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("import runes/sentry"), "sentry_alerting.fav must use import runes/sentry");
    }
    #[test]
    fn changelog_has_v28_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.9.0]") || src.contains("## v28.9.0"), "CHANGELOG.md must contain '[v28.9.0]'");
    }
}
```

`include_str!` パス一覧:
- `../../examples/observability/sentry_alerting.fav`
- `../../examples/observability/docker-compose.yml`
- `../../site/content/docs/tools/sentry-alerting.mdx`
- `../../CHANGELOG.md`

---

## 既存テスト保護（T8.5）

`cargo test --bin fav sentry` を実行し、v285000_tests の以下が引き続き PASS することを確認:
- `sentry_example_has_pipeline`（`SentryAlertingDemo` seq 維持により PASS）
- その他 `sentry_*` テスト群
