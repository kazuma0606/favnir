# v82.4.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,871 tests pass、0 failures であることを確認する（前提: v82.3.0 完了済み）

## T1: `ViolationKind` enum 追加

- [x] `fav/src/test_framework.rs` に `ViolationKind` enum を追加する
  - variants: `TypeMismatch { expected: String, actual: String }` / `MissingField(String)` / `ExtraField(String)` / `NullNotAllowed(String)`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T2: `ContractViolation` 構造体追加

- [x] `fav/src/test_framework.rs` に `ContractViolation` 構造体を追加する
  - `field: String` / `kind: ViolationKind` / `row_index: Option<usize>`
  - `#[derive(Debug, Clone, PartialEq)]` を付与する

## T3: `ContractViolationReport` 構造体追加

- [x] `fav/src/test_framework.rs` に `ContractViolationReport` 構造体を追加する
  - `contract_name: String` / `violations: Vec<ContractViolation>`
  - `#[derive(Debug, Clone)]` を付与する

## T4: `violation_severity` 関数追加

- [x] `violation_severity(violation: &ContractViolation) -> RuleSeverity` を実装する
  - `TypeMismatch` / `MissingField` / `NullNotAllowed` → `RuleSeverity::Error`
  - `ExtraField` → `RuleSeverity::Warning`

## T5: `format_violation_report` 関数追加

- [x] `format_violation_report(report: &ContractViolationReport) -> String` を実装する
  - 違反なし → `"{contract_name}: no violations"`
  - 違反あり → ヘッダ + 各違反を `"[{severity}] field '{field}': {description}"` 形式で改行結合

## T6: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.4.0 エントリを追加する

## T7: `v82400_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82400_tests` を追加する
  - `violation_report_shows_type_mismatch`: TypeMismatch 違反 → severity=Error、フォーマットに "type mismatch" が含まれることを確認
  - `violation_report_shows_missing_field`: MissingField 違反 → severity=Error、フォーマットに "missing" が含まれることを確認

## T8: テスト通過確認

- [x] `cargo test` が 3,873 tests pass（+2）、0 failures であることを確認する

## T9: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
