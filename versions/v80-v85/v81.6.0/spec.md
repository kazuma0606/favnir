# Spec: v81.6.0 — 品質ゲート（`QualityGate` / パイプライン停止条件）

## Background

v81.4.0 で `QualityScore` / `QualityDimension` を導入した。
本バージョンでは品質スコアが閾値を下回った場合にパイプラインを停止する仕組み（品質ゲート）を追加する。

ロードマップ: `versions/roadmap/roadmap-v81.1-v82.0.md`（v81.6.0 セクション）

> **テスト数**: 実際のベースは **3853**（v81.5.0 完了後）。
> 本バージョンの完了条件は **3853 + 2 = 3855**。

## Goals

- `GateDecision` enum を `test_framework.rs` に追加する
- `QualityGate` 構造体を追加する
- `QualityGate::strict()` / `QualityGate::permissive()` コンストラクタを実装する
- `evaluate_quality_gate(gate: &QualityGate, score: &QualityScore) -> GateDecision` を実装する
- `format_gate_decision(decision: &GateDecision) -> String` を実装する
- テスト 2 件を追加して **3855 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

/// 品質ゲートの評価結果。
///
/// - `Pass`: すべての条件を満たした
/// - `Fail(String)`: 最初に失敗した条件の説明
/// - `Warn(String)`: 条件は満たしているが注意が必要（将来拡張用）
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    Pass,
    Fail(String),
    Warn(String),
}

/// パイプライン停止条件を定義する品質ゲート。
///
/// - `min_overall_score`: `QualityScore.overall` がこの値を下回ると `Fail`
/// - `required_dimensions`: この次元のスコアをチェックする対象リスト
/// - `min_dimension_score`: 各 required 次元がこの値を下回ると `Fail`
#[derive(Debug, Clone)]
pub struct QualityGate {
    pub min_overall_score: f64,
    pub required_dimensions: Vec<QualityDimension>,
    pub min_dimension_score: f64,
}

impl QualityGate {
    /// 全ディメンション 0.9 以上を要求する厳格ゲート。
    pub fn strict() -> QualityGate;

    /// overall 0.6 以上のみを要求する緩やかなゲート（required_dimensions = 空）。
    pub fn permissive() -> QualityGate;
}

/// `gate` の条件に対して `score` を評価する。
///
/// 評価順序:
/// 1. `score.overall < gate.min_overall_score` → `Fail("overall score X below minimum Y")`
/// 2. `gate.required_dimensions` の各次元について、`score.dimensions` に該当スコアが存在しない
///    またはスコアが `gate.min_dimension_score` を下回る → `Fail("dimension X score Y below minimum Z")`
/// 3. すべて通過 → `Pass`
pub fn evaluate_quality_gate(gate: &QualityGate, score: &QualityScore) -> GateDecision;

/// `GateDecision` を人間向けの文字列に変換する。
///
/// - `Pass`       → `"PASS"`
/// - `Fail(msg)`  → `"FAIL: {msg}"`
/// - `Warn(msg)`  → `"WARN: {msg}"`
pub fn format_gate_decision(decision: &GateDecision) -> String;
```

### 出力例

```text
// 概念説明（Favnir 風疑似コード）
bind gate <- QualityGate::strict();
// gate.min_overall_score == 0.9
// gate.min_dimension_score == 0.9
// gate.required_dimensions == [Completeness, Consistency, Timeliness, Accuracy, Validity]

bind score_high <- QualityScore { dimensions: vec![], overall: 0.95 };
// evaluate_quality_gate(&gate, &score_high) → Pass（ただし required_dimensions が空の場合）

bind gate_perm <- QualityGate::permissive();
// gate_perm.min_overall_score == 0.6
// gate_perm.required_dimensions == []
bind score_low <- QualityScore { dimensions: vec![], overall: 0.5 };
// evaluate_quality_gate(&gate_perm, &score_low) → Fail("overall score 0.500 below minimum 0.600")
```

## Success Criteria

- `cargo test` が **3855 tests**, 0 failures
- `quality_gate_fails_below_threshold`:
  - `QualityGate::permissive()` に `overall = 0.5` を渡すと `Fail` を返すことを確認する
  - `format_gate_decision` の出力が `"FAIL"` を含むことを確認する
  - `overall` と `min_overall_score` の値が Fail メッセージに含まれることを確認する
- `quality_gate_passes_above_threshold`:
  - `QualityGate::permissive()` に `overall = 0.8` を渡すと `Pass` を返すことを確認する
  - `QualityGate::strict()` に `overall = 0.95` かつ全次元スコアが 0.95 を渡すと `Pass` を返すことを確認する
  - `format_gate_decision(&GateDecision::Pass)` が `"PASS"` を返すことを確認する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `GateDecision` / `QualityGate` / `QualityGate::strict` / `QualityGate::permissive` / `evaluate_quality_gate` / `format_gate_decision` |
| `fav/src/driver.rs` | 追記 | `mod v81600_tests`（テスト 2 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `QualityGate::strict()` の `required_dimensions` は全 5 次元（`Completeness` / `Consistency` / `Timeliness` / `Accuracy` / `Validity`）を含む。
- `evaluate_quality_gate` は required_dimensions の判定に `score.dimensions` を `dimension` フィールドで検索する。該当する `DimensionScore` が見つからない場合は「スコア不明」として `Fail` とする。
- `Warn` variant は将来拡張のために定義するが、`evaluate_quality_gate` は現バージョンでは `Warn` を返さない。
- `format_gate_decision` の出力の `{msg}` は内部文字列をそのまま使用する（追加フォーマットなし）。
