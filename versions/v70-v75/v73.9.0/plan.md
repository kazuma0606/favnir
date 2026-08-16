# v73.9.0 実装計画 — 安定化・コードフリーズ（Production Proven 前調整）

Date: 2026-08-13

---

## 実装ステップ

### Step 1: `v739000_tests` モジュールを追加

`v738000_tests` の直後に以下のモジュールを追加する。

```rust
#[cfg(test)]
mod v739000_tests {
    use super::*;

    #[test]
    fn production_proven_all_stable() {
        // v73.1: DataContract
        let contract = DataContract {
            name: "test".to_string(),
            input_fields: vec![],
            output_fields: vec![],
            sla: DataContractSla { max_latency_ms: 1000, min_throughput: 100, max_error_rate: 0.05 },
        };
        let _ = validate_contract_schema(&contract, &[]);

        // v73.2: QualityReport
        let report = compute_quality_report(&[]);
        assert!(report.overall_score <= 100);

        // v73.3: PII
        let fields = vec![("email".to_string(), "secret@example.com".to_string())];
        let masked = mask_pii_fields(&fields, PiiMaskStrategy::Hash);
        assert!(!masked.is_empty());

        // v73.4: AuditLog
        let entry = AuditLogEntry {
            run_id: "r1".to_string(),
            parent_run_id: None,
            pipeline_name: "test".to_string(),
            status: "ok".to_string(),
            started_at: "2026-08-13T00:00:00Z".to_string(),
            ended_at: None,
            row_count: None,
        };
        let jsonl = format_audit_log_entry(&entry);
        assert!(jsonl.contains("run_id"));

        // v73.5: SLA
        let sla = SlaConfig {
            max_latency_ms: 1000,
            min_throughput: 100,
            max_error_rate: 0.05,
        };
        let violations = check_sla(&sla, 500, 200, 0.01);
        assert!(violations.is_empty());

        // v73.6: Linalg
        let a = RuneLinalgMatrix { rows: 1, cols: 1, data: vec![2.0] };
        let b = RuneLinalgMatrix { rows: 1, cols: 1, data: vec![3.0] };
        let result = rune_linalg_matmul(&a, &b).expect("matmul should succeed");
        assert_eq!(result.data[0], 6.0);

        // v73.7: Dogfooding
        let pipelines = list_dogfooding_pipelines();
        assert_eq!(pipelines.len(), 5);

        // v73.8: GitHub Action
        let cfg = GithubActionConfig {
            version: "73.9.0".to_string(),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let url = format_github_action_url(&cfg);
        assert!(url.starts_with("https://github.com/favnir/favnir/releases/download/"));
    }

    #[test]
    fn dogfooding_all_5_pipelines_pass() {
        let pipelines = list_dogfooding_pipelines();
        assert_eq!(pipelines.len(), 5);

        let expected_names = [
            "benchmark_analytics",
            "coverage_report",
            "changelog_lint",
            "rune_catalog_sync",
            "doc_link_check",
        ];
        let names: Vec<&str> = pipelines.iter().map(|p| p.name.as_str()).collect();
        for name in &expected_names {
            assert!(names.contains(name), "missing pipeline: {}", name);
        }

        // 全 path が "pipelines/*.fav" 形式
        for p in &pipelines {
            assert!(p.path.starts_with("pipelines/"), "bad path: {}", p.path);
            assert!(p.path.ends_with(".fav"), "bad path: {}", p.path);
            assert!(!p.description.is_empty(), "empty description for: {}", p.name);
        }

        // 各 .fav ファイルが実際に存在し pipeline 名を含む（コンパイル時チェック）
        assert!(include_str!("../pipelines/benchmark_analytics.fav").contains("benchmark_analytics"));
        assert!(include_str!("../pipelines/coverage_report.fav").contains("coverage_report"));
        assert!(include_str!("../pipelines/changelog_lint.fav").contains("changelog_lint"));
        assert!(include_str!("../pipelines/rune_catalog_sync.fav").contains("rune_catalog_sync"));
        assert!(include_str!("../pipelines/doc_link_check.fav").contains("doc_link_check"));
    }
}
```

### Step 2: バージョン更新

- `fav/Cargo.toml`: `version = "73.8.0"` → `version = "73.9.0"`
- `driver.rs` 内の `version = "73.8.0"` 参照を `version = "73.9.0"` に replace_all

### Step 3: テスト確認

- `cargo test v739000` で 2 件 pass を確認
- `cargo test` 全体で 3665 tests pass を確認

### Step 4: `CHANGELOG.md` 更新

- v73.9.0 エントリを先頭に追加

### Step 5: `versions/current.md` 更新

- 最終更新を `2026-08-13 (v73.9.0)` に変更
- 進行中を `v73.9.0` に変更
- 次を `v74.0.0` に変更
