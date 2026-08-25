# Tasks: v81.8.0 — 異常検知（`AnomalyDetector` / Z スコアベース）

> COMPLETE — 2026-08-19
> 3859 tests, 0 failures（+2 from 3857）

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3857 tests, 0 failures を確認する
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "81.0.0"` であることを確認する
  （v81.x マイナーバージョンは Cargo.toml を更新しない慣例。このバージョン完了後も `81.0.0` のまま変更しない）
- [x] `fav/src/driver.rs` に `mod v81700_tests` が存在することを確認する（v81.7.0 完了済みの証拠）
- [x] `fav/src/test_framework.rs` に `DistributionStats` / `compute_distribution_stats` が定義済みであることを確認する（v81.2.0 依存）

## T1: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.8.0 エントリを追加する

## T2: `fav/src/test_framework.rs` に追記

- [x] `AnomalyDetector` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `baseline_stats: DistributionStats`, `z_threshold: f64`
- [x] `AnomalyResult` 構造体（`#[derive(Debug, Clone)]`）を追加する
  - フィールド: `is_anomaly: bool`, `z_score: f64`, `value: f64`
- [x] `impl AnomalyDetector` ブロックを追加する
  - `from_baseline(values: &[f64], z_threshold: f64) -> AnomalyDetector`: `compute_distribution_stats` を呼び出す
  - 空スライスを渡してもパニックしないことを確認する（`compute_distribution_stats` への委譲で担保）
- [x] `detect_anomaly(detector: &AnomalyDetector, value: f64) -> AnomalyResult` を実装する
  - Z スコア: `|value - mean| / std_dev`
  - `std_dev == 0.0` のとき `z_score = 0.0`, `is_anomaly = false`（ゼロ除算ガード）
  - `is_anomaly = z_score > z_threshold`
- [x] `scan_for_anomalies(detector: &AnomalyDetector, values: &[f64]) -> Vec<AnomalyResult>` を実装する
  - `values.iter().map(|&v| detect_anomaly(detector, v)).collect()`
- [x] `format_anomaly_report(results: &[AnomalyResult]) -> String` を実装する
  - フォーマット: `"anomaly_report total={n} anomalies={k}"`

## T3: `fav/src/driver.rs` に `mod v81800_tests` を追加

- [x] `mod v81700_tests { ... }` の直後に `#[cfg(test)] mod v81800_tests { ... }` を追加する
- [x] `use fav_core::test_framework::*;` でインポートする
- [x] `anomaly_detector_catches_outlier` テストを実装する
  - ベースライン [1,2,3,2,2]、z_threshold=2.0 で外れ値 10.0 → `is_anomaly=true`、`z_score > 2.0` を確認する
  - 正常値 2.0 → `is_anomaly=false` を確認する
  - 全同値ベースライン（std_dev=0）での `z_score=0.0`, `is_anomaly=false` を確認する
- [x] `anomaly_scan_returns_all_results` テストを実装する
  - `scan_for_anomalies` が全要素の結果を返すことを確認する（`results.len() == 3`）
  - 外れ値のみ `is_anomaly=true` になることを確認する
  - `format_anomaly_report` の出力に `"total=3"` / `"anomalies=1"` が含まれることを確認する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3859 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
