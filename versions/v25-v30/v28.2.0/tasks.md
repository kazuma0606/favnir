# v28.2.0 Tasks — datadog Rune 追加

Status: COMPLETE
test_count: 2244

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.1.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2235 tests` を含むこと
- [x] `driver.rs` に `mod v282000_tests` が存在しないこと

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.1.0` → `28.2.0` | [x] |
| T2 | `vm.rs` に `Datadog.*_raw` 5 primitive 追加 | [x] |
| T3 | `runes/datadog/datadog.fav` 新規作成（5 関数） | [x] |
| T4 | `examples/observability/datadog_apm.fav` 新規作成 | [x] |
| T5 | `site/content/docs/runes/datadog.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` に `[v28.2.0]` セクション追加 | [x] |
| T7 | `benchmarks/v28.2.0.json` 新規作成（test_count: 2244） | [x] |
| T8 | `driver.rs` に `v282000_tests` 9 件追加 | [x] |
| T9a | `fav/self/checker.fav` `ns_to_effect` に `"Datadog" => "IO"` 追加 | [x] |
| T9b | `cargo test --bin fav datadog` — 8 件以上 PASS 確認 | [x] |
| T9c | `cargo test --bin fav v282000` — 9/9 PASS 確認 | [x] |
| T9d | `cargo test --bin fav` 全体 — 2244 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T8）

```rust
// ── v282000_tests (v28.2.0) — datadog Rune 追加 ───────────────────────
#[cfg(test)]
mod v282000_tests {
    #[test]
    fn datadog_rune_has_metric_fn() {
        let src = include_str!("../../runes/datadog/datadog.fav");
        assert!(src.contains("fn metric("), "datadog rune must define fn metric(");
    }
    #[test]
    fn datadog_rune_has_log_fn() {
        let src = include_str!("../../runes/datadog/datadog.fav");
        assert!(src.contains("fn log("), "datadog rune must define fn log(");
    }
    #[test]
    fn datadog_rune_has_trace_fn() {
        let src = include_str!("../../runes/datadog/datadog.fav");
        assert!(src.contains("fn trace("), "datadog rune must define fn trace(");
    }
    #[test]
    fn datadog_rune_has_event_fn() {
        let src = include_str!("../../runes/datadog/datadog.fav");
        assert!(src.contains("fn event("), "datadog rune must define fn event(");
    }
    #[test]
    fn datadog_rune_has_service_check_fn() {
        let src = include_str!("../../runes/datadog/datadog.fav");
        assert!(src.contains("fn service_check("), "datadog rune must define fn service_check(");
    }
    #[test]
    fn datadog_rune_uses_io_effect() {
        let src = include_str!("../../runes/datadog/datadog.fav");
        assert!(src.contains("!Io"), "datadog rune must use !Io effect");
    }
    #[test]
    fn vm_has_datadog_metric_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("Datadog.metric_raw"), "vm.rs must implement Datadog.metric_raw");
    }
    #[test]
    fn datadog_example_has_pipeline() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("DatadogApmDemo"), "datadog_apm.fav must define DatadogApmDemo seq");
    }
    #[test]
    fn changelog_has_v28_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.2.0]") || src.contains("## v28.2.0"), "CHANGELOG.md must contain '[v28.2.0]'");
    }
}
```

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.2.0"
- [x] `runes/datadog/datadog.fav` 存在（5 関数、`!Io` エフェクト）
- [x] `Datadog.*_raw` 5 VM primitive 存在（`#[cfg]` ガード付き）
- [x] `fav/self/checker.fav` `ns_to_effect` に `ns == "Datadog"` → `"IO"` あり
- [x] `examples/observability/datadog_apm.fav` に `DatadogApmDemo` seq あり
- [x] `site/content/docs/runes/datadog.mdx` 存在
- [x] `CHANGELOG.md` に `[v28.2.0]` セクションあり
- [x] `benchmarks/v28.2.0.json` 存在（test_count: 2244）
- [x] `cargo test --bin fav datadog` — 8 件以上 PASS（`changelog_has_v28_2_0` は v282000 フィルタでのみヒット）
- [x] `site/content/docs/runes/datadog.mdx` 存在（手動確認）
- [x] `benchmarks/v28.2.0.json` 存在、test_count: 2244（手動確認）
- [x] `cargo test --bin fav v282000` — 9/9 PASS
- [x] `cargo test --bin fav` — 2244 tests PASS

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `checker.fav` `ns_to_effect` に `"Datadog" => "Io"`（小文字 o）と書いてしまうと v28.1.0 と同じバグになる | v28.1.0 の反省を活かし `"IO"`（全大文字）で最初から実装 |
| [MED] | `datadog_apm.fav` の stage 戻り型が `Unit`（実際は `Result<Unit, String>`） | `Unit -> Result<Unit, String> !Io` で実装済み |
| [MED] | `vm.rs` SSRF リスク: `Datadog.metric_raw` / `log_raw` / `trace_raw` 等でエンドポイント URL を引数に取る場合、呼び出し元検証なし | stub 段階では影響なし。v28.x 実装時に URL バリデーションを追加予定 |
| [LOW] | `#[trace]` アノテーション（AST 変更）は v28.2.0 スコープ外に延期 | spec.md に明記済み（v28.3+ 対応）|
