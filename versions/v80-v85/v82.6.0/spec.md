# v82.6.0 — 契約バージョニング（`ContractVersion` / 後方互換チェック）

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 6 版。
`IoContract` はすでに `version: String` フィールドを持つが、
バージョン文字列をセマンティックに解析し、2 つの契約間の後方互換性を自動チェックする型を追加する。

`check_contract_compatibility` により「契約の更新が下流パイプラインを壊すか」を
コンパイル時ではなく CI ステップで検出できる。

---

## Goals

1. `ContractVersion` 構造体を定義する（`major: u32`, `minor: u32`, `patch: u32`）
2. `ContractVersion::parse(s: &str) -> Result<ContractVersion, String>` を実装する
   - `"1.2.3"` → `Ok(ContractVersion { major: 1, minor: 2, patch: 3 })`
   - フォーマット不正 → `Err("invalid version: ...")`
3. `CompatibilityResult` enum を定義する
   - `Compatible`: 変更なし（後方互換）
   - `BackwardsCompatible(Vec<String>)`: フィールドの追加（required/optional 問わず、削除・型変更なし）
   - `Breaking(Vec<String>)`: 必須フィールドの削除・型変更など（既存消費者が壊れる）
4. `check_contract_compatibility(old: &IoContract, new_: &IoContract) -> CompatibilityResult` を実装する
5. `format_compatibility_result(result: &CompatibilityResult) -> String` を実装する

---

## API Examples（Rust テストコード）

```rust
// ContractVersion::parse
let v = ContractVersion::parse("1.2.3").unwrap();
assert_eq!(v.major, 1);
assert_eq!(v.minor, 2);
assert_eq!(v.patch, 3);

let err = ContractVersion::parse("bad");
assert!(err.is_err());
let err2 = ContractVersion::parse("1.x.3"); // 数値変換失敗
assert!(err2.is_err());

// check_contract_compatibility: フィールド削除 → Breaking
let field_id = ContractField { name: "id".into(), field_type: ContractFieldType::Int, required: true };
let field_name = ContractField { name: "name".into(), field_type: ContractFieldType::Str, required: true };

let old = IoContract { name: "orders".into(), version: "1.0.0".into(),
    input: vec![field_id.clone(), field_name.clone()], output: vec![] };
let new_ = IoContract { name: "orders".into(), version: "2.0.0".into(),
    input: vec![field_id.clone()], output: vec![] };  // name を削除

let result = check_contract_compatibility(&old, &new_);
assert!(matches!(result, CompatibilityResult::Breaking(_)));

// format
let s = format_compatibility_result(&CompatibilityResult::Compatible);
assert_eq!(s, "Compatible");
```

### `check_contract_compatibility` の判定ロジック

以下の順序でチェックし、最初に該当したカテゴリを返す（優先度: Breaking > BackwardsCompatible > Compatible）。

1. **Breaking**: `old.input` にある required フィールドが `new_.input` に存在しない → `Breaking(削除されたフィールド名リスト)`
2. **Breaking**: `old.input` と `new_.input` で同名フィールドの `field_type` が異なる → `Breaking(型変更フィールド名リスト)`（required / optional 問わず全フィールドが対象）
3. **BackwardsCompatible**: `new_.input` に `old.input` にないフィールドが追加されている → `BackwardsCompatible(追加フィールド名リスト)`（required / optional 問わず追加フィールド全体が対象）
4. **Compatible**: 上記いずれも該当しない

複数カテゴリが同時に発生する場合は `Breaking` を優先して返す。

### `format_compatibility_result` の出力形式

| `CompatibilityResult` | 出力 |
|---|---|
| `Compatible` | `"Compatible"` |
| `BackwardsCompatible(fields)` | `"BackwardsCompatible: added [field1, field2]"` |
| `Breaking(fields)` | `"Breaking: [field1, field2]"` |

---

## Success Criteria

- `cargo test` 全 pass（3,877 tests = 3,875 + 2）
- 新規テスト 2 件（`v82600_tests` モジュール）:
  - `contract_version_parsed`
  - `breaking_change_detected_on_field_removal`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `ContractVersion` / `ContractVersion::parse` / `CompatibilityResult` / `check_contract_compatibility` / `format_compatibility_result` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82600_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.6.0 エントリを先頭に追加 |
