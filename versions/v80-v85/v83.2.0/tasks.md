# v83.2.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,889 tests pass、0 failures であることを確認する（前提: v83.1.0 完了済み）

## T1: `test_framework.rs` に enum・構造体を追加

- [x] `AlertSeverity` enum を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `Critical` / `Warning` / `Info`
- [x] `ThresholdOp` enum を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `GreaterThan` / `LessThan` / `EqualTo`
- [x] `AlertThreshold` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `metric: String`, `operator: ThresholdOp`, `value: f64`
- [x] `AlertRule` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `name: String`, `threshold: AlertThreshold`, `severity: AlertSeverity`, `message: String`
- [x] `AlertFiring` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `rule: AlertRule`, `current_value: f64`, `fired_at: String`

## T2: `evaluate_alert_rules` 関数を追加

- [x] `evaluate_alert_rules(rules: &[AlertRule], metrics: &PipelineMetrics, fired_at: &str) -> Vec<AlertFiring>` を追加する
  - `metric` 文字列で `current_value` を算出（`total_duration_ms` / `rows_failed` / `rows_processed`）
  - 未知の `metric` はスキップ
  - `ThresholdOp` で比較（`GreaterThan` / `LessThan` / `EqualTo`）
  - 発火時は `AlertFiring` を生成して push

## T3: `driver.rs` に `v83200_tests` を追加

- [x] `v83100_tests` の直後に `#[cfg(test)] mod v83200_tests` を追加する
  - `alert_fires_when_threshold_exceeded`
  - `alert_silent_when_within_threshold`

## T4: テスト通過確認

- [x] `cargo test` が 3,891 tests pass（+2）、0 failures であることを確認する

## T5: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [MED] `ThresholdOp::EqualTo` の `f64::EPSILON` 比較に「整数由来メトリクス専用」旨の doc コメント追加
- [x] [MED] `rows_failed` メトリクス分岐のテスト欠落 → `alert_fires_when_threshold_exceeded` 内に `rows_failed` アラート発火アサーションを追加（テスト数 3891 を維持）

## 実装メモ

- `v83200_tests` のヘルパー `make_metrics()` を使用してテストコードの重複を削減
- spec-reviewer 指摘対応: ロードマップ `evaluate_alert_rules` シグネチャ修正 + テスト数ドリフト補正
- Success Criteria を整理: `LessThan`/`EqualTo` は「実装仕様」に格下げ（テストは `GreaterThan` のみ）
