# v82.4.0 実装計画

## 方針

**前提**: v82.3.0 完了済み（3,871 tests pass）。

`test_framework.rs` に契約違反レポート型・関数を追加し、`driver.rs` に `v82400_tests` を追加する。
`RuleSeverity` は v81.1.0 で定義済み（`test_framework.rs` の `pub enum RuleSeverity { Error, Warning }`）。

---

## 実装ステップ

### Step 1: `ViolationKind` enum を追加

`fav/src/test_framework.rs` の v82.3.0 セクション末尾に続けて追加する。

```rust
// ── v82.4.0: ContractViolation / ContractViolationReport ─────────────────────

/// 契約違反の種別。
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationKind {
    /// フィールドの型が期待と異なる。
    TypeMismatch { expected: String, actual: String },
    /// 必須フィールドが存在しない。
    MissingField(String),
    /// 契約に定義されていない余分なフィールドがある。
    ExtraField(String),
    /// null 禁止フィールドに null が入った。
    NullNotAllowed(String),
}
```

### Step 2: `ContractViolation` 構造体を追加

```rust
/// 単一フィールドの契約違反。
#[derive(Debug, Clone, PartialEq)]
pub struct ContractViolation {
    pub field: String,
    pub kind: ViolationKind,
    pub row_index: Option<usize>,
}
```

### Step 3: `ContractViolationReport` 構造体を追加

```rust
/// 契約検証の違反レポート。
#[derive(Debug, Clone)]
pub struct ContractViolationReport {
    pub contract_name: String,
    pub violations: Vec<ContractViolation>,
}
```

### Step 4: `violation_severity` 関数を実装

```rust
/// 違反の重大度を返す。
///
/// - `TypeMismatch` / `MissingField` / `NullNotAllowed` → `Error`
/// - `ExtraField` → `Warning`
pub fn violation_severity(violation: &ContractViolation) -> RuleSeverity {
    match &violation.kind {
        ViolationKind::TypeMismatch { .. } => RuleSeverity::Error,
        ViolationKind::MissingField(_) => RuleSeverity::Error,
        ViolationKind::NullNotAllowed(_) => RuleSeverity::Error,
        ViolationKind::ExtraField(_) => RuleSeverity::Warning,
    }
}
```

### Step 5: `format_violation_report` 関数を実装

```rust
/// 契約違反レポートを人間が読める文字列に変換する。
///
/// 出力形式:
/// ```
/// orders_pipeline: 2 violation(s)
/// [Error] field 'id': type mismatch (expected=Int, actual=Str) at row 3
/// [Error] field 'name': missing required field
/// ```
pub fn format_violation_report(report: &ContractViolationReport) -> String {
    if report.violations.is_empty() {
        return format!("{}: no violations", report.contract_name);
    }
    let mut lines = vec![format!(
        "{}: {} violation(s)",
        report.contract_name,
        report.violations.len()
    )];
    for v in &report.violations {
        let severity = violation_severity(v);
        let sev_str = match severity {
            RuleSeverity::Error => "Error",
            RuleSeverity::Warning => "Warning",
        };
        let kind_str = match &v.kind {
            ViolationKind::TypeMismatch { expected, actual } => {
                format!("type mismatch (expected={expected}, actual={actual})")
            }
            ViolationKind::MissingField(_) => "missing required field".into(),
            ViolationKind::ExtraField(_) => "extra field not in contract".into(),
            ViolationKind::NullNotAllowed(_) => "null not allowed".into(),
        };
        let row_str = v.row_index.map_or(String::new(), |r| format!(" at row {r}"));
        lines.push(format!("[{sev_str}] field '{}': {kind_str}{row_str}", v.field));
    }
    lines.join("\n")
}
```

### Step 6: CHANGELOG 更新

`CHANGELOG.md` の先頭に v82.4.0 エントリを追加する。

### Step 7: `v82400_tests` テストモジュール追加（driver.rs）

`fav/src/driver.rs` 末尾に `#[cfg(test)] mod v82400_tests` を追加する。

- `violation_report_shows_type_mismatch`:
  - `TypeMismatch` 違反を作成し `violation_severity` が `Error` を返すことを確認
  - `format_violation_report` の出力に `"TypeMismatch"` または `"type mismatch"` が含まれることを確認
- `violation_report_shows_missing_field`:
  - `MissingField` 違反を作成し `violation_severity` が `Error` を返すことを確認
  - `format_violation_report` の出力に `"missing"` が含まれることを確認

### Step 8: `cargo test` 全通過確認

3,873 tests pass（+2）、0 failures であることを確認する。
