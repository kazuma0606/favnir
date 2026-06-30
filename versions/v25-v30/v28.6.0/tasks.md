# v28.6.0 Tasks — grafana Rune 追加

Status: COMPLETE
test_count: 2281

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.5.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2272 tests` を含むこと
- [x] `driver.rs` に `mod v286000_tests` が存在しないこと

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.5.0` → `28.6.0` | [x] |
| T2 | `vm.rs` に `Grafana.*_raw` 3 primitive 追加 | [x] |
| T3 | `runes/grafana/grafana.fav` 新規作成（3 関数） | [x] |
| T4 | `examples/observability/grafana_dashboard.fav` 新規作成 | [x] |
| T5 | `site/content/docs/runes/grafana.mdx` 新規作成 | [x] |
| T6 | `CHANGELOG.md` に `[v28.6.0]` セクション追加 | [x] |
| T7 | `benchmarks/v28.6.0.json` 新規作成（test_count: 2281） | [x] |
| T8 | `driver.rs` に `v286000_tests` 9 件追加 | [x] |
| T9a | `fav/self/checker.fav` `ns_to_effect` に `"Grafana" => "IO"` 追加 | [x] |
| T9b | `cargo test --bin fav v286000` — 9/9 PASS 確認 | [x] |
| T9c | `cargo test --bin fav grafana` — 8 件以上 PASS 確認 | [x] |
| T9d | `cargo test --bin fav` 全体 — 2281 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T8）

```rust
// ── v286000_tests (v28.6.0) — grafana Rune 追加 ────────────────────────────
#[cfg(test)]
mod v286000_tests {
    #[test]
    fn grafana_rune_has_create_annotation_fn() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("fn create_annotation("), "grafana rune must define fn create_annotation(");
    }
    #[test]
    fn grafana_rune_has_push_dashboard_fn() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("fn push_dashboard("), "grafana rune must define fn push_dashboard(");
    }
    #[test]
    fn grafana_rune_has_snapshot_fn() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("fn snapshot("), "grafana rune must define fn snapshot(");
    }
    #[test]
    fn grafana_rune_uses_io_effect() {
        let src = include_str!("../../runes/grafana/grafana.fav");
        assert!(src.contains("!Io"), "grafana rune must use !Io effect");
    }
    #[test]
    fn vm_has_grafana_create_annotation_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("Grafana.create_annotation_raw"), "vm.rs must implement Grafana.create_annotation_raw");
    }
    #[test]
    fn grafana_example_has_pipeline() {
        let src = include_str!("../../examples/observability/grafana_dashboard.fav");
        assert!(src.contains("GrafanaDashboardDemo"), "grafana_dashboard.fav must define GrafanaDashboardDemo seq");
    }
    #[test]
    fn checker_has_grafana_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"Grafana\"") && src.contains("\"IO\""),
            "checker.fav ns_to_effect must contain 'ns == \"Grafana\"' and map it to \"IO\" (note: \"IO\" alone is insufficient — ns == \"Grafana\" is the anchor)"
        );
    }
    #[test]
    fn changelog_has_v28_6_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.6.0]") || src.contains("## v28.6.0"), "CHANGELOG.md must contain '[v28.6.0]'");
    }
    #[test]
    fn grafana_doc_exists() {
        let src = include_str!("../../site/content/docs/runes/grafana.mdx");
        assert!(src.contains("Grafana"), "grafana.mdx must mention Grafana");
    }
}
```

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "28.6.0"
- [x] `runes/grafana/grafana.fav` 存在（3 関数、`!Io` エフェクト）
- [x] `Grafana.*_raw` 3 VM primitive 存在（`#[cfg]` ガード付き）
- [x] `fav/self/checker.fav` `ns_to_effect` に `ns == "Grafana"` → `"IO"` あり
- [x] `examples/observability/grafana_dashboard.fav` に `GrafanaDashboardDemo` seq あり
- [x] `site/content/docs/runes/grafana.mdx` 存在
- [x] `CHANGELOG.md` に `[v28.6.0]` セクションあり
- [x] `benchmarks/v28.6.0.json` 存在（test_count: 2281）
- [x] `cargo test --bin fav v286000` — 9/9 PASS
- [x] `cargo test --bin fav grafana` — 8 件以上 PASS（`changelog_has_v28_6_0` は `grafana` フィルタではマッチしない）
- [x] `cargo test --bin fav` — 2281 tests PASS

## コードレビュー指摘対応

### [HIGH] 指摘
なし

### [MED] 指摘（対応不要 — 既存パターン踏襲）
- `include_str!("../../fav/self/checker.fav")` と `../self/checker.fav` の2系統が driver.rs 内に混在している。v285000_tests 等の先行テスト群が同パスで PASS 済みのため今バージョンは修正不要。将来の統一はリファクタリング課題として記録。

### [LOW] 指摘（次バージョン以降の対応候補）
- `snapshot_raw` 内で `_dashboard_id` が無視されることの説明コメントが不足。v28.7 で実際の HTTP 実装を行う際に引数利用漏れを防ぐため、「`_dashboard_id` は v28.7+ で URL 生成に使用予定」旨のコメント追加を v28.7 実装時に対応。
