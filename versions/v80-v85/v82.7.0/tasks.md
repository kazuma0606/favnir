# v82.7.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,877 tests pass、0 failures であることを確認する（前提: v82.6.0 完了済み）

## T1: `VerifyContractOptions` 構造体追加

- [x] `fav/src/test_framework.rs` に `VerifyContractOptions` 構造体を追加する
  - `contract_path: String` / `input_schema: Option<String>` / `strict: bool`
  - `#[derive(Debug, Clone)]` を付与する

## T2: `ContractVerifyResult` 構造体追加

- [x] `fav/src/test_framework.rs` に `ContractVerifyResult` 構造体を追加する
  - `io_result: ContractValidationResult` / `sla_result: Option<SlaStatus>`
  - `#[derive(Debug)]` を付与する

## T3: `cmd_verify_contract` 関数追加

- [x] `cmd_verify_contract(options, contract, actual_input, sla_check) -> ContractVerifyResult` を実装する
  - `validate_io_contract` で io_result を取得する
  - `sla_check` が Some なら `evaluate_sla` で sla_result を設定する

## T4: `format_verify_result` 関数追加

- [x] `format_verify_result(result: &ContractVerifyResult) -> String` を実装する
  - `io_result.valid == true` → `"Contract: PASS"`
  - `io_result.valid == false` → `"Contract: FAIL ({n} error(s))"`
  - `sla_result` が Some なら次行に `format_sla_status` の結果を追加する

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v82.7.0 エントリを追加する

## T6: `v82700_tests` テストモジュール追加

- [x] `fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82700_tests` を追加する
  - `verify_contract_cmd_passes_valid_contract`: 全フィールド存在で PASS・`format_verify_result` に "PASS" が含まれることを確認
  - `verify_contract_cmd_fails_breaking_change`: 必須フィールド欠損で FAIL・`check_contract_compatibility` でも Breaking と判定されることを確認（統合テスト）

## T7: テスト通過確認

- [x] `cargo test` が 3,879 tests pass（+2）、0 failures であることを確認する

## T8: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## code-reviewer 対応

- [x] [MED] SLA `Some` ケース（Met / Breached）のテストを `verify_contract_cmd_passes_valid_contract` に追加
- [x] [LOW] `ContractVerifyResult` と `ContractValidationResult` に `#[derive(PartialEq)]` を追加
