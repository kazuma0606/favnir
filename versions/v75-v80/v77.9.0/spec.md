# v77.9.0 仕様書 — 安定化・コードフリーズ

Date: 2026-08-16
Status: 計画中

---

## Background

v77.1〜v77.8 の全機能（PipelineInvariant / AggregateInvariant / FilterInvariant / JoinInvariant / VerificationReport / CiResult / CounterExampleResult / ProbabilisticContract）を通しで確認する最終安定化スプリント。新機能追加は一切行わない。E2E スモークテスト 2 件を追加し、Verifiable Pipelines スプリント（v77.x）の安定性を保証する。

各型・関数の追加バージョン参照:
- `InvariantViolation` / `PipelineInvariant` / `InvariantCheckPoint`: v77.1.0
- `AggregateInvariant` / `AggregateProperty` / `check_aggregate_invariant`: v77.2.0 / v77.3.0
- `FilterInvariant` / `FilterPredicate` / `check_filter_invariant`: v77.3.0
- `JoinInvariant` / `JoinType` / `JoinNullPolicy` / `check_join_invariant`: v77.4.0
- `InvariantResult` / `VerificationReport` / `cmd_verify` / `format_verification_report`: v77.5.0
- `CiVerificationConfig` / `CiResult` / `run_ci_verification` / `format_ci_result_summary`: v77.6.0
- `CounterExampleResult` / `generate_counter_example_values`: v77.7.0
- `ProbabilisticContract` / `check_probabilistic_invariant`: v77.8.0

---

## Goals

1. `verifiable_full_sprint_all_stable` テストを追加する（v77.1〜v77.8 の全主要型が instantiate できることを確認）
2. `verifiable_e2e_pipeline_verified` テストを追加する（v77 型を組み合わせた E2E 合成テスト）
3. Rust テスト 2 件を追加し 3756 tests に到達する（現在 3754）
4. バグ修正のみ受け入れ（新機能追加なし）

> **注意**: ロードマップの目標テスト数は 3754 だったが、v77.8.0 で code-reviewer 指摘対応により 2 件追加した結果、現在 3754 に到達済み。v77.9.0 では +2 追加して 3756 を目標とする。

---

## テスト仕様

### `verifiable_full_sprint_all_stable`

v77.1〜v77.8 の全主要型を instantiate し、基本動作を確認するスモークテスト。

> 以下はテスト本体のロジックのみ。実際には `#[cfg(test)] mod v779000_tests { use super::*; ... }` でラップする（plan.md / tasks.md T2 参照）。

```rust
// v77.1: InvariantViolation / PipelineInvariant
let violation = InvariantViolation { column: "x".to_string(), actual: "0".to_string(), expected: "NonNegative".to_string() };
assert_eq!(violation.column, "x");

// v77.2: AggregateInvariant / check_aggregate_invariant
let agg_inv = AggregateInvariant { column: "amount".to_string(), property: AggregateProperty::NonNegative };
assert!(check_aggregate_invariant(&[1.0, 2.0], &agg_inv).is_ok());

// v77.3: FilterInvariant / check_filter_invariant
let filter_inv = FilterInvariant { name: "positive_only".to_string(), predicate: FilterPredicate::GreaterThan(0.0) };
assert!(check_filter_invariant(5.0, &filter_inv).is_ok());

// v77.4: JoinInvariant / check_join_invariant
let join_inv = JoinInvariant { name: "id_join".to_string(), join_type: JoinType::Inner, null_policy: JoinNullPolicy::RejectNull };
assert!(check_join_invariant(true, true, &join_inv).is_ok());

// v77.5: cmd_verify / VerificationReport
let inv = PipelineInvariant { name: "test_inv".to_string(), check_point: InvariantCheckPoint::PreStage, description: "".to_string() };
let report = cmd_verify("test_pipeline", &[inv]);
assert!(report.all_passed);

// v77.6: run_ci_verification / CiResult
let config = CiVerificationConfig { pipeline: "test".to_string(), fail_fast: false, data_path: "/tmp".to_string() };
let ci_result = run_ci_verification(&config, &[]);
assert_eq!(ci_result.exit_code, 0);

// v77.7: generate_counter_example_values / CounterExampleResult
let agg = AggregateInvariant { column: "score".to_string(), property: AggregateProperty::NonNegative };
let ce = generate_counter_example_values(&agg, 1);
assert!(!ce.example.is_empty());

// v77.8: check_probabilistic_invariant / ProbabilisticContract
let pc = ProbabilisticContract { name: "accuracy".to_string(), confidence: 0.95, sample_size: 100 };
assert!(check_probabilistic_invariant(&[0.9, 0.95, 0.92], 0.8, 1.0, &pc).is_ok());
```

### `verifiable_e2e_pipeline_verified`

v77 型を組み合わせた E2E 合成テスト。aggregate → filter → join → verify → ci の各レイヤーを通す。

```rust
// Step 1: AggregateInvariant 検証
let agg_inv = AggregateInvariant { column: "revenue".to_string(), property: AggregateProperty::NonNegative };
let agg_ok = check_aggregate_invariant(&[100.0, 200.0, 150.0], &agg_inv);
assert!(agg_ok.is_ok());

// Step 2: FilterInvariant 検証
let filter_inv = FilterInvariant { name: "positive_revenue".to_string(), predicate: FilterPredicate::GreaterThan(0.0) };
let filter_ok = check_filter_invariant(100.0, &filter_inv);
assert!(filter_ok.is_ok());

// Step 3: ProbabilisticContract 検証
let pc = ProbabilisticContract { name: "revenue_avg".to_string(), confidence: 0.95, sample_size: 1_000 };
let prob_ok = check_probabilistic_invariant(&[100.0, 200.0, 150.0], 50.0, 300.0, &pc);
assert!(prob_ok.is_ok());

// Step 4: cmd_verify → CiResult
let pipeline_inv = PipelineInvariant {
    name:        "revenue_non_negative".to_string(),
    check_point: InvariantCheckPoint::PostStage,
    description: "Revenue must be non-negative".to_string(),
};
let report = cmd_verify("revenue_pipeline", &[pipeline_inv]);
let config = CiVerificationConfig { pipeline: "revenue_pipeline".to_string(), fail_fast: false, data_path: "/data".to_string() };
let ci = run_ci_verification(&config, &[]);
assert_eq!(ci.exit_code, 0);
assert!(format_ci_result_summary(&ci).contains("passed"));
```

---

## Success Criteria

- `verifiable_full_sprint_all_stable` が pass（v77.1〜v77.8 全型の instantiate が成功）
- `verifiable_e2e_pipeline_verified` が pass（aggregate → filter → probabilistic → verify → ci の E2E 動作確認）
- `cargo test` が 3756 tests all pass
- `driver.rs` 内の `cargo_toml_version_is_X` 系テストの `77.8.0` バージョン文字列アサーションがすべて `77.9.0` に更新されている（セクションコメント `// --- v77.8.0: 確率的契約 ---` は変更しない）
- `CHANGELOG.md` の先頭に v77.9.0 エントリが存在する

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `v779000_tests` モジュールを追加（2 テスト）
- `CHANGELOG.md` — v77.9.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `77.8.0` → `77.9.0` に更新
- `fav/Cargo.lock` — `Cargo.toml` バージョン更新に伴い自動更新（手動編集不要）

---

## 対象外

- 新機能追加: 一切行わない（バグ修正のみ受け入れ）
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
- 統計的検定（t 検定・信頼区間）: v78.x 以降
