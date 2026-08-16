# v73.5.0 タスクリスト — SLA 監視 + アラート統合

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.4.0` であることを確認
- [x] `cargo test` が 3655 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v734000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v735000_tests` が未存在であることを確認

---

## T1: `SlaConfig` / `SlaAlertConfig` 構造体追加

- [x] `SlaConfig { max_latency_ms: u64, min_throughput: u64, max_error_rate: f64 }` を `driver.rs` に追加した
- [x] `SlaAlertConfig { slack: Option<String>, pagerduty: Option<String> }` を `driver.rs` に追加した
- [x] 両方 `pub struct` であることを確認
- [x] `SlaConfig` に `#[derive(Debug)]` を付与した（`.expect()` が Debug を要求するため）
- [x] `cargo build` でエラーがないことを確認

---

## T2: `parse_sla_config` 追加

- [x] `pub fn parse_sla_config(toml_str: &str) -> Result<SlaConfig, String>` を実装した
  - `[sla]` セクションを行パースで検出
  - `max_latency_ms` / `min_throughput` / `max_error_rate` を解析
  - いずれかが欠落 → `Err("missing required sla fields: ...")`
  - `[sla.alerts]` セクションは無視（スコープ外）
- [x] `cargo build` でエラーがないことを確認

---

## T3: `check_sla` 追加

- [x] `pub fn check_sla(config: &SlaConfig, actual_latency_ms: u64, actual_throughput: u64, actual_error_rate: f64) -> Vec<String>` を実装した
  - `actual_latency_ms > config.max_latency_ms` → `"latency exceeded: Xms > Yms"`
  - `actual_throughput < config.min_throughput` → `"throughput below: X < Y rows/sec"`
  - `actual_error_rate > config.max_error_rate` → `"error_rate exceeded: X.XX% > Y.YY%"`
  - 違反なし → 空 `Vec`
- [x] `cargo build` でエラーがないことを確認

---

## T4: `format_sla_alert` 追加

- [x] `pub fn format_sla_alert(violations: &[String]) -> String` を実装した
  - 空 → `"All SLA conditions met."`
  - 非空 → `"[SLA ALERT] N violation(s):\n  - v1\n  - v2"`
- [x] `cargo build` でエラーがないことを確認

---

## T5: `v735000_tests` モジュール追加

- [x] `v734000_tests` の直後に `v735000_tests` モジュールを追加した
- [x] `use super::{SlaConfig, parse_sla_config, check_sla, format_sla_alert}` を追加した
- [x] `sla_violation_triggers_alert` テストを実装した
  - 全条件違反 → `violations.len() == 3` を assert
  - 各違反メッセージに `"latency exceeded"` / `"throughput below"` / `"error_rate exceeded"` を assert
  - `format_sla_alert` で `"[SLA ALERT]"` / `"3 violation(s)"` / `"6200ms > 5000ms"` を assert
  - 全条件 OK → `ok.is_empty()` / `"All SLA conditions met."` を assert
- [x] `sla_toml_config_parsed` テストを実装した
  - `[sla]` セクションのパース → `max_latency_ms == 5000` / `min_throughput == 1000` / `max_error_rate ≈ 0.01` を assert
  - 必須フィールド欠落 → `Err` で `"missing"` を含むことを assert
- [x] `cargo test v735000` で 2 件 pass することを確認

---

## T6: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.4.0"` → `version = "73.5.0"` に変更した
- [x] `driver.rs` 内の `"73.4.0"` を `"73.5.0"` に replace_all した（`cargo_toml_version_is_*` テスト等のバージョン検証文字列を含む）
- [x] 残存 `73.4.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "73.5.0"` を含むことを確認

---

## T7: 部分テスト確認

- [x] T6 のバージョン更新後も `cargo test v735000` で引き続き 2 件 pass することを確認

---

## T8: 全体テスト確認

- [x] `cargo test` 全体で 3657 tests pass（0 failures）であることを確認

---

## T9: `CHANGELOG.md` 更新

- [x] `## [v73.5.0]` エントリを先頭に追加した
  - Added: `SlaConfig` / `SlaAlertConfig` / `parse_sla_config` / `check_sla` / `format_sla_alert`
  - Tests: 2 件、合計テスト数 3657（+2）

---

## T10: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.5.0)` に更新した
- [x] 「進行中バージョン」を `v73.5.0` に更新した
- [x] 「次に切る版」を `v73.6.0` に更新した

---

## T11: 最終確認（T9・T10 完了後）

- [x] `cargo test v735000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3657 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.5.0` であることを確認
- [x] `SlaConfig` / `SlaAlertConfig` / `parse_sla_config` / `check_sla` / `format_sla_alert` が pub で存在することを確認
- [x] `CHANGELOG.md` に `[v73.5.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.5.0` であることを確認

---

## コードレビュー指摘対応

実装時に発覚したビルドエラー:
- `SlaConfig` に `Debug` trait 未実装 → テストの `.expect()` が `Debug` を要求 → `#[derive(Debug)]` を追加

code-reviewer 指摘（修正済み）:

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [MED] | `strip_prefix` 前方一致 — 将来キー追加で誤パースリスク | `find('=')` + `match key` による完全一致パースに変更 |
| [MED] | 境界値テスト欠如（閾値ちょうど・1件のみ違反） | `boundary`（閾値ちょうどは違反なし）・`one`（latency のみ違反）テストケースを追加 |
| [MED] | `SlaAlertConfig` がデッドコード — Clippy 警告の恐れ | `#[allow(dead_code)]` + 将来実装コメントを追加 |
| [LOW] | `SlaAlertConfig` に `#[derive(Debug)]` がない | `#[derive(Debug)]` を追加 |
| [LOW] | `f64 >` 境界値の仕様確認 | 現仕様（上限以内は許容 = `>` のみ）が正しい — 対応不要 |

---

## スコープ外（明示的除外）

- `--enforce-sla` CLI フラグ（`main.rs` への組み込み）
- Slack / PagerDuty への実際の HTTP 通知
- `[sla.alerts]` セクションのパース（`SlaAlertConfig` は構造体のみ定義）
- 環境変数展開（`${PAGERDUTY_KEY}` 等）
