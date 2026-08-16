# Roadmap v77.1.0 〜 v78.0.0 — Verifiable Pipelines

Date: 2026-08-14
Status: 未着手（v77.0.0 完了後に開始）

マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)

---

## 前提

- 直前完了: v77.0.0「Data Provenance 1.0 宣言」（tests = 3736）
- 本スプリントは Phase 6「Favnir 3.0 宣言」の第 3 スプリント
- 目標: v78.0.0「Verifiable Pipelines 宣言」（tests = 3758）

### スプリントの性格

「テストを書く前に、コンパイラが反例を見つける」。
パイプラインの不変条件を `contract` ブロック内の `invariant` 節として宣言し、
`fav verify` で証明・反例生成・CI 統合を実現する。
A（新言語機能）60% + B（ツール）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v77.1.0 | `PipelineInvariant` 型基盤 | 3736 + 2 = 3738 | 未着手 |
| v77.2.0 | フィルター系不変条件 | 3738 + 2 = 3740 | 未着手 |
| v77.3.0 | 集約系不変条件 | 3740 + 2 = 3742 | 未着手 |
| v77.4.0 | Join 系不変条件 | 3742 + 2 = 3744 | 未着手 |
| v77.5.0 | `fav verify` コマンド | 3744 + 2 = 3746 | 未着手 |
| v77.6.0 | 証明付き CI 統合 | 3746 + 2 = 3748 | 未着手 |
| v77.7.0 | 反例自動生成 | 3748 + 2 = 3750 | 未着手 |
| v77.8.0 | Probabilistic contracts | 3750 + 2 = 3752 | 未着手 |
| v77.9.0 | 安定化・コードフリーズ | 3752 + 2 = 3754 | 未着手 |
| v78.0.0 | Verifiable Pipelines 宣言 ★クリーンアップ | 3754 + 4 = 3758 | 未着手 |

---

## v77.1.0 — `PipelineInvariant` 型基盤

パイプラインの不変条件を型として表現する基盤。`contract` ブロックに `invariant` 節を追加する。

```favnir
contract OrderPipeline {
    input:     { orders: List<Row> }
    output:    { processed: List<Row> }
    invariant: output.row_count <= input.row_count   // フィルターは増やさない
}
```

**実装内容:**
- `InvariantCheckPoint` enum（Pre, Post, Both）
- `PipelineInvariant` 構造体（name: String, expression: String, check_point: InvariantCheckPoint）
- `InvariantViolation` 構造体（invariant_name: String, expected: String, actual: String）
- `check_count_invariant(expected_max: usize, actual: usize, name: &str) -> Result<(), InvariantViolation>`

**完了条件**: Rust テスト 2 件（3736 + 2 = 3738）
- `invariant_count_passes`
- `invariant_count_violated`

---

## v77.2.0 — フィルター系不変条件

フィルター操作が持つべき性質（行数が減る・比率の上限など）を不変条件として検証する。

```favnir
contract FilterPipeline {
    invariant filter_reduces_rows:
        output.row_count < input.row_count
    invariant filter_ratio_reasonable:
        output.row_count as Float / input.row_count as Float >= 0.01
}
```

**実装内容:**
- `FilterInvariant` 構造体（expected_ratio_min: f64, expected_ratio_max: f64）
- `check_filter_invariant(input_count: usize, output_count: usize, inv: &FilterInvariant) -> Result<(), InvariantViolation>`
- `format_filter_invariant_report(inv: &FilterInvariant, result: &Result<(), InvariantViolation>) -> String`

**完了条件**: Rust テスト 2 件（3738 + 2 = 3740）
- `filter_invariant_ratio_valid`
- `filter_invariant_ratio_violated`

---

## v77.3.0 — 集約系不変条件

集約結果（SUM・COUNT・AVG）が持つべき数学的性質を不変条件として検証する。

```favnir
contract AggregatePipeline {
    invariant total_amount_non_negative: SUM(output.amount) >= 0.0
    invariant avg_score_bounded: AVG(output.score) BETWEEN 0.0 AND 100.0
}
```

**実装内容:**
- `AggregateProperty` enum（NonNegative, NonPositive, Bounded { min: f64, max: f64 }, NonNull）
- `AggregateInvariant` 構造体（column: String, property: AggregateProperty）
- `check_aggregate_invariant(values: &[f64], inv: &AggregateInvariant) -> Result<(), InvariantViolation>`

**完了条件**: Rust テスト 2 件（3740 + 2 = 3742）
- `aggregate_invariant_non_negative_passes`
- `aggregate_invariant_bounded_violated`

---

## v77.4.0 — Join 系不変条件

Join の種類に応じた不変条件（行数の増減・NULL 発生）を検証する。

```favnir
contract JoinPipeline {
    invariant left_join_preserves_left:
        output.row_count >= input.left.row_count
}
```

**実装内容:**
- `JoinType` enum（Inner, Left, Right, Full）
- `JoinNullPolicy` enum（Fail, Warn, Allow）
- `JoinInvariant` 構造体（join_type: JoinType, null_policy: JoinNullPolicy）
- `check_join_invariant(left_count: usize, result_count: usize, null_count: usize, inv: &JoinInvariant) -> Result<(), InvariantViolation>`

**完了条件**: Rust テスト 2 件（3742 + 2 = 3744）
- `join_invariant_inner_no_nulls`
- `join_invariant_left_preserves_rows`

---

## v77.5.0 — `fav verify` コマンド

コントラクトの不変条件をサンプルデータに対して検証するコマンド。

