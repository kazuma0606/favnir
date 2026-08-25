# Plan: v81.1.0 — `QualityRule` / `QualityCheck` 型基盤

## Step 1: 前提確認

- `cargo test` を実行し、3841 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v80.8.0 の `TestReport` が定義済みであることを確認する

## Step 2: `fav/src/test_framework.rs` に追記

`TestReport` / `format_test_summary` の定義の直後に以下を追加する。

```rust
// ── v81.1.0: QualityRule / QualityCheck 型基盤 ──────────────────────────────

/// 品質ルールの種類。
#[derive(Debug, Clone)]
pub enum QualityRuleKind {
    NotNull,
    Unique,
    Range { min: f64, max: f64 },
    Regex(String),
    Custom(String),
}

/// ルール違反時の重大度。
#[derive(Debug, Clone)]
pub enum RuleSeverity {
    Error,
    Warning,
}

/// 単一カラムへの品質ルール定義。
#[derive(Debug, Clone)]
pub struct QualityRule {
    pub column: String,
    pub kind: QualityRuleKind,
    pub severity: RuleSeverity,
}

/// 複数の品質ルールをまとめるチェックセット。
#[derive(Debug)]
pub struct QualityCheck {
    pub rules: Vec<QualityRule>,
}

/// 品質ルール違反の詳細。
#[derive(Debug)]
pub struct QualityViolation {
    pub rule: QualityRule,
    pub row_index: usize,
    pub actual: String,
}

/// `QualityCheck` のルールを全行に対して適用し、違反一覧を返す。
///
/// `column` フィールドはカラムのインデックス文字列（"0", "1", ...）として解釈する。
/// インデックスが行の範囲外の場合はその行をスキップする。
///
/// `QualityRuleKind` 適用ルール:
/// - `NotNull`: `value.trim().is_empty()` なら違反
/// - `Range { min, max }`: `f64` にパースして `v < min || v > max` なら違反（パース失敗はスキップ）
/// - `Regex(pattern)`: `!value.contains(pattern)` なら違反
/// - `Unique` / `Custom`: 行単位チェック非対応のためスキップ
pub fn run_quality_check(check: &QualityCheck, rows: &[Vec<String>]) -> Vec<QualityViolation> {
    let mut violations = Vec::new();
    for rule in &check.rules {
        let col_idx: usize = match rule.column.parse() {
            Ok(i) => i,
            Err(_) => continue,
        };
        for (row_idx, row) in rows.iter().enumerate() {
            let value = match row.get(col_idx) {
                Some(v) => v,
                None => continue,
            };
            let violated = match &rule.kind {
                QualityRuleKind::NotNull => value.trim().is_empty(),
                QualityRuleKind::Range { min, max } => {
                    match value.parse::<f64>() {
                        Ok(v) => v < *min || v > *max,
                        Err(_) => false,
                    }
                }
                QualityRuleKind::Regex(pattern) => !value.contains(pattern.as_str()),
                QualityRuleKind::Unique | QualityRuleKind::Custom(_) => false,
            };
            if violated {
                violations.push(QualityViolation {
                    rule: rule.clone(),
                    row_index: row_idx,
                    actual: value.clone(),
                });
            }
        }
    }
    violations
}
```

## Step 3: `fav/src/driver.rs` に `mod v81100_tests` を追加

`mod v81000_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81100_tests {
    use fav_core::test_framework::*;

    #[test]
    fn quality_rule_not_null_catches_violation() {
        let check = QualityCheck {
            rules: vec![QualityRule {
                column: "0".to_string(),
                kind: QualityRuleKind::NotNull,
                severity: RuleSeverity::Error,
            }],
        };
        let rows = vec![
            vec!["".to_string()],       // 行0: 空 → 違反
            vec!["value".to_string()],  // 行1: 違反なし
        ];
        let violations = run_quality_check(&check, &rows);
        assert_eq!(violations.len(), 1, "should catch exactly 1 violation");
        assert_eq!(violations[0].row_index, 0);
        assert_eq!(violations[0].actual, "");
    }

    #[test]
    fn quality_check_returns_all_violations() {
        let check = QualityCheck {
            rules: vec![
                QualityRule {
                    column: "0".to_string(),
                    kind: QualityRuleKind::NotNull,
                    severity: RuleSeverity::Error,
                },
                QualityRule {
                    column: "1".to_string(),
                    kind: QualityRuleKind::Range { min: 0.0, max: 100.0 },
                    severity: RuleSeverity::Warning,
                },
            ],
        };
        let rows = vec![
            vec!["".to_string(),    "50".to_string()],   // 行0: col0 違反
            vec!["ok".to_string(), "150".to_string()],   // 行1: col1 違反
            vec!["ok".to_string(),  "50".to_string()],   // 行2: 違反なし
        ];
        let violations = run_quality_check(&check, &rows);
        assert_eq!(violations.len(), 2, "should have 2 violations total");
        assert_eq!(violations[0].row_index, 0);
        assert_eq!(violations[1].row_index, 1);
        assert_eq!(violations[1].actual, "150");
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3843 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.1.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
