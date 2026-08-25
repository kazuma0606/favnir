# v82.4.0 — 契約違反の詳細レポート（`ContractViolation`）

Date: 2026-08-20
Status: 計画中

---

## Background

Pipeline Contracts 1.0 スプリントの第 4 版。
v82.1.0 の `ContractValidationResult` は `valid: bool` と `errors: Vec<String>` のみだったが、
違反ごとに「どのフィールドが・なぜ・どの行で」違反しているかを型で表現する。

`ContractViolation` はフィールド名・違反種別・行インデックスを保持し、
`violation_severity` で重大度（Error / Warning）を返す。
これにより CI でエラー件数を集計したり、ログに構造化レポートを出力したりできる。

---

## Goals

1. `ViolationKind` enum を定義する
   - `TypeMismatch { expected: String, actual: String }`: フィールドの型が期待と異なる
   - `MissingField(String)`: 必須フィールドが存在しない
   - `ExtraField(String)`: 契約に存在しない余分なフィールドがある
   - `NullNotAllowed(String)`: null 禁止フィールドに null が入った
2. `ContractViolation` 構造体を定義する（`field: String`, `kind: ViolationKind`, `row_index: Option<usize>`）
3. `ContractViolationReport` 構造体を定義する（`contract_name: String`, `violations: Vec<ContractViolation>`）
4. `format_violation_report(report: &ContractViolationReport) -> String` を実装する
5. `violation_severity(violation: &ContractViolation) -> RuleSeverity` を実装する
   - `TypeMismatch` → `Error`
   - `MissingField` → `Error`
   - `ExtraField` → `Warning`
   - `NullNotAllowed` → `Error`

---

## API Examples（Rust テストコード）

```rust
// TypeMismatch 違反
let v1 = ContractViolation {
    field: "id".into(),
    kind: ViolationKind::TypeMismatch {
        expected: "Int".into(),
        actual: "Str".into(),
    },
    row_index: Some(3),
};
assert_eq!(violation_severity(&v1), RuleSeverity::Error);

// MissingField 違反
let v2 = ContractViolation {
    field: "name".into(),
    kind: ViolationKind::MissingField("name".into()),
    row_index: None,
};
assert_eq!(violation_severity(&v2), RuleSeverity::Error);

// ExtraField 違反
let v3 = ContractViolation {
    field: "debug_flag".into(),
    kind: ViolationKind::ExtraField("debug_flag".into()),
    row_index: None,
};
assert_eq!(violation_severity(&v3), RuleSeverity::Warning);

// レポートフォーマット
let report = ContractViolationReport {
    contract_name: "orders_pipeline".into(),
    violations: vec![v1, v2],
};
let s = format_violation_report(&report);
assert!(s.contains("orders_pipeline"));
assert!(s.contains("TypeMismatch") || s.contains("type mismatch"));
assert!(s.contains("MissingField") || s.contains("missing"));
```

### `violation_severity` の判定ロジック

| `ViolationKind` | `RuleSeverity` |
|---|---|
| `TypeMismatch` | `Error` |
| `MissingField` | `Error` |
| `ExtraField` | `Warning` |
| `NullNotAllowed` | `Error` |

### `format_violation_report` の出力形式

```
orders_pipeline: 2 violation(s)
[Error] field 'id': type mismatch (expected=Int, actual=Str) at row 3
[Error] field 'name': missing required field
```

- ヘッダ: `"{contract_name}: {n} violation(s)"`
- 各違反: `"[{severity}] field '{field}': {description}"` + `" at row {n}"` (row_index がある場合)
- 違反がない場合: `"{contract_name}: no violations"`

---

## Success Criteria

- `cargo test` 全 pass（3,873 tests = 3,871 + 2）
- 新規テスト 2 件（`v82400_tests` モジュール）:
  - `violation_report_shows_type_mismatch`
  - `violation_report_shows_missing_field`

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/test_framework.rs` | `ViolationKind` / `ContractViolation` / `ContractViolationReport` / `format_violation_report` / `violation_severity` を追加 |
| `fav/src/driver.rs` | `#[cfg(test)] mod v82400_tests` を追加（テスト 2 件） |
| `CHANGELOG.md` | v82.4.0 エントリを先頭に追加 |
