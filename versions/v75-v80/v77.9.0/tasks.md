# v77.9.0 タスクリスト — 安定化・コードフリーズ

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.8.0` であることを確認
- [x] `cargo test` が全 pass（3754 tests）であることを確認（v77.9.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.9.0 エントリを追加する（形式: `## [v77.9.0] — 2026-08-16 — 安定化・コードフリーズ`）
- [x] Tests セクション（2 件）を含める（型追加なしのため Added セクションは不要）

---

## T2: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` の末尾に `v779000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `verifiable_full_sprint_all_stable` テストを実装する
  - `InvariantViolation` を instantiate して `column` フィールドを検証
  - `check_aggregate_invariant(&[1.0, 2.0], &AggregateInvariant { column: "amount", property: NonNegative })` → `is_ok()` を検証
  - `check_filter_invariant(5.0, &FilterInvariant { ..., predicate: GreaterThan(0.0) })` → `is_ok()` を検証
  - `check_join_invariant(true, true, &JoinInvariant { ..., join_type: Inner, null_policy: RejectNull })` → `is_ok()` を検証
  - `cmd_verify("test_pipeline", &[PipelineInvariant { ... }])` → `all_passed` を検証
  - `run_ci_verification(&CiVerificationConfig { ... }, &[])` → `exit_code == 0` を検証
  - `generate_counter_example_values(&agg, 1)` → `!example.is_empty()` を検証
  - `check_probabilistic_invariant(&[0.9, 0.95, 0.92], 0.8, 1.0, &ProbabilisticContract { ... })` → `is_ok()` を検証
- [x] `verifiable_e2e_pipeline_verified` テストを実装する
  - `check_aggregate_invariant` + `check_filter_invariant` + `check_probabilistic_invariant` を順次呼び出し、各 `is_ok()` を検証
  - `cmd_verify` + `run_ci_verification` を呼び出し、`exit_code == 0` と `format_ci_result_summary` に "passed" が含まれることを検証
- [x] `cargo test v779000` で 2 件が pass することを確認する

---

## T3: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.8.0"` → `"77.9.0"` に変更する
- [x] `driver.rs` 内の `77.8.0` バージョン文字列アサーションを `77.9.0` に一括更新（`replace_all: true` で全件置換）
- [x] **replace_all 後に** `grep "v77.8.0" fav/src/driver.rs` を実行し、`// --- v77.8.0: 確率的契約 ---` が残っていることを確認する（`v77.9.0` に書き換わっていた場合は手動で `v77.8.0` に戻す）
- [x] `grep "v77.8.0" fav/src/driver.rs` で `check_probabilistic_invariant` の doc コメント内 `v77.8.0` 記述が維持されていることを確認する（書き換わっていた場合は `v77.8.0` に戻す）

---

## T4: versions/current.md 更新

- [x] 「進行中バージョン」を v77.9.0 に更新する
- [x] 「次に切る版」を v78.0.0 に更新する

---

## T5: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3756 tests）
- [x] `cargo test v779000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.9.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.9.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.9.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T5）が完了している
- [x] `verifiable_full_sprint_all_stable` が pass
- [x] `verifiable_e2e_pipeline_verified` が pass
- [x] テスト総数: 3756（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（安定化スプリント）
- [x] `changelog_has_v77_9_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T5 の手動確認（CHANGELOG.md 先頭が `[v77.9.0]` であること）で代替する
