# v28.3.0 Tasks — OpenTelemetry Rune 強化

Status: COMPLETE
test_count: 2253

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.2.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2244 tests` を含むこと
- [x] `driver.rs` に `mod v283000_tests` が存在しないこと

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.2.0` → `28.3.0` | [x] |
| T2 | `vm.rs` に `OTel.*_raw` 4 primitive 追加 | [x] |
| T3 | `runes/otel/otel.fav` 新規作成（4 関数） | [x] |
| T4 | `examples/observability/otel_tracing.fav` 新規作成 | [x] |
| T5 | `site/content/docs/runes/otel.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` に `[v28.3.0]` セクション追加 | [x] |
| T7 | `benchmarks/v28.3.0.json` 新規作成（test_count: 2253） | [x] |
| T8 | `driver.rs` に `v283000_tests` 9 件追加 | [x] |
| T9a | `fav/self/checker.fav` `ns_to_effect` に `"OTel" => "IO"` 追加 | [x] |
| T9b | `cargo test --bin fav v283000` — 9/9 PASS 確認 | [x] |
| T9c | `cargo test --bin fav otel` — 7 件以上 PASS 確認 | [x] |
| T9d | `cargo test --bin fav` 全体 — 2253 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T8）

```rust
// ── v283000_tests (v28.3.0) — OpenTelemetry Rune 強化 ──────────────────────
#[cfg(test)]
mod v283000_tests {
    #[test]
    fn otel_rune_has_start_span_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn start_span("), "otel rune must define fn start_span(");
    }
    #[test]
    fn otel_rune_has_set_attribute_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn set_attribute("), "otel rune must define fn set_attribute(");
    }
    #[test]
    fn otel_rune_has_add_event_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn add_event("), "otel rune must define fn add_event(");
    }
    #[test]
    fn otel_rune_has_end_span_fn() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("fn end_span("), "otel rune must define fn end_span(");
    }
    #[test]
    fn otel_rune_uses_io_effect() {
        let src = include_str!("../../runes/otel/otel.fav");
        assert!(src.contains("!Io"), "otel rune must use !Io effect");
    }
    #[test]
    fn vm_has_otel_start_span_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("OTel.start_span_raw"), "vm.rs must implement OTel.start_span_raw");
    }
    #[test]
    fn otel_example_has_pipeline() {
        let src = include_str!("../../examples/observability/otel_tracing.fav");
        assert!(src.contains("OTelTracingDemo"), "otel_tracing.fav must define OTelTracingDemo seq");
    }
    #[test]
    fn checker_has_otel_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"OTel\"") && src.contains("\"IO\""),
            "checker.fav ns_to_effect must contain 'ns == \"OTel\"' mapped to \"IO\""
        );
    }
    #[test]
    fn changelog_has_v28_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.3.0]") || src.contains("## v28.3.0"), "CHANGELOG.md must contain '[v28.3.0]'");
    }
}
```

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.3.0"
- [x] `runes/otel/otel.fav` 存在（4 関数、`!Io` エフェクト）
- [x] `OTel.*_raw` 4 VM primitive 存在（`#[cfg]` ガード付き）
- [x] `fav/self/checker.fav` `ns_to_effect` に `ns == "OTel"` → `"IO"` あり
- [x] `examples/observability/otel_tracing.fav` に `OTelTracingDemo` seq あり
- [x] `site/content/docs/runes/otel.mdx` 存在
- [x] `CHANGELOG.md` に `[v28.3.0]` セクションあり
- [x] `benchmarks/v28.3.0.json` 存在（test_count: 2253）
- [x] `cargo test --bin fav v283000` — 9/9 PASS
- [x] `cargo test --bin fav otel` — 7 件以上 PASS（`v283000_tests` のうち `otel` 含む 7 件がマッチ。`checker_has_otel_effect` / `changelog_has_v28_3_0` は `v283000` フィルタでのみヒット）
- [x] `cargo test --bin fav` — 2253 tests PASS

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [LOW] | `vm.rs` OTel primitives ブロックにスタブコメント欠落（Prometheus/Datadog は `// Stub: ...` コメントあり） | `// Stub: OTLP HTTP エクスポートは fav/src/otel.rs 経由（v28.x 以降）` を追加 |
| [MED] | `checker_has_prometheus_effect`（v28.1.0）が `assert!` 2 分割のまま残存しており v28.3.0 の `checker_has_otel_effect`（`&&` 単一 assert）と非対称 | 既存コードの問題のため今回は対象外。v28.x 以降での統一を推奨 |
