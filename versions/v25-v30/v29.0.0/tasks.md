# v29.0.0 Tasks — Observability First マイルストーン宣言

**状態**: COMPLETE
**開始日**: 2026-06-28
**完了日**: 2026-06-28

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.9.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2306 passed` を含むこと
- [x] `driver.rs` に `mod v290000_tests` が存在しないこと
- [x] 前提 Rune が存在すること:
  - `runes/prometheus/` ディレクトリ（v28.1 で作成）
  - `runes/sentry/` ディレクトリ（v28.5 で作成）
  - `runes/grafana/` ディレクトリ（v28.6 で作成）
  - `examples/observability/prometheus_grafana.fav`（v28.7 で作成）
  - `examples/observability/datadog_apm.fav`（v28.8 で作成）
  - `examples/observability/sentry_alerting.fav`（v28.9 で作成）

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.9.0` → `29.0.0` | [x] |
| T2 | MILESTONE.md に "Observability First" セクション追加（先頭） | [x] |
| T3 | README.md に v29.0 参照追記 | [x] |
| T4 | `site/content/docs/observability-first.mdx` 新規作成 | [x] |
| T5 | `versions/roadmap/roadmap-v28.1-v29.0.md` 完了マーク追記 | [x] |
| T6 | CHANGELOG.md に `[v29.0.0]` セクション追加 | [x] |
| T7 | `benchmarks/v29.0.0.json` 新規作成（test_count: 2312） | [x] |
| T8 | `driver.rs` に `v290000_tests` 6 件追加 | [x] |
| T8.5 | `cargo test --bin fav v290000` — 6/6 PASS 確認 | [x] |
| T9 | `cargo test --bin fav` 全体 — 2312 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T8）

```rust
// ── v290000_tests (v29.0.0) — Observability First マイルストーン宣言 ────────────────────────────
#[cfg(test)]
mod v290000_tests {
    // include_str! のみ使用のため use super::* 不要
    #[test]
    fn milestone_md_mentions_observability_first() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Observability First"), "MILESTONE.md must mention Observability First");
    }
    #[test]
    fn milestone_md_lists_prometheus_rune() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("prometheus") || src.contains("Prometheus"), "MILESTONE.md must list prometheus rune");
    }
    #[test]
    fn milestone_md_lists_sentry_rune() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("sentry") || src.contains("Sentry"), "MILESTONE.md must list sentry rune");
    }
    #[test]
    fn readme_mentions_v29() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v29.0") || src.contains("v29.0.0"), "README.md must mention v29.0");
    }
    #[test]
    fn site_observability_first_page_exists() {
        let src = include_str!("../../site/content/docs/observability-first.mdx");
        assert!(
            src.contains("Observability First") && (src.contains("prometheus") || src.contains("Prometheus")),
            "observability-first.mdx must contain 'Observability First' and 'prometheus'"
        );
    }
    #[test]
    fn changelog_has_v29_0_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v29.0.0]") || src.contains("## v29.0.0"), "CHANGELOG.md must contain '[v29.0.0]'");
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.0.0"
- [x] MILESTONE.md に "Observability First" セクションあり
- [x] README.md に `v29.0` または `v29.0.0` の記述あり
- [x] `site/content/docs/observability-first.mdx` 存在（"Observability First" + "prometheus" を含む）
- [x] `CHANGELOG.md` に `[v29.0.0]` セクションあり
- [x] `benchmarks/v29.0.0.json` 存在（test_count: 2312）
- [x] `cargo test --bin fav v290000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2312 tests PASS
- [x] `docker compose -f examples/observability/docker-compose.yml up -d` — 全サービス Up（6 サービス確認）
- [x] `fav run examples/observability/prometheus_grafana.fav` — exit 0（`fn main()` 追加 + `Result.ok(())` 修正）
- [x] `fav run examples/observability/datadog_apm.fav` — exit 0（`fn main()` 追加 + `Result.ok(())` 修正）
- [x] `fav run examples/observability/sentry_alerting.fav` — exit 0
- [x] `fav profile --format flamegraph fav/tests/fixtures/etl.fav` — `flamegraph.svg`（46 bytes）出力確認

---

## コードレビュー指摘対応

| 優先度 | 指摘内容 | 対応 |
|---|---|---|
| [MED] | `fav profile --format flamegraph` 確認が tasks.md から欠落 | tasks.md・spec.md に追記（`fav/tests/fixtures/etl.fav` 使用） |
| [MED] | spec.md の完了条件と tasks.md のチェックリスト乖離 | spec.md に Docker + `fav run` E2E 3 件を追記 |
| [LOW] | T0 grep 文字列が `"2306 tests"` で実際の出力と不一致 | `"2306 passed"` に修正 |
| [LOW] | plan.md の行番号固定参照（line 37874） | 行番号なし表現に修正 |

### 実装時に発見・修正した問題

| 問題 | 対応 |
|---|---|
| `prometheus_grafana.fav` / `datadog_apm.fav` に `fn main()` が欠落 | `fn main() -> Result<Unit, String> !Io` を追加 |
| 両ファイルで `Result.ok(unit)` が `RuntimeError: unknown global or builtin: unit` | `Result.ok(())` に修正（sentry_alerting.fav の既存パターンに合わせ） |
| `fav/tests/fixtures/etl.fav` の stage が `unit` を返すためプロファイル時エラー | `{ unit }` → `{ () }` に修正、`fn main()` を追加 |
