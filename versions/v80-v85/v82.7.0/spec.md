# v82.7.0 — `fav verify --contract` コマンド強化

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 7 版。
v82.1.0〜v82.6.0 で構築した型基盤（`IoContract` / `SlaContract` / `check_contract_compatibility`）を
`fav verify --contract` コマンドに統合する。

`cmd_verify_contract` は入力オプションに従い IoContract 検証・SLA 評価を行い、
`ContractVerifyResult` にまとめて返す。

---

## Goals

1. `VerifyContractOptions` 構造体を定義する（`contract_path: String`, `input_schema: Option<String>`, `strict: bool`）
2. `ContractVerifyResult` 構造体を定義する（`io_result: ContractValidationResult`, `sla_result: Option<SlaStatus>`）
3. `cmd_verify_contract(options, contract, actual_input, sla_check) -> ContractVerifyResult` を実装する
   - `validate_io_contract` を呼んで `io_result` を取得する
   - `sla_check: Option<(&SlaContract, u64, f64)>` が Some なら `evaluate_sla` を呼んで `sla_result` を設定する
   - `options.contract_path` / `options.strict` は将来拡張のための予約フィールドであり、本バージョンでは参照しない（`_options` として受け取る）
4. `format_verify_result(result: &ContractVerifyResult) -> String` を実装する

---

## API Examples（Rust テストコード）

```rust
let options = VerifyContractOptions {
    contract_path: "contracts/orders.toml".into(),
    input_schema: None,
    strict: false,
};

let field_id = ContractField { name: "id".into(), field_type: ContractFieldType::Int, required: true };
let contract = IoContract {
    name: "orders".into(), version: "1.0.0".into(),
    input: vec![field_id.clone()], output: vec![],
};

// Pass: 全必須フィールドが存在
let result = cmd_verify_contract(&options, &contract, &[field_id.clone()], None);
assert!(result.io_result.valid);
let s = format_verify_result(&result);
assert!(s.contains("PASS"));

// Fail: 必須フィールド欠損（= 破壊的変更）
let result2 = cmd_verify_contract(&options, &contract, &[], None);
assert!(!result2.io_result.valid);
let s2 = format_verify_result(&result2);
assert!(s2.contains("FAIL"));
```

### `cmd_verify_contract` の引数

| 引数 | 型 | 説明 |
|---|---|---|
| `options` | `&VerifyContractOptions` | コマンドオプション（現バージョンでは `contract_path` のみ参照） |
| `contract` | `&IoContract` | 検証対象の契約 |
| `actual_input` | `&[ContractField]` | 実際の入力フィールド |
| `sla_check` | `Option<(&SlaContract, u64, f64)>` | SLA チェック用 (contract, latency_ms, rps)。None なら sla_result は None |

### `format_verify_result` の出力形式

```
Contract: PASS
```

```
Contract: FAIL (2 error(s))
SLA: Breached — latency exceeded: 250 ms > 200 ms
```

- `io_result.valid == true` → `"Contract: PASS"`
- `io_result.valid == false` → `"Contract: FAIL ({n} error(s))"`
- `sla_result` が Some なら次行に `format_sla_status` の結果を追加する

---

## Success Criteria

- `cargo test` 全 pass（3,879 tests = 3,877 + 2）
- 新規テスト 2 件（`v82700_tests` モジュール）:
  - `verify_contract_cmd_passes_valid_contract`
  - `verify_contract_cmd_fails_breaking_change`（必須フィールド欠損で FAIL を確認し、さらに `check_contract_compatibility` でも同じ変更が `Breaking` と判定されることを確認する統合テスト）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `VerifyContractOptions` / `ContractVerifyResult` / `cmd_verify_contract` / `format_verify_result` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82700_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.7.0 エントリを先頭に追加 |
