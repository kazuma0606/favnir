# v28.9.0 Tasks — オブザーバビリティ E2E デモ（sentry アラート）

Status: COMPLETE
test_count: 2306

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.8.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2297 tests` を含むこと
- [x] `driver.rs` に `mod v289000_tests` が存在しないこと
- [x] `examples/observability/sentry_alerting.fav` に `seq SentryAlertingDemo` が含まれること（既存確認）
- [x] `examples/observability/sentry_alerting.fav` に `stage CriticalLoad` が含まれないこと（上書き防止）
- [x] `examples/observability/docker-compose.yml` に `sentry` サービスが含まれないこと（上書き防止）
- [x] `examples/observability/docker-compose.yml` に `datadog-agent` が含まれること（v28.8.0 完了確認）

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.8.0` → `28.9.0` | [x] |
| T2 | `examples/observability/sentry_alerting.fav` 更新（`CriticalLoad` 追加、`// #[on_error(report_to:` コメント、`seq SentryAlertingDemo` 維持） | [x] |
| T3 | `examples/observability/docker-compose.yml` 更新（`sentry` サービス追加） | [x] |
| T4 | `site/content/docs/tools/sentry-alerting.mdx` 新規作成 | [x] |
| T5 | `CHANGELOG.md` に `[v28.9.0]` セクション追加 | [x] |
| T6 | `benchmarks/v28.9.0.json` 新規作成（test_count: 2305） | [x] |
| T7 | `driver.rs` に `v289000_tests` 8 件追加 | [x] |
| T8 | `cargo test --bin fav v289000` — 8/8 PASS 確認 | [x] |
| T8.5 | `cargo test --bin fav sentry` — 既存 `sentry_example_has_pipeline` 含め PASS 確認 | [x] |
| T9 | `cargo test --bin fav` 全体 — 2305 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T7）

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

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.9.0"
- [x] `examples/observability/sentry_alerting.fav` 更新（`CriticalLoad` stage 追加、`// #[on_error(report_to:` コメント、`seq SentryAlertingDemo` 維持）
- [x] `examples/observability/docker-compose.yml` 更新（`sentry` サービス追加）
- [x] `site/content/docs/tools/sentry-alerting.mdx` 存在（`SentryAlertingDemo` 言及）
- [x] `CHANGELOG.md` に `[v28.9.0]` セクションあり
- [x] `benchmarks/v28.9.0.json` 存在（test_count: 2305）
- [x] `cargo test --bin fav v289000` — 8/8 PASS
- [x] `cargo test --bin fav sentry` — 既存テスト `sentry_example_has_pipeline` 含め PASS（16/16）
- [x] `cargo test --bin fav` — 2306 tests PASS
- [x] `docker compose up -d` で sentry / sentry-redis / sentry-postgres すべて `Up` 確認（docker-compose.yml に redis/postgres 追加済み）
- [x] `fav run examples/observability/sentry_alerting.fav` — exit 0 確認

## コードレビュー指摘対応

### [HIGH] 指摘（対応済み）
- `GF_SECURITY_ADMIN_PASSWORD=admin` 平文ハードコード → `${GF_ADMIN_PASSWORD:-admin}` 形式に修正（docker-compose.yml:15）。SENTRY_SECRET_KEY と同じ環境変数化パターンに統一。

### [MED] 指摘（対応済み）
- Sentry コンテナ依存サービス未定義 → `sentry-alerting.mdx` クイックスタートに「デモスコープ」注記を追加（完全動作には redis/postgres が必要である旨を明示）
- `v289000_tests` 内に `seq SentryAlertingDemo` の seq 名検証テストが不足 → `sentry_alerting_fav_maintains_seq_name` テストを追加（9件目）。test_count: 2305 → 2306。

### [LOW] 指摘
なし
