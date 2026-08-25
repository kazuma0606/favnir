# v82.2.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,867 tests pass、0 failures であることを確認する（前提: v82.1.0 完了済み）

## T1: `SlaTarget` 構造体追加

- [x] `fav/src/test_framework.rs` に `SlaTarget` 構造体を追加する
  - `max_latency_ms: u64` / `min_throughput_rps: f64` / `min_availability_pct: f64`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T2: `SlaContract` 構造体追加

- [x] `fav/src/test_framework.rs` に `SlaContract` 構造体を追加する
  - `name: String` / `target: SlaTarget` / `adaptive_strategy: Option<String>` / `cache_ttl_secs: Option<u64>`
  - `#[derive(Debug, Clone)]` を付与する

## T3: `SlaStatus` enum 追加

- [x] `fav/src/test_framework.rs` に `SlaStatus` enum を追加する
  - variants: `Met` / `AtRisk(String)` / `Breached(String)`
  - `#[derive(Debug, PartialEq)]` を付与する

## T4: `evaluate_sla` + `format_sla_status` 関数追加

- [x] `evaluate_sla(contract, actual_latency_ms, actual_rps) -> SlaStatus` を実装する
  - レイテンシ超過 → `Breached("latency exceeded: ...")`
  - スループット不足 → `Breached("throughput below minimum: ...")`
  - いずれも違反なし → `Met`
- [x] `format_sla_status(status) -> String` を実装する
  - `Met` → `"SLA: Met"`
  - `AtRisk(msg)` → `"SLA: AtRisk — {msg}"`
  - `Breached(msg)` → `"SLA: Breached — {msg}"`

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.2.0 エントリを追加する

## T6: `v82200_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82200_tests` を追加する
  - `sla_contract_met_within_target`: レイテンシ・スループット目標内 → `Met`・`"SLA: Met"` を確認
  - `sla_contract_breached_over_latency`: レイテンシ超過 → `Breached`・メッセージに数値を含む

## T7: テスト通過確認

- [x] `cargo test` が 3,869 tests pass（+2）、0 failures であることを確認する

## T8: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
