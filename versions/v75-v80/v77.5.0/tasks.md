# v77.5.0 タスクリスト — `fav verify` コマンド

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `77.4.0` であることを確認
- [x] `cargo test` が全 pass（3744 tests）であることを確認（v77.5.0 テスト追加前の状態）
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v77.5.0: fav verify コマンド ---` コメントを追加する
- [x] `InvariantResult` 構造体を追加する（name: String, passed: bool, detail: String、Debug / Clone 付き）
- [x] `VerificationReport` 構造体を追加する（pipeline: String, results: Vec<InvariantResult>, all_passed: bool、Debug / Clone 付き）
- [x] `cmd_verify(pipeline_name: &str, invariants: &[PipelineInvariant]) -> VerificationReport` を追加する
  - 各 PipelineInvariant から `InvariantResult { passed: true, detail: "invariant '...' declared for ..." }` を生成
  - `all_passed = results.iter().all(|r| r.passed)`
- [x] `format_verification_report(report: &VerificationReport) -> String` を追加する
  - `"Verifying {pipeline}...\n"` で始まる
  - 各結果: passed → `"  ✓ {name} ({detail})\n"`, failed → `"  ✗ {name} ({detail})\n"`
  - all_passed=true → 末尾 `"Verification passed. N/N invariants checked."`
  - all_passed=false → 末尾 `"Verification FAILED. M of N invariants violated."`
- [x] `cargo test` で既存 3744 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v77.5.0 エントリを追加する
- [x] Added セクション（struct 2 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v775000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `verify_cmd_all_pass` テストを実装する
  - PipelineInvariant 2 件（filter_reduces_rows / total_amount_non_negative）を用意
  - `cmd_verify("OrderPipeline", &invariants)` → `report.all_passed == true`
  - `report.pipeline == "OrderPipeline"`, `report.results.len() == 2` を検証
  - `format_verification_report(&report)` が `"Verifying OrderPipeline"` と `"passed"` を含むことを検証
- [x] `verify_cmd_violation_reported` テストを実装する
  - `VerificationReport` を直接構築（passed=true の ok_inv + passed=false の fail_inv）
  - `report.all_passed == false` を検証
  - `format_verification_report(&report)` が `"FAILED"` と `"fail_inv"` を含むことを検証
- [x] `cargo test v775000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"77.4.0"` → `"77.5.0"` に変更する
- [x] `driver.rs` 内の `77.4.0` バージョン文字列アサーションを `77.5.0` に一括更新（`replace_all: true` で全件置換）
- [x] **replace_all 後に** `grep "// ---"` でセクションコメントを確認し、誤書き換えがあれば手動修正する

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v77.5.0 に更新する
- [x] 「次に切る版」を v77.6.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3746 tests）
- [x] `cargo test v775000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `77.5.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v77.5.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v77.5.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `verify_cmd_all_pass` が pass
- [x] `verify_cmd_violation_reported` が pass
- [x] テスト総数: 3746（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v77_5_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。T6 の手動確認（CHANGELOG.md 先頭が `[v77.5.0]` であること）で代替する
