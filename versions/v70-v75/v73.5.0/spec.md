# v73.5.0 仕様書 — SLA 監視 + アラート統合

Date: 2026-08-13
Status: 計画中

---

## Background

Favnir v73.4.0 で監査ログと OpenLineage エクスポートを実装した。
v73.5.0 では「SLA 監視」と「アラート統合」を実装し、
エンタープライズパイプラインが要求するサービスレベル合意の自動検証・違反通知を提供する。

> **注**: ロードマップには `--enforce-sla` CLI フラグや Slack / PagerDuty への HTTP 送信が記載されているが、
> 本バージョンでは SLA 設定の解析（`SlaConfig`）・違反判定（`check_sla`）・アラートフォーマット（`format_sla_alert`）の実装に絞る。
> CLI フラグ組み込みと外部 HTTP 通知は将来バージョン（v73.7.0 ドッグフーディング Sprint 等）で実施する。
>
> また、ロードマップは「`toml.rs` 拡張」と明記しているが、本バージョンでは `driver.rs` 内の行パースで完結する。
> `toml.rs` への本格的な SLA セクション統合は v74.x 以降で実施予定。

---

## Goals

| 優先度 | 目標 |
|---|---|
| P0 | `SlaConfig` 構造体 — `max_latency_ms` / `min_throughput` / `max_error_rate` を保持 |
| P0 | `SlaAlertConfig` 構造体 — `slack` / `pagerduty` の通知先 URL / キーを保持 |
| P0 | `parse_sla_config(toml_str)` — TOML 文字列から `SlaConfig` を解析（`serde_json` 不使用、文字列パース） |
| P0 | `check_sla(config, actual_latency_ms, actual_throughput, actual_error_rate)` — SLA 違反を検出して `Vec<String>` を返す |
| P0 | `format_sla_alert(violations)` — 違反リストをアラートメッセージ文字列に整形 |
| P0 | `v735000_tests` — 2 件（`sla_violation_triggers_alert` / `sla_toml_config_parsed`） |

---

## API 設計

### `SlaConfig` / `SlaAlertConfig`

```rust
pub struct SlaConfig {
    pub max_latency_ms: u64,
    pub min_throughput: u64,
    pub max_error_rate: f64,
}

pub struct SlaAlertConfig {
    pub slack: Option<String>,
    pub pagerduty: Option<String>,
}
```

### `parse_sla_config`

```rust
pub fn parse_sla_config(toml_str: &str) -> Result<SlaConfig, String>
// TOML 文字列を受け取り、[sla] セクションの値を解析
// max_latency_ms / min_throughput / max_error_rate が欠落している場合は
//   Err("missing required sla fields: max_latency_ms, min_throughput, max_error_rate")
// 環境変数展開はスコープ外（リテラル値のみ対応）
```

### `check_sla`

```rust
pub fn check_sla(
    config: &SlaConfig,
    actual_latency_ms: u64,
    actual_throughput: u64,
    actual_error_rate: f64,
) -> Vec<String>
// 違反がなければ空 Vec を返す
// 違反項目ごとに文字列を追加:
//   "latency exceeded: {actual}ms > {max}ms"
//   "throughput below: {actual} < {min} rows/sec"
//   "error_rate exceeded: {actual:.2}% > {max:.2}%"
```

### `format_sla_alert`

```rust
pub fn format_sla_alert(violations: &[String]) -> String
// 違反なし → "All SLA conditions met."
// 違反あり → "[SLA ALERT] N violation(s):\n  - violation1\n  - violation2"
```

---

## サンプル出力

```
[SLA ALERT] 2 violation(s):
  - latency exceeded: 6200ms > 5000ms
  - error_rate exceeded: 2.50% > 1.00%
```

```
All SLA conditions met.
```

---

## スコープ外

- `--enforce-sla` CLI フラグ（`main.rs` への組み込み）
- Slack / PagerDuty への実際の HTTP 通知
- `[sla.alerts]` セクションのパース（`SlaAlertConfig` は構造体のみ定義、パースはスコープ外）
- 環境変数展開（`${PAGERDUTY_KEY}` 等）

---

## 成功条件

1. `cargo build` がエラーなし
2. `cargo test v735000` で 2 件 pass
3. `cargo test` 全体で 3657 tests pass（3655 + 2）
4. `fav/Cargo.toml` version = "73.5.0"
5. `CHANGELOG.md` に `[v73.5.0]` エントリあり
6. `versions/current.md` の進行中バージョンが v73.5.0

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `SlaConfig` / `SlaAlertConfig` / `parse_sla_config` / `check_sla` / `format_sla_alert` 追加、`v735000_tests` モジュール追加 |
| `fav/Cargo.toml` | version → "73.5.0" |
| `CHANGELOG.md` | v73.5.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン・次バージョン更新 |
