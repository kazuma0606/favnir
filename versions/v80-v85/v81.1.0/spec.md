# Spec: v81.1.0 — `QualityRule` / `QualityCheck` 型基盤

## Background

v81.0.0 で Test-Driven Data 1.0 を宣言した。
本バージョンからは **Data Quality 2.0 スプリント**（v81.1〜v82.0）を開始する。
最初のステップとして、品質ルールをファーストクラスの型として表現する基盤を構築する。

ロードマップ: `versions/roadmap/roadmap-v81.1-v82.0.md`（v81.1.0 セクション）

> **テスト数補足**: ロードマップは 3831 + 2 = 3833 と記載しているが、
> v80.x スプリントの code-reviewer 累積 drift により実際のベースは **3841**。
> 本バージョンの完了条件は **3841 + 2 = 3843**。

## Goals

- `QualityRuleKind` enum を `test_framework.rs` に追加する
- `RuleSeverity` enum を追加する
- `QualityRule` 構造体を追加する
- `QualityCheck` 構造体を追加する
- `QualityViolation` 構造体を追加する
- `run_quality_check(check: &QualityCheck, rows: &[Vec<String>]) -> Vec<QualityViolation>` を実装する
- テスト 2 件を追加して **3843 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

/// 品質ルールの種類。
#[derive(Debug, Clone)]
pub enum QualityRuleKind {
    /// 値が空文字列でないことを確認する。
    NotNull,
    /// カラム内の値が一意であることを確認する（`run_quality_check` ではサポート外: 行単位チェックのため）。
    Unique,
    /// 数値が `[min, max]` 範囲内であることを確認する（`f64` にパース可能な文字列が対象）。
    Range { min: f64, max: f64 },
    /// 値に指定パターンが部分文字列として含まれることを確認する。
    /// 現バージョンでは `value.contains(pattern)` で実装する（本格的な正規表現エンジンは将来バージョンで追加予定）。
    Regex(String),
    /// カスタムルール名（現バージョンでは常に違反なしとして扱う）。
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
    /// 違反が検出された行のインデックス（0 始まり）。
    pub row_index: usize,
    /// 違反した実際の値（文字列表現）。
    pub actual: String,
}

/// `QualityCheck` のルールを全行に対して適用し、違反一覧を返す。
///
/// 各行は `Vec<String>` であり、`QualityRule.column` はカラム名ではなく
/// カラムのインデックス文字列（"0", "1", ...）として扱う。
/// カラムインデックスが行の範囲外の場合はその行をスキップする。
///
/// `QualityRuleKind` の適用ルール:
/// - `NotNull`: 値が空文字列（`""` または空白のみ）なら違反
/// - `Range { min, max }`: `f64` にパース可能で `v < min || v > max` なら違反（パース失敗はスキップ）
/// - `Regex(pattern)`: 値に `pattern` が含まれない（`!value.contains(pattern)`）なら違反
/// - `Unique` / `Custom`: 行単位チェック非対応のため常にスキップ
pub fn run_quality_check(check: &QualityCheck, rows: &[Vec<String>]) -> Vec<QualityViolation>;
```

### 出力例

```rust
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
    vec!["".to_string(),   "50".to_string()],   // 行0: col0 が空 → NotNull 違反
    vec!["ok".to_string(), "150".to_string()],  // 行1: col1 = 150 > 100 → Range 違反
    vec!["ok".to_string(), "50".to_string()],   // 行2: 違反なし
];
let violations = run_quality_check(&check, &rows);
// violations.len() == 2
// violations[0]: row_index=0, actual=""
// violations[1]: row_index=1, actual="150"
```

## Success Criteria

- `cargo test` が **3843 tests**, 0 failures
- `quality_rule_not_null_catches_violation`:
  - 空値を含む行に `QualityRuleKind::NotNull` を適用すると `QualityViolation` が返される
  - `violation.row_index` と `violation.actual` が正しい
- `quality_check_returns_all_violations`:
  - 複数ルール（NotNull + Range）を持つ `QualityCheck` を適用すると全違反が返される
  - 違反のない行は含まれない
- 範囲外カラムインデックス動作（`run_quality_check` の実装要件として明記）:
  - `column: "99"` のようにインデックスが行の範囲外の場合、その行はスキップされ違反として報告されない
  - テストとしては追加せず（2 テストのみが完了条件）、`run_quality_check` のドキュメントコメントに明記する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `QualityRuleKind` / `RuleSeverity` / `QualityRule` / `QualityCheck` / `QualityViolation` / `run_quality_check` |
| `fav/src/driver.rs` | 追記 | `mod v81100_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。

## Error Codes

新規エラーコードなし。

## 注記

- カラムインデックス方式（`column` フィールドが "0"/"1"/... のインデックス文字列）は
  ロードマップの設計に従う。カラム名マッピングは v81.x の後続バージョンで検討。
- `Unique` / `Custom` の行単位チェック非対応は本バージョンのスコープ外。
- `Regex` ルールの本格的な正規表現マッチは本バージョンでは `contains` で代替する。