```bash
$ fav verify pipeline.fav --data data/sample.csv
Verifying OrderPipeline...
  ✓ filter_reduces_rows       (input=1000, output=847)
  ✓ total_amount_non_negative (sum=49823.5)
Verification passed. 2/2 invariants checked.
```

**実装内容:**
- `InvariantResult` 構造体（name: String, passed: bool, detail: String）
- `VerificationReport` 構造体（pipeline: String, results: Vec<InvariantResult>, all_passed: bool）
- `cmd_verify(pipeline_name: &str, invariants: &[PipelineInvariant]) -> VerificationReport`
- `format_verification_report(report: &VerificationReport) -> String`

**完了条件**: Rust テスト 2 件（3744 + 2 = 3746）
- `verify_cmd_all_pass`
- `verify_cmd_violation_reported`

---

## v77.6.0 — 証明付き CI 統合

CI パイプライン（GitHub Actions など）で `fav verify` を自動実行し、不変条件の証明をブロッカーにする。

```yaml
# .github/workflows/verify.yml
- name: Verify invariants
  run: fav verify pipelines/order.fav --fail-on-violation
```

**実装内容:**
- `CiVerificationConfig` 構造体（pipeline: String, fail_fast: bool, data_path: String）
- `CiResult` 構造体（passed: bool, report: VerificationReport, exit_code: i32）
- `run_ci_verification(config: &CiVerificationConfig, invariants: &[PipelineInvariant]) -> CiResult`
- `format_ci_result_summary(result: &CiResult) -> String`

**完了条件**: Rust テスト 2 件（3746 + 2 = 3748）
- `ci_verification_passes`
- `ci_verification_fails_on_violation`

---

## v77.7.0 — 反例自動生成

不変条件を「破る」サンプルデータを自動生成する。不変条件の設計ミスを早期に発見する。

```bash
$ fav verify --generate-counter-examples pipeline.fav
Generating counter-examples for: total_amount_non_negative
  Counter-example found: values=[-100.0] → sum=-100.0 (VIOLATES)
  → Invariant is reachable. Recommend adding input validation.
```

**実装内容:**
- `CounterExampleResult` 構造体（invariant_name: String, example: Vec<f64>, violates: bool）
- `generate_counter_example_values(inv: &AggregateInvariant, seed: u64) -> CounterExampleResult`
  — 境界値付近のサンプル（0.0, -0.001, f64::MIN 等）を生成して検証

**完了条件**: Rust テスト 2 件（3748 + 2 = 3750）
- `counter_example_finds_violation`
- `counter_example_none_for_trivially_valid`

---

## v77.8.0 — Probabilistic contracts

確率的にしか検証できない不変条件（サンプリングベース）を表現する。大規模データ向け。

```favnir
contract LargeDataPipeline {
    // 全件ではなくサンプルで検証
    invariant score_distribution:
        confidence: 0.95,
        sample_size: 10_000,
        property: AVG(score) BETWEEN 40.0 AND 60.0
}
```

**実装内容:**
- `ProbabilisticContract` 構造体（name: String, confidence: f64, sample_size: usize）
- `check_probabilistic_invariant(samples: &[f64], target_min: f64, target_max: f64, contract: &ProbabilisticContract) -> Result<(), String>`
  — サンプル平均が範囲内かを検証

**完了条件**: Rust テスト 2 件（3750 + 2 = 3752）
- `probabilistic_contract_passes`
- `probabilistic_contract_low_confidence_fails`

---

## v77.9.0 — 安定化・コードフリーズ（Verifiable Pipelines 前最終調整）

v77.1〜v77.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- v77.1〜v77.8 の全テスト通過確認（`cargo test` 全 pass）
- `fav verify` / `PipelineInvariant` / `ProbabilisticContract` の E2E 動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3754 + 2 = 3756）
> **注**: v77.8.0 で code-reviewer 指摘対応により +4 テスト追加（計画 +2 → 実績 +4）。ベースは 3754。
- `verifiable_full_sprint_all_stable`
- `verifiable_e2e_pipeline_verified`

---

## v78.0.0 — Verifiable Pipelines 宣言 ★クリーンアップ

**宣言文**:
> 「不変条件が型となり、反例がコンパイラから届く。
>  Favnir のパイプラインは今、その正しさを証明できる。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `78.0.0` に更新
- `CHANGELOG.md` に v78.0.0 エントリを追加
- `MILESTONE.md` に「Verifiable Pipelines」を追記
- `README.md` に v78.0 達成を追記
- `versions/current.md` を更新

**完了条件**: `v78000_tests` 4 件（3756 + 4 = 3760）
- `cargo_toml_version_is_78_0_0`
- `changelog_has_v78_0_0`
- `milestone_has_verifiable_pipelines`
- `readme_mentions_verifiable_pipelines`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v77.0.0（ベース） | 3,736 | — |
| v77.1.0 | 3,738 | +2 |
| v77.2.0 | 3,740 | +2 |
| v77.3.0 | 3,742 | +2 |
| v77.4.0 | 3,744 | +2 |
| v77.5.0 | 3,746 | +2 |
| v77.6.0 | 3,748 | +2 |
| v77.7.0 | 3,750 | +2 |
| v77.8.0 | 3,754 | +4（code-reviewer 指摘対応で +2 追加） |
| v77.9.0 | 3,756 | +2 |
| v78.0.0（宣言） | 3,760 | +4 |

**本スプリント合計**: +24 tests（3,736 → 3,760）

---

## 参考リンク

- マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)
- 前スプリント: [roadmap-v76.1-v77.0.md](roadmap-v76.1-v77.0.md)
- 次スプリント: [roadmap-v78.1-v79.0.md](roadmap-v78.1-v79.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
