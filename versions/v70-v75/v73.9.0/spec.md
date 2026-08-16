# v73.9.0 仕様書 — 安定化・コードフリーズ（Production Proven 前調整）

Date: 2026-08-13

---

## Background

v73.1〜v73.8 で実装した以下の機能が正常に統合されていることを確認する安定化バージョン。
v74.0.0 の Production Proven 宣言の前に、全機能の安定性を Rust テストで保証する。

| バージョン | 主な機能 |
|---|---|
| v73.1.0 | データコントラクト（DataContract / validate_contract） |
| v73.2.0 | 品質スコア（QualityScore / compute_quality_score） |
| v73.3.0 | PII 検出・マスキング（PiiField / mask_pii_value） |
| v73.4.0 | 監査ログ + OpenLineage（AuditLogEntry / OpenLineageEvent） |
| v73.5.0 | SLA 監視（SlaConfig / check_sla） |
| v73.6.0 | Rune 品質パス（RuneLinalgMatrix / RuneStatsResult） |
| v73.7.0 | ドッグフーディング Sprint（DogfoodingPipeline / list_dogfooding_pipelines） |
| v73.8.0 | GitHub Actions 公式 Action（GithubActionConfig / format_github_action_url） |

---

## Goals

1. v73.1〜v73.8 の全主要関数がビルドエラー・テスト失敗なく動作することを確認するテストを追加
2. ドッグフーディング 5 パイプラインが全て存在し、期待する名前を含むことを確認するテストを追加
3. `cargo test` で 3665 tests pass（0 failures）を達成する

---

## API 例（テストコード）

```rust
// production_proven_all_stable: v73.1〜v73.8 の主要関数の呼び出し可能性を確認
fn production_proven_all_stable() {
    // v73.1: DataContract
    let _ = validate_contract_schema(&DataContract { name: "test".to_string(), input_fields: vec![], output_fields: vec![], sla: DataContractSla { .. } }, &[]);
    // v73.2: QualityReport
    let report = compute_quality_report(&[]);
    assert!(report.overall_score <= 100);
    // v73.3: PII
    let _ = mask_pii_fields(&[("email".to_string(), "secret".to_string())], PiiMaskStrategy::Hash);
    // v73.4: AuditLog
    let _ = format_audit_log_entry(&AuditLogEntry { ... });
    // v73.5: SLA
    let _ = check_sla(&SlaConfig { ... }, 100, 1000, 0.01);
    // v73.6: Linalg
    let _ = rune_linalg_matmul(&RuneLinalgMatrix { rows: 1, cols: 1, data: vec![1.0] }, ...);
    // v73.7: Dogfooding
    let _ = list_dogfooding_pipelines();
    // v73.8: GitHub Action
    let _ = format_github_action_url(&GithubActionConfig { ... });
}

// dogfooding_all_5_pipelines_pass: 5 本のパイプラインを全て検証
fn dogfooding_all_5_pipelines_pass() {
    let pipelines = list_dogfooding_pipelines();
    assert_eq!(pipelines.len(), 5);
    // 各 .fav ファイルが存在し、pipeline 名を含む
    // path 形式が "pipelines/*.fav" である
}
```

---

## Success Criteria

1. `production_proven_all_stable` テストが pass する（v73.1〜v73.8 の各関数が呼び出し可能）
2. `dogfooding_all_5_pipelines_pass` テストが pass する（5 パイプラインが全て存在）
3. `cargo test` で 3665 tests pass（0 failures）

---

## Error Codes

新規エラーコードなし（安定化・テスト追加のみ）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v739000_tests` モジュール追加（`production_proven_all_stable` / `dogfooding_all_5_pipelines_pass`） |
| `fav/Cargo.toml` | `version = "73.9.0"` に更新 |
| `CHANGELOG.md` | v73.9.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
