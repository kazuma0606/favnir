# v77.6.0 タスクリスト — 証明付き CI 統合

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.5.0` であることを確認
- [x] `cargo test` が全 pass（3746 tests）であることを確認（v77.6.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.6.0: 証明付き CI 統合 ---` コメントを追加する
- [x] `CiVerificationConfig` 構造体を追加する（pipeline: String, fail_fast: bool, data_path: String、Debug / Clone / PartialEq 付き）
- [x] `CiResult` 構造体を追加する（passed: bool, report: VerificationReport, exit_code: i32、Debug / Clone / PartialEq 付き）
- [x] `run_ci_verification(config: &CiVerificationConfig, invariants: &[PipelineInvariant]) -> CiResult` を追加する
  - `cmd_verify(&config.pipeline, invariants)` で `VerificationReport` を生成
  - `passed = report.all_passed`
  - `exit_code = if passed { 0 } else { 1 }`
- [x] `format_ci_result_summary(result: &CiResult) -> String` を追加する
  - passed=true → `"[CI] ✓ All invariants passed. Exit code: {exit_code}"`
  - passed=false → `"[CI] ✗ Invariant violations detected. {failed}/{total} failed. Exit code: {exit_code}"`
- [x] `cargo test` で既存 3746 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.6.0 エントリを追加する
- [x] Added セクション（struct 2 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v776000_tests` モジュールを追加する（`use super::*` 必須: `run_ci_verification` / `CiResult` / `CiVerificationConfig` 等が outer scope にあるため）
- [x] `ci_verification_passes` テストを実装する
  - `CiVerificationConfig`（pipeline="OrderPipeline", fail_fast=false, data_path="data/sample.csv"）を用意
  - `PipelineInvariant` 2 件（row_count_reduced / amount_non_negative）を用意
  - `run_ci_verification(&config, &invariants)` → `result.passed == true`、`result.exit_code == 0`
  - `result.report.pipeline == "OrderPipeline"` を検証
  - `format_ci_result_summary(&result)` が `"[CI]"` と `"passed"` を含むことを検証
- [x] `ci_verification_fails_on_violation` テストを実装する
  - `VerificationReport`（all_passed=false、ok_inv=true + fail_inv=false）を直接構築
  - `CiResult { passed: false, report, exit_code: 1 }` を直接構築
  - `result.exit_code == 1` を検証
  - `format_ci_result_summary(&result)` が `"[CI]"`、`"violations"`、`"Exit code: 1"` を含むことを検証
- [x] `cargo test v776000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.5.0"` → `"77.6.0"` に変更する
- [x] `driver.rs` 内の `77.5.0` バージョン文字列アサーションを `77.6.0` に一括更新（`replace_all: true` で全件置換）
- [x] **replace_all 後に** `grep "v77.5.0" fav/src/driver.rs` を実行し、`// --- v77.5.0: fav verify コマンド ---` が残っていることを確認する（`v77.6.0` に書き換わっていたら手動で元に戻す）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.6.0 に更新する
- [x] 「次に切る版」を v77.7.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3748 tests）
- [x] `cargo test v776000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.6.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.6.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.6.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `ci_verification_passes` が pass
- [x] `ci_verification_fails_on_violation` が pass
- [x] テスト総数: 3748（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_6_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.6.0]` であること）で代替する
