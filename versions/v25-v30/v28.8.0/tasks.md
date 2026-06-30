# v28.8.0 Tasks — オブザーバビリティ E2E デモ（datadog APM）

Status: COMPLETE
test_count: 2297

## 事前確認（T0）

- [x] `Cargo.toml` の version が `28.7.0` であること
- [x] `cargo test --bin fav 2>&1 | tail -1` が `2289 tests` を含むこと
- [x] `driver.rs` に `mod v288000_tests` が存在しないこと
- [x] `examples/observability/datadog_apm.fav` に `seq DatadogApmDemo` が含まれること（既存確認）
- [x] `examples/observability/docker-compose.yml` に `datadog-agent` が含まれないこと（上書き防止）
- [x] `examples/observability/docker-compose.yml` に `prometheus` サービスが含まれること（v28.7.0 完了確認）

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `28.7.0` → `28.8.0` | [x] |
| T2 | `examples/observability/datadog_apm.fav` 更新（3 stage + `// #[trace(service:]`、`import runes/datadog` 修正、`seq DatadogApmDemo` 維持） | [x] |
| T3 | `examples/observability/docker-compose.yml` 更新（`datadog-agent` 追加） | [x] |
| T4 | `site/content/docs/tools/datadog-apm.mdx` 新規作成 | [x] |
| T5 | `CHANGELOG.md` に `[v28.8.0]` セクション追加 | [x] |
| T6 | `benchmarks/v28.8.0.json` 新規作成（test_count: 2297） | [x] |
| T7 | `driver.rs` に `v288000_tests` 8 件追加 | [x] |
| T8 | `cargo test --bin fav v288000` — 8/8 PASS 確認 | [x] |
| T8.5 | `cargo test --bin fav datadog` — 既存 `datadog_example_has_pipeline` 含め PASS 確認 | [x] |
| T9 | `cargo test --bin fav` 全体 — 2297 tests PASS 確認 | [x] |
| T10 | tasks.md を COMPLETE に更新 | [x] |

## テスト詳細（T7）

```rust
// ── v288000_tests (v28.8.0) — datadog APM E2E デモ ────────────────────────────
#[cfg(test)]
mod v288000_tests {
    // include_str! のみ使用のため use super::* 不要
    #[test]
    fn datadog_apm_example_has_extract_events_stage() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("stage ExtractEvents"), "datadog_apm.fav must define stage ExtractEvents");
    }
    #[test]
    fn datadog_apm_example_has_transform_events_stage() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("stage TransformEvents"), "datadog_apm.fav must define stage TransformEvents");
    }
    #[test]
    fn datadog_apm_example_has_load_events_stage() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("stage LoadEvents"), "datadog_apm.fav must define stage LoadEvents");
    }
    #[test]
    fn datadog_apm_example_has_trace_annotation() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("// #[trace(service:"), "datadog_apm.fav must contain // #[trace(service: annotation in comment form");
    }
    #[test]
    fn datadog_apm_example_uses_datadog_metric() {
        let src = include_str!("../../examples/observability/datadog_apm.fav");
        assert!(src.contains("Datadog.metric"), "datadog_apm.fav must use Datadog.metric");
    }
    #[test]
    fn docker_compose_has_datadog_agent() {
        let src = include_str!("../../examples/observability/docker-compose.yml");
        assert!(src.contains("datadog-agent"), "docker-compose.yml must define datadog-agent service");
    }
    #[test]
    fn datadog_apm_doc_exists() {
        let src = include_str!("../../site/content/docs/tools/datadog-apm.mdx");
        assert!(src.contains("DatadogApmDemo"), "datadog-apm.mdx must mention DatadogApmDemo");
    }
    #[test]
    fn changelog_has_v28_8_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.8.0]") || src.contains("## v28.8.0"), "CHANGELOG.md must contain '[v28.8.0]'");
    }
}
```

## 完了条件チェックリスト

- [ ] `Cargo.toml` version = "28.8.0"
- [x] `examples/observability/datadog_apm.fav` 更新（3 stage、`// #[trace(service:` コメント、`import runes/datadog`、`seq DatadogApmDemo` 維持）
- [x] `examples/observability/docker-compose.yml` 更新（`datadog-agent` サービス追加）
- [x] `site/content/docs/tools/datadog-apm.mdx` 存在（`DatadogApmDemo` 言及）
- [x] `CHANGELOG.md` に `[v28.8.0]` セクションあり
- [x] `benchmarks/v28.8.0.json` 存在（test_count: 2297）
- [x] `cargo test --bin fav v288000` — 8/8 PASS
- [x] `cargo test --bin fav datadog` — 既存テスト `datadog_example_has_pipeline` 含め PASS
- [x] `cargo test --bin fav` — 2297 tests PASS
- [ ] （手動確認）`DD_API_KEY=xxx docker compose -f examples/observability/docker-compose.yml up -d` で datadog-agent 起動
- [ ] （手動確認）`fav run examples/observability/datadog_apm.fav` がエラーなく実行できる

## コードレビュー指摘対応

### [HIGH] 指摘
なし

### [MED] 指摘（対応済み / 記録のみ）
- `DD_API_KEY` デフォルト値（`dummy-key-for-local-mode`）の動作説明が不足 → `datadog-apm.mdx` のクイックスタートに「未設定時はダミーキーで起動、メトリクスは送信されません」の一文を追記して解消
- `v288000_tests` と `v282000_tests` が `datadog_apm.fav` を共有テストする際の役割分担（スタイル上の推奨） → 対応不要（意味的に整合しており、バグではない）

### [LOW] 指摘（対応不要）
- `GF_SECURITY_ADMIN_PASSWORD=admin`（v28.7.0 からの既存コード・変更外）
- `datadog-apm.mdx` の `open` コマンドが macOS 専用（デモドキュメントの慣例として許容）
