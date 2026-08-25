# v83.6.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,897 tests pass、0 failures であることを確認する（前提: v83.5.0 完了済み）

## T1: `test_framework.rs` に構造体と impl を追加

- [x] `PerfBaseline` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `pipeline_name: String`, `p50_ms: u64`, `p95_ms: u64`, `p99_ms: u64`
- [x] `impl PerfBaseline` — `from_samples(pipeline_name: &str, samples_ms: &[u64]) -> PerfBaseline` を追加する
  - 空配列のとき p50/p95/p99 = 0
  - `sort_unstable` でソートし、インデックスベース（`n * pct / 100`）で百分位数を算出
- [x] `PerfRegression` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]`）
  - `pipeline_name: String`, `baseline: PerfBaseline`, `current_ms: u64`, `regression_pct: f64`

## T2: `detect_perf_regression` / `format_regression_report` 関数を追加

- [x] `detect_perf_regression(baseline: &PerfBaseline, current_ms: u64, threshold_pct: f64) -> Option<PerfRegression>` を追加する
  - `p95_ms == 0` のとき `None`（ゼロ除算ガード）
  - `regression_pct = (current_ms - p95_ms) / p95_ms * 100.0`
  - `regression_pct > threshold_pct` のとき `Some`
- [x] `format_regression_report(regression: &PerfRegression) -> String` を追加する
  - "PerfRegression:"、"Baseline p95:"、"Current:"、"Regression:" の各行を含む
  - "Regression:" 行は `+{:.2}%` 形式（小数点以下 2 桁）で出力すること

## T3: `driver.rs` に `v83600_tests` を追加

- [x] `v83500_tests` の直後に `#[cfg(test)] mod v83600_tests` を追加する
  - `perf_regression_detected_above_threshold`
  - `perf_no_regression_within_threshold`

## T4: `CHANGELOG.md` 更新

- [x] `CHANGELOG.md` の先頭に v83.6.0 エントリを追加する

## T5: テスト通過確認

- [x] `cargo test` が 3,899 tests pass（+2）、0 failures であることを確認する

## T6: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [HIGH] `format_regression_report` の `regression_pct` が負のとき `+-XX%` になる問題: sign 変数で `+`/`""` を切り替えるよう修正
- [MED] `detect_perf_regression` の境界条件（`regression_pct == threshold_pct` → `None`）を doc コメントに「厳密に超えた場合」と明記
- [LOW] `from_samples` の戻り値型を `PerfBaseline` → `Self` に変更
- [HIGH] パーセンタイルインデックス計算式 `n * pct / 100`: spec.md / plan.md に明記された仕様のため対応不要
- [MED] `p50_ms`/`p99_ms` 未使用フィールド: `pub` フィールドのため Clippy 警告なし、将来ステップで使用予定のため対応不要
