# v82.2.0 — `SlaContract`（SLA 遵守型）

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 2 版。
応答時間・スループット・可用性の SLA（Service Level Agreement）を型として宣言し、
Favnir 3.0 の `!Adaptive` / `!Cached` エフェクトと連携させる仕組みを構築する。

`SlaContract` はパイプラインが「どれだけ速く、どれだけ安定して動くべきか」を
型で表現し、実測値と照合して `SlaStatus` を返す。

---

## Goals

1. `SlaTarget` 構造体を定義する（`max_latency_ms: u64`, `min_throughput_rps: f64`, `min_availability_pct: f64`）
2. `SlaContract` 構造体を定義する（`name: String`, `target: SlaTarget`, `adaptive_strategy: Option<String>`, `cache_ttl_secs: Option<u64>`）
3. `SlaStatus` enum を定義する（`Met` / `AtRisk(String)` / `Breached(String)`）
4. `evaluate_sla(contract: &SlaContract, actual_latency_ms: u64, actual_rps: f64) -> SlaStatus` を実装する
5. `format_sla_status(status: &SlaStatus) -> String` を実装する

---

## API Examples（Rust テストコード）

```rust
let target = SlaTarget {
    max_latency_ms: 200,
    min_throughput_rps: 100.0,
    min_availability_pct: 99.9,
};
let contract = SlaContract {
    name: "orders_sla".into(),
    target,
    adaptive_strategy: Some("!Adaptive".into()),
    cache_ttl_secs: Some(60),
};

// SLA 充足: 実測レイテンシ 150ms、スループット 120 rps
let status = evaluate_sla(&contract, 150, 120.0);
assert!(matches!(status, SlaStatus::Met));

// SLA 違反: レイテンシが 200ms を超過
let status2 = evaluate_sla(&contract, 250, 120.0);
assert!(matches!(status2, SlaStatus::Breached(_)));

// format
let s = format_sla_status(&SlaStatus::Met);
assert_eq!(s, "SLA: Met");
```

### `evaluate_sla` の判定ロジック

1. `actual_latency_ms > target.max_latency_ms` → `Breached("latency exceeded: {actual} ms > {max} ms")`
2. `actual_rps < target.min_throughput_rps` → `Breached("throughput below minimum: {actual} rps < {min} rps")`
3. いずれも違反なし → `Met`

> **注記**: `AtRisk` および `min_availability_pct` の評価は将来拡張用（本バージョンでは使用しない）。

---

## Success Criteria

- `cargo test` 全 pass（3,869 tests = 3,867 + 2）
- 新規テスト 2 件（`v82200_tests` モジュール）:
  - `sla_contract_met_within_target`
  - `sla_contract_breached_over_latency`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `SlaTarget` / `SlaContract` / `SlaStatus` / `evaluate_sla` / `format_sla_status` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82200_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.2.0 エントリを先頭に追加 |
