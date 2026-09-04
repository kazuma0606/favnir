# Spec: v98.9.0 — 安定化・コードフリーズ

## Background

v98.1.0〜v98.8.0 で SAP Analytics の全機能（型定義 / BW クエリ / SAC プッシュ / KPI アラート / CLI / E2E デモ / サイトドキュメント）を実装した。
v98.9.0 では、これらをまとめて安定化確認（コードフリーズ）を行う。
2 件のスモークテストを追加し、SAP Analytics 全体の整合性を確認する。

## Goals

1. `fav/src/driver.rs` — `mod v98900_tests`（2 テスト）追加
   - `sap_odata_rune_exports_kpi_alert`: `sap_odata.fav` が `KpiAlert` を re-export していることを確認
   - `analytics_demo_run_script_exists`: `analytics_demo/run.sh` が存在することを確認

## Syntax / API Examples

### mod v98900_tests（Rust テスト）

```rust
#[cfg(test)]
mod v98900_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sap_odata_rune_exports_kpi_alert() {
        let content = std::fs::read_to_string(
            "../runes/sap-odata/sap_odata.fav",
        )
        .expect("sap_odata.fav should exist");
        assert!(
            content.contains("KpiAlert"),
            "sap_odata.fav should re-export KpiAlert (v98.9.0 freeze check)"
        );
    }

    #[test]
    fn analytics_demo_run_script_exists() {
        std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/analytics_demo/run.sh",
        )
        .expect("analytics_demo/run.sh should exist (v98.9.0 freeze check)");
    }
}
```

## Success Criteria

- `fav/src/driver.rs` に `mod v98900_tests` が存在する
- `cargo test -- --test-threads=1` が 4,253 tests, 0 failures で通過する
- `cargo clippy --locked -- -D warnings` が通過する
- `./target/debug/fav fmt --check self/compiler.fav` が通過する
- `./target/debug/fav fmt --check self/checker.fav` が通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | 追記（`mod v98900_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |
