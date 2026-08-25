# v83.3.0 仕様書 — `SloTarget` / `SloStatus`（SLO 型）

## Background

v83.2.0 でアラートルール型が導入された。次のステップとして、
サービスレベル目標（SLO）を型で宣言し、エラーバジェットの消費状況を追跡する。

SLO（Service Level Objective）は v82.x.x の `SlaStatus`（SLA チェック結果）とは別概念:
- `SlaContract` / `SlaStatus` — ステージ実行時間の事前宣言型チェック（v82.3.0）
- `SloTarget` / `SloStatus` — good_events / total_events ベースの目標達成率追跡（本バージョン）

本バージョンは Observability 2.0 スプリント（v83.1〜v84.0）の第 3 段階。

ロードマップ参照: `versions/roadmap/roadmap-v83.1-v84.0.md` — v83.3.0 セクション

## Goals

1. `SloTarget` 構造体を追加する（SLO の目標宣言）
2. `SloMeasurement` 構造体を追加する（観測期間内の good/total イベント数）
3. `SloStatus` 構造体を追加する（目標達成率・エラーバジェット残量・breach 判定）
4. `compute_slo_status(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus` を追加する
5. `format_slo_status(status: &SloStatus) -> String` を追加する

## 型定義・API

```rust
/// SLO の目標宣言。
#[derive(Debug, Clone, PartialEq)]
pub struct SloTarget {
    pub name: String,
    pub objective_pct: f64,  // 例: 99.9
    pub window_hours: u64,   // 観測窓（時間）
}

/// 観測期間内の good/total イベント数。
#[derive(Debug, Clone, PartialEq)]
pub struct SloMeasurement {
    pub good_events: u64,
    pub total_events: u64,
    pub window_hours: u64,
}

/// SLO の達成状況。
#[derive(Debug, Clone, PartialEq)]
pub struct SloStatus {
    pub target: SloTarget,
    pub current_pct: f64,                    // good_events / total_events * 100.0
    pub error_budget_remaining_pct: f64,     // ((current_pct - objective_pct) / (100.0 - objective_pct)) * 100.0
    pub breached: bool,                      // current_pct < objective_pct
}

/// `SloMeasurement` から `SloStatus` を計算する。
///
/// `total_events == 0` の場合:
/// - `current_pct = 100.0`（イベントなし = 全成功とみなす）
/// - `error_budget_remaining_pct = 100.0`
/// - `breached = false`
pub fn compute_slo_status(target: &SloTarget, measurement: &SloMeasurement) -> SloStatus

/// SLO 達成状況のテキストサマリーを返す。
///
/// 例（目標達成）:
/// "SLO: api_availability\nObjective: 99.9%\nCurrent: 99.95%\nError Budget: 50.00% remaining\nStatus: OK"
///
/// 例（違反）:
/// "SLO: api_availability\nObjective: 99.9%\nCurrent: 99.80%\nError Budget: -200.00% remaining\nStatus: BREACHED"
pub fn format_slo_status(status: &SloStatus) -> String
```

## 計算式

```
current_pct = good_events / total_events * 100.0

error_budget_consumed_pct = objective_pct - current_pct   // 負の場合はバジェット未消費
error_budget_total_pct = 100.0 - objective_pct            // エラーバジェット全体
error_budget_remaining_pct = (current_pct - objective_pct) / (100.0 - objective_pct) * 100.0

breached = current_pct < objective_pct
```

## テスト（v83.3.0 で追加）

実際のテスト数ベース（※ drift 補正後）: **3891 + 2 = 3893**

（ロードマップ記載値 3879 + 2 = 3881 は旧バージョン到達時点のドリフト。
 実際の v83.2.0 完了テスト数は 3891。）

### `slo_status_within_budget`

```rust
let target = SloTarget {
    name: "api_availability".into(),
    objective_pct: 99.9,
    window_hours: 720,
};
let measurement = SloMeasurement {
    good_events: 99950,
    total_events: 100000,
    window_hours: 720,
};
// current_pct = 99.95 ≥ 99.9 → breached = false
let status = compute_slo_status(&target, &measurement);
assert!(!status.breached, "SLO should not be breached");
assert!((status.current_pct - 99.95).abs() < 0.001, "current_pct should be ~99.95");
assert!(status.error_budget_remaining_pct > 0.0, "error budget should be positive");
```

### `slo_status_breached`

```rust
let target = SloTarget {
    name: "api_availability".into(),
    objective_pct: 99.9,
    window_hours: 720,
};
let measurement = SloMeasurement {
    good_events: 99800,
    total_events: 100000,
    window_hours: 720,
};
// current_pct = 99.80 < 99.9 → breached = true
let status = compute_slo_status(&target, &measurement);
assert!(status.breached, "SLO should be breached");
assert!((status.current_pct - 99.80).abs() < 0.001, "current_pct should be ~99.80");
assert!(status.error_budget_remaining_pct < 0.0, "error budget should be negative when breached");
```

## Success Criteria

- `cargo test` が 3893 tests pass（+2）、0 failures
- `total_events == 0` のとき `current_pct = 100.0`、`breached = false`
- `format_slo_status` が "SLO:"、"Objective:"、"Current:"、"Error Budget:"、"Status:" を含む文字列を返す

## Files to Modify

- `fav/src/test_framework.rs` — 型定義・関数追加
- `fav/src/driver.rs` — `v83300_tests` モジュール追加
