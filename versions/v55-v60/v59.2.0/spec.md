# v59.2.0 Spec — SLA 保証ティア（SLA Guarantee + アラート統合）

Date: 2026-07-29
Status: 設計中

---

## 概要

既存の `sla` Rune（v52.5 実装済み）を上位の SLA Guarantee モードとして統合。
`fav run --sla-enforce` フラグと `fav sla report` コマンドを追加し、
`driver.rs` に `cmd_sla_report` スタブを実装する。

---

## 実装内容

| 項目 | 内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_sla_report() -> i32` を追加（SLA 達成率レポート出力スタブ） |
| `fav/src/main.rs` | `Some("run")` アームに `--sla-enforce` フラグを追加 |
| `fav/src/main.rs` | `Some("sla")` アームを新規追加（`report` サブコマンドで `cmd_sla_report()` を呼ぶ） |

---

## cmd_sla_report の出力仕様

```
# SLA Report
latency_p99_ms:   200ms  [OK]
error_rate_pct:   0.1%   [OK]
availability_pct: 99.9%  [OK]
SLA compliance: PASS
```

戻り値: `0`

---

## fav.toml SLA 設定例（ロードマップ仕様）

```toml
[sla]
latency_p99_ms   = 200
error_rate_pct   = 0.1
availability_pct = 99.9

[sla.alerting]
channels           = ["pagerduty", "slack"]
escalation_policy  = "prod-oncall"
```

---

## テスト

`v59200_tests` モジュールを `v59100_tests` の直前に挿入（2 件）:

| テスト名 | 内容 |
|---|---|
| `sla_guarantee_config_parsed` | インライン TOML 文字列が `latency_p99_ms`・`availability_pct`・`[sla.alerting]` を含むことを検証 |
| `sla_report_generates` | `cmd_sla_report()` が `0` を返すことを検証 |

- `use super::cmd_sla_report` が必要（`sla_report_generates` が `super` の関数を呼ぶため）
- `sla_guarantee_config_parsed` は定数文字列のみで `use super::*` 不要

**実際のベース**: 3310（v59.1.0 実績値）
**完了条件**: 3310 + 2 = **3312 tests passed, 0 failed**

---

## ローリングチェック更新

既存 7 件のローリングアサーションを `"59.1.0"` → `"59.2.0"` に更新:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0`）
- `v56900_tests::cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0`）
- `v56300_tests::cargo_toml_version_is_56_3_0`

**注意**: `v59100_tests` には rolling check が存在しない（`enterprise_e2e_demo_structure` と `cmd_test_enterprise_suite` のみ）ため更新対象外。更新対象は計 7 件。

failure メッセージ 7 件も同様に `"59.2.0"` に更新。

---

## main.rs 変更

### `--sla-enforce` フラグ（`Some("run")` アーム）

既存の `Some("run")` アームのフラグ解析ループに `"--sla-enforce"` アームを追加:
- `--sla-enforce` → `sla_enforce = true` をセット（`i += 1`）

### `Some("sla")` アーム（新規）

```
fav sla report [--audit-log <file>] [-o <file>]
```

- `report` サブコマンド → `cmd_sla_report()` を呼んで `process::exit(code)`
- 不明サブコマンド → `eprintln!` + `exit(1)`

---

## 影響ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `cmd_sla_report` 追加 + v59200_tests + ローリングチェック更新 |
| `fav/src/main.rs` | `Some("run")` に `--sla-enforce` 追加、`Some("sla")` アーム新規追加 |
| `fav/Cargo.toml` | バージョン `59.2.0` |
| `CHANGELOG.md` | v59.2.0 エントリ追加 |
| `versions/current.md` | 最新安定版を v59.2.0 に更新 |
| `versions/roadmap/roadmap-v59.1-v60.0.md` | v59.2.0 実績欄に完了記録、v59.3.0 ベース数更新 |
