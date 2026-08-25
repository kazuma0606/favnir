# v82.7.0 実装計画

## 方針

**前提**: v82.6.0 完了済み（3,877 tests pass）。

`test_framework.rs` に `fav verify --contract` 統合型・関数を追加し、
`driver.rs` に `v82700_tests` を追加する。

依存型（すべて定義済み）:
- `ContractValidationResult` / `validate_io_contract` — v82.1.0
- `SlaStatus` / `SlaContract` / `evaluate_sla` / `format_sla_status` — v82.2.0
- `IoContract` / `ContractField` — v82.1.0

---

## 実装ステップ

### Step 1: `VerifyContractOptions` 構造体を追加

`fav/src/test_framework.rs` の v82.6.0 セクション末尾に続けて追加する。

```rust
// ── v82.7.0: VerifyContractOptions / ContractVerifyResult / cmd_verify_contract ──

/// `fav verify --contract` コマンドのオプション。
#[derive(Debug, Clone)]
pub struct VerifyContractOptions {
    pub contract_path: String,
    pub input_schema: Option<String>,
    pub strict: bool,
}
```

### Step 2: `ContractVerifyResult` 構造体を追加

```rust
/// `cmd_verify_contract` の戻り値。IoContract 検証結果と SLA 評価結果を保持する。
#[derive(Debug)]
pub struct ContractVerifyResult {
    pub io_result: ContractValidationResult,
    pub sla_result: Option<SlaStatus>,
}
```

### Step 3: `cmd_verify_contract` 関数を実装

```rust
/// `fav verify --contract` のコアロジック。
///
/// - `validate_io_contract` で IoContract を検証して `io_result` を取得する
/// - `sla_check` が Some なら `evaluate_sla` を呼んで `sla_result` を設定する
/// - `options.strict` は将来拡張用（本バージョンでは参照しない）
pub fn cmd_verify_contract(
    _options: &VerifyContractOptions,
    contract: &IoContract,
    actual_input: &[ContractField],
    sla_check: Option<(&SlaContract, u64, f64)>,
) -> ContractVerifyResult {
    let io_result = validate_io_contract(contract, actual_input);
    let sla_result = sla_check.map(|(sla, lat, rps)| evaluate_sla(sla, lat, rps));
    ContractVerifyResult { io_result, sla_result }
}
```

### Step 4: `format_verify_result` 関数を実装

```rust
/// `ContractVerifyResult` を人間が読める文字列に変換する。
///
/// - `io_result.valid == true` → `"Contract: PASS"`
/// - `io_result.valid == false` → `"Contract: FAIL ({n} error(s))"`
/// - `sla_result` が Some なら次行に `format_sla_status` の結果を追加する
pub fn format_verify_result(result: &ContractVerifyResult) -> String {
    let io_line = if result.io_result.valid {
        "Contract: PASS".to_string()
    } else {
        format!("Contract: FAIL ({} error(s))", result.io_result.errors.len())
    };
    match &result.sla_result {
        Some(sla_status) => {
            format!("{}\n{}", io_line, format_sla_status(sla_status))
        }
        None => io_line,
    }
}
```

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.7.0 エントリを追加する。

### Step 6: `v82700_tests` テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82700_tests` を追加する。

- `verify_contract_cmd_passes_valid_contract`:
  - 全必須フィールドが存在する actual_input で `cmd_verify_contract` を呼ぶ
  - `io_result.valid == true`・`format_verify_result` に `"PASS"` が含まれることを確認
- `verify_contract_cmd_fails_breaking_change`:
  - 必須フィールドを欠損した actual_input で `cmd_verify_contract` を呼ぶ（破壊的変更のシミュレーション）
  - `io_result.valid == false`・`format_verify_result` に `"FAIL"` が含まれることを確認
  - `check_contract_compatibility` でも同じ変更が Breaking と判定されることを確認（統合テスト）

### Step 7: `cargo test` 全通過確認

3,879 tests pass（+2）、0 failures であることを確認する。
