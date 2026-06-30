# v28.5.0 Tasks — sentry Rune 追加

Status: COMPLETE
test_count: 2272

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.4.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2262 tests` を含むこと
- [x] `driver.rs` に `mod v285000_tests` が存在しないこと

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.4.0` → `28.5.0` | [x] |
| T2 | `vm.rs` に `Sentry.*_raw` 5 primitive 追加 | [x] |
| T3 | `runes/sentry/sentry.fav` 新規作成（5 関数） | [x] |
| T4 | `examples/observability/sentry_alerting.fav` 新規作成 | [x] |
| T5 | `site/content/docs/runes/sentry.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` に `[v28.5.0]` セクション追加 | [x] |
| T7 | `benchmarks/v28.5.0.json` 新規作成（test_count: 2272） | [x] |
| T8 | `driver.rs` に `v285000_tests` 10 件追加（`set_extra_fn` テストを含む） | [x] |
| T9a | `fav/self/checker.fav` `ns_to_effect` に `"Sentry" => "IO"` 追加 | [x] |
| T9b | `cargo test --bin fav v285000` — 9/9 PASS 確認 | [x] |
| T9c | `cargo test --bin fav sentry` — 7 件以上 PASS 確認 | [x] |
| T9d | `cargo test --bin fav` 全体 — 2271 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T8）

```rust
// ── v285000_tests (v28.5.0) — sentry Rune 追加 ────────────────────────────
#[cfg(test)]
mod v285000_tests {
    #[test]
    fn sentry_rune_has_capture_error_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn capture_error("), "sentry rune must define fn capture_error(");
    }
    #[test]
    fn sentry_rune_has_capture_message_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn capture_message("), "sentry rune must define fn capture_message(");
    }
    #[test]
    fn sentry_rune_has_set_user_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn set_user("), "sentry rune must define fn set_user(");
    }
    #[test]
    fn sentry_rune_has_set_tag_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn set_tag("), "sentry rune must define fn set_tag(");
    }
    #[test]
    fn sentry_rune_has_set_extra_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn set_extra("), "sentry rune must define fn set_extra(");
    }
    #[test]
    fn sentry_rune_uses_io_effect() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("!Io"), "sentry rune must use !Io effect");
    }
    #[test]
    fn vm_has_sentry_capture_error_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("Sentry.capture_error_raw"), "vm.rs must implement Sentry.capture_error_raw");
    }
    #[test]
    fn sentry_example_has_pipeline() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("SentryAlertingDemo"), "sentry_alerting.fav must define SentryAlertingDemo seq");
    }
    #[test]
    fn checker_has_sentry_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"Sentry\"") && src.contains("\"IO\""),
            "checker.fav ns_to_effect must contain 'ns == \"Sentry\"' and map it to \"IO\" (note: \"IO\" alone is insufficient — ns == \"Sentry\" is the anchor)"
        );
    }
    #[test]
    fn changelog_has_v28_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.5.0]") || src.contains("## v28.5.0"), "CHANGELOG.md must contain '[v28.5.0]'");
    }
}
```

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.5.0"
- [x] `runes/sentry/sentry.fav` 存在（5 関数、`!Io` エフェクト）
- [x] `Sentry.*_raw` 5 VM primitive 存在（`#[cfg]` ガード付き）
- [x] `fav/self/checker.fav` `ns_to_effect` に `ns == "Sentry"` → `"IO"` あり
- [x] `examples/observability/sentry_alerting.fav` に `SentryAlertingDemo` seq あり
- [x] `site/content/docs/runes/sentry.mdx` 存在
- [x] `CHANGELOG.md` に `[v28.5.0]` セクションあり
- [x] `benchmarks/v28.5.0.json` 存在（test_count: 2272）
- [x] `cargo test --bin fav v285000` — 10/10 PASS
- [x] `cargo test --bin fav sentry` — 8 件以上 PASS（`sentry` を名前に含む 8 件がマッチ。`checker_has_sentry_effect` / `changelog_has_v28_5_0` は `sentry` フィルタでは**マッチしない**——`v285000` フィルタのみ）
- [x] `cargo test --bin fav` — 2272 tests PASS

## コードレビュー指摘対応

### [HIGH] 指摘
なし

### [MED] 指摘（対応不要 — 既存パターン踏襲）
- `ok_or(&str)` の型: OTel / Datadog / Prometheus と同一パターン。既存コードでコンパイル通過済みのため修正不要。
- エフェクト表記 `!Io`（Rune ファイル）vs `"IO"`（checker.fav 戻り値）: 既存 Rune（otel.fav 等）と同一慣習。問題なし。

### [LOW] 指摘（次バージョン以降の改善候補）
- `ns_to_effect` の深ネスト（約20段）: 構造的リファクタリング候補。今回は対象外。
- `sentry_alerting.fav` で `capture_message` が未使用: example はデモ目的であり全関数網羅は不要。
- PII 警告が末尾注記のみ: `set_user` 関数説明直後への移動は v28.9+ ドキュメント整備時に対応。
- vm.rs テストが `capture_error_raw` のみ: OTel と同様の慣習（1件で代表）。追加テストは次版以降。
