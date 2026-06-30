# v29.0.0 Spec — Observability First マイルストーン宣言

## 概要

v28.1〜v28.9 で実装した Observability スタックの完成を宣言するマイルストーンバージョン。
「prometheus / datadog / sentry / grafana / otel Rune + `#[track]` / `#[trace]` / `#[on_error]` アノテーション + E2E デモ 3 本」
の全コンポーネントが揃ったことを MILESTONE.md・README・サイトドキュメントで正式記録する。

## 背景

| マイルストーン | バージョン | 内容 |
|---|---|---|
| Practical Self-Hosting | v25.0.0 | compiler.fav が自分自身をコンパイル |
| Rune Foundation | v26.0.0 | 主要データソース Rune 完備 |
| Streaming Native | v27.0.0 | リアルタイムパイプライン型安全対応 |
| Data Lakehouse | v28.0.0 | 現代データ基盤への完全統合 |
| **Observability First** | **v29.0.0** | パイプラインの内側が見える |

## 完了コンポーネント（v28.1〜v28.9）

| Rune / 機能 | バージョン | 状態 |
|---|---|---|
| prometheus Rune | v28.1 | COMPLETE |
| datadog Rune | v28.2 | COMPLETE |
| OpenTelemetry Rune（otel 強化） | v28.3 | COMPLETE |
| `fav profile --format flamegraph` / `--compare` | v28.4 | COMPLETE |
| sentry Rune | v28.5 | COMPLETE |
| grafana Rune | v28.6 | COMPLETE |
| E2E デモ（prometheus + grafana） | v28.7 | COMPLETE |
| E2E デモ（datadog APM） | v28.8 | COMPLETE |
| E2E デモ（sentry アラート） | v28.9 | COMPLETE |

## 実装内容

### T1 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version` を `"28.9.0"` → `"29.0.0"` に更新。

### T2 — MILESTONE.md 更新
`MILESTONE.md` に "Observability First" セクションを追加。
- 完了した Rune 一覧（5 Rune + 3 アノテーション + 3 デモ）
- 象徴的なデモコード（`#[track]` stage → Grafana に自動反映）
- v29.x 残件（実メトリクス送信の統合テスト等）

### T3 — README.md 更新
`README.md` に v29.0 "Observability First" 参照を追記。

### T4 — サイトドキュメント
`site/content/docs/observability-first.mdx` 新規作成。

### T5 — roadmap 完了マーク
`versions/roadmap/roadmap-v28.1-v29.0.md` に完了マークを追記。

### T6 — CHANGELOG
`CHANGELOG.md` に `[v29.0.0]` セクション追加。

### T7 — ベンチマーク
`benchmarks/v29.0.0.json` 新規作成（test_count: 2312）。

### T8 — driver.rs テスト（6 件）

```
v290000_tests:
  milestone_md_mentions_observability_first
  milestone_md_lists_prometheus_rune
  milestone_md_lists_sentry_rune
  readme_mentions_v29
  site_observability_first_page_exists
  changelog_has_v29_0_0
```

> prometheus（メトリクス基盤の中核）と sentry（エラートラッキング）をマイルストーンの
> 象徴として個別確認。他 3 Rune（datadog / otel / grafana）は v28.2〜v28.6 の
> 各テストモジュールで検証済みのため重複しない。

## テスト数

- v28.9.0: 2306 tests
- v29.0.0: **2312 tests**（+6）

## 完了条件

- [ ] `Cargo.toml` version = "29.0.0"
- [ ] MILESTONE.md に "Observability First" セクションあり
- [ ] README.md に `v29.0` または `v29.0.0` の記述あり
- [ ] `site/content/docs/observability-first.mdx` 存在（"Observability First" + "prometheus" を含む）
- [ ] `CHANGELOG.md` に `[v29.0.0]` セクションあり
- [ ] `benchmarks/v29.0.0.json` 存在（test_count: 2312）
- [ ] `cargo test --bin fav v290000` が 6/6 PASS
- [ ] `cargo test --bin fav` 全体が 2312 tests PASS
- [ ] `docker compose -f examples/observability/docker-compose.yml up -d` — 全サービス Up
- [ ] `fav run examples/observability/prometheus_grafana.fav` — exit 0
- [ ] `fav run examples/observability/datadog_apm.fav` — exit 0
- [ ] `fav run examples/observability/sentry_alerting.fav` — exit 0
- [ ] `fav profile --format flamegraph fav/tests/fixtures/etl.fav` — SVG 出力確認
