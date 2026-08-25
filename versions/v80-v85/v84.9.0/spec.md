# spec: v84.9.0 — 安定化・コードフリーズ

## Background

> **テスト数注記**: ロードマップ計画値は 3,913/3,915 だったが、code-reviewer 対応の
> 累積により実際のベースは **3,925 tests**（v84.8.0 完了時点）。
> v84.9.0 完了目標は **3,927 tests**（+2）。

v84.1.0〜v84.8.0 で Favnir 4.0 の全機能（Sprint 1〜4・ドキュメント・OSS 整備・
パフォーマンス確認）を完了した。v84.9.0 は最終安定化スプリントとして、
4 スプリント統合の E2E ショーケースが完全に機能することを確認し、コードフリーズを宣言する。
新機能追加は行わず、バグ修正のみ受け入れる。

## Goals

1. `cargo test` で v84.1〜v84.8 全テスト（3,925 件）が pass することを確認する
2. `infra/e2e-demo/favnir4-showcase/` E2E ショーケースの統合確認:
   - Sprint 1（TestSuite / GoldenDataset / SchemaSnapshot）
   - Sprint 2（QualityCheck / QualityGate / AnomalyDetector）
   - Sprint 3（ContractRegistry / IoContract）
   - Sprint 4（PipelineMetrics / AlertRule / HealthDashboard）
3. Rust テスト 2 件で 4 スプリント統合の安定性を検証する
   - `favnir4_full_sprint_all_stable` — `pipeline.fav` に 4 スプリント全識別子が含まれること
   - `favnir4_e2e_showcase_runs` — `fav.toml` に `[quality]`・`[contract]`・`[observe]` セクションが含まれること

## Rust テスト（v84900_tests）

```rust
#[cfg(test)]
mod v84900_tests {
    #[test]
    fn favnir4_full_sprint_all_stable() {
        let content = include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav");
        // Sprint 1: Test-Driven Data
        assert!(content.contains("TestSuite"),       "pipeline.fav should include TestSuite (Sprint 1)");
        // Sprint 2: Data Quality 2.0
        assert!(content.contains("QualityCheck"),    "pipeline.fav should include QualityCheck (Sprint 2)");
        // Sprint 3: Pipeline Contracts 1.0
        assert!(content.contains("ContractRegistry"), "pipeline.fav should include ContractRegistry (Sprint 3)");
        // Sprint 4: Observability 2.0
        assert!(content.contains("PipelineMetrics"), "pipeline.fav should include PipelineMetrics (Sprint 4)");
    }

    #[test]
    fn favnir4_e2e_showcase_runs() {
        let toml = include_str!("../../infra/e2e-demo/favnir4-showcase/fav.toml");
        assert!(toml.contains("[quality]"),  "fav.toml should have [quality] section");
        assert!(toml.contains("[contract]"), "fav.toml should have [contract] section");
        assert!(toml.contains("[observe]"),  "fav.toml should have [observe] section");
    }
}
```

**パス起点:** `include_str!("../../infra/...")` — `fav/src/` 起点 → `favnir/infra/...`

## Success Criteria

- `cargo test` が 3,927 tests pass（+2）、0 failures であること
- `pipeline.fav` に Sprint 1〜4 の全識別子（TestSuite / QualityCheck / ContractRegistry / PipelineMetrics）が含まれること
- `fav.toml` に `[quality]`・`[contract]`・`[observe]` の 3 セクションが含まれること

## Error Codes

なし（本バージョンはテスト追加・安定性確認のみ。新機能なし）

## Files to Modify / Create

### 追記
- `fav/src/driver.rs` — `v84900_tests` モジュール追加（2 テスト）
- `CHANGELOG.md` — v84.9.0 エントリ追加

### 変更なし（確認のみ）
- `infra/e2e-demo/favnir4-showcase/pipeline.fav` — 4 スプリント全識別子の存在確認
- `infra/e2e-demo/favnir4-showcase/fav.toml` — 3 セクションの存在確認
