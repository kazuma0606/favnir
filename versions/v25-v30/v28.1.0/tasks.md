# v28.1.0 Tasks — prometheus Rune 追加

Status: COMPLETE
test_count: 2235

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.0.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2226 tests` を含むこと
- [x] `driver.rs` に `mod v281000_tests` が存在しないこと

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.0.0` → `28.1.0` | [x] |
| T2 | `vm.rs` に `Prometheus.*_raw` 4 primitive 追加 | [x] |
| T3 | `runes/prometheus/prometheus.fav` 新規作成（4 関数） | [x] |
| T4 | `examples/observability/prometheus_demo.fav` 新規作成 | [x] |
| T5 | `site/content/docs/runes/prometheus.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` に `[v28.1.0]` セクション追加 | [x] |
| T7 | `benchmarks/v28.1.0.json` 新規作成（test_count: 2235） | [x] |
| T8 | `driver.rs` に `v281000_tests` 9 件追加 | [x] |
| T9a | `fav/self/checker.fav` `ns_to_effect` に `"Prometheus" => "Io"` 追加 | [x] |
| T9b | `cargo test --bin fav v281000` — 9/9 PASS 確認 | [x] |
| T9c | `cargo test --bin fav` 全体 — 2235 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T8）

```rust
// ── v281000_tests (v28.1.0) — prometheus Rune 追加 ────────────────────
#[cfg(test)]
mod v281000_tests {
    #[test]
    fn prometheus_rune_has_counter_fn() {
        let src = include_str!("../../runes/prometheus/prometheus.fav");
        assert!(src.contains("fn counter("), "prometheus rune must define fn counter(");
    }
    #[test]
    fn prometheus_rune_has_gauge_fn() {
        let src = include_str!("../../runes/prometheus/prometheus.fav");
        assert!(src.contains("fn gauge("), "prometheus rune must define fn gauge(");
    }
    #[test]
    fn prometheus_rune_has_histogram_fn() {
        let src = include_str!("../../runes/prometheus/prometheus.fav");
        assert!(src.contains("fn histogram("), "prometheus rune must define fn histogram(");
    }
    #[test]
    fn prometheus_rune_has_push_fn() {
        let src = include_str!("../../runes/prometheus/prometheus.fav");
        assert!(src.contains("fn push("), "prometheus rune must define fn push(");
    }
    #[test]
    fn prometheus_rune_uses_io_effect() {
        let src = include_str!("../../runes/prometheus/prometheus.fav");
        assert!(src.contains("!Io"), "prometheus rune must use !Io effect");
    }
    #[test]
    fn vm_has_prometheus_counter_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("Prometheus.counter_raw"), "vm.rs must implement Prometheus.counter_raw");
    }
    #[test]
    fn prometheus_example_has_pipeline() {
        let src = include_str!("../../examples/observability/prometheus_demo.fav");
        assert!(src.contains("PrometheusDemo"), "prometheus_demo.fav must define PrometheusDemo seq");
    }
    #[test]
    fn checker_has_prometheus_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"Prometheus\""),
            "checker.fav ns_to_effect must contain 'ns == \"Prometheus\"' condition"
        );
    }
    #[test]
    fn changelog_has_v28_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.1.0]") || src.contains("## v28.1.0"), "CHANGELOG.md must contain '[v28.1.0]'");
    }
}
```

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.1.0"
- [x] `runes/prometheus/prometheus.fav` 存在（4 関数、`!Io` エフェクト）
- [x] `Prometheus.counter_raw` / `gauge_raw` / `histogram_raw` / `push_raw` VM primitive 存在（`#[cfg]` ガード付き）
- [x] `fav/self/checker.fav` `ns_to_effect` に `ns == "Prometheus"` 条件あり
- [x] `examples/observability/prometheus_demo.fav` に `PrometheusDemo` seq あり
- [x] `site/content/docs/runes/prometheus.mdx` 存在
- [x] `CHANGELOG.md` に `[v28.1.0]` セクションあり
- [x] `benchmarks/v28.1.0.json` 存在（test_count: 2235）
- [x] `cargo test --bin fav prometheus` — 7 件以上 PASS
- [x] `cargo test --bin fav v281000` — 9/9 PASS
- [x] `cargo test --bin fav` — 2235 tests PASS

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `checker.fav` L1163: `"Io"` → 既存 JSONL パターンは `"IO"`（全大文字）で不一致 | `"Io"` → `"IO"` に修正 |
| [MED] | `prometheus_demo.fav` の stage 戻り型が `Unit`（実際は `Result<Unit, String>`） | `Unit -> Result<Unit, String> !Io` に修正 |
| [MED] | `checker_has_prometheus_effect` テストが `"IO"` マッピングを検証していない | アサーションに `"IO"` 確認を追加 |
| [LOW] | vm.rs: v28.x 実装時に `gateway_url` の SSRF リスクへのコメント追加推奨 | コメント追記（stub 段階では影響なし） |
| [LOW] | `include_str!` パスが他テストと書き方不統一（機能には影響なし） | 許容（v27.9.0 以降の実証済みパターン） |
| [LOW] | `push` の蓄積メトリクス送信仕様コメント不足 | v28.x 実装時対応 |
