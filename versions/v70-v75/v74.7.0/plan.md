# v74.7.0 実装計画 — コミュニティ Rune 品質基準

Date: 2026-08-14

---

## 実装ステップ

### Step 1: 構造体 + 関数を `driver.rs` に追加

```rust
// --- v74.7.0: コミュニティ Rune 品質基準 ---

#[derive(Debug, Clone, PartialEq)]
pub struct RuneValidationItem {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuneValidationReport {
    pub rune_name: String,
    pub items: Vec<RuneValidationItem>,
    /// 0–100 の整数スコア（呼び出し元が設定）
    pub score: u32,
}

/// score >= 80 なら true（公開要件を満たす）
pub fn validate_rune_score(report: &RuneValidationReport) -> bool {
    report.score >= 80
}

/// レポートをテキスト形式でフォーマットする
pub fn format_rune_validation_report(report: &RuneValidationReport) -> String {
    let mut lines: Vec<String> = report
        .items
        .iter()
        .map(|item| {
            let mark = if item.passed { "✓" } else { "⚠" };
            format!("{} {}: {}", mark, item.name, item.message)
        })
        .collect();
    lines.push(format!("Score: {}/100 (Publish requires >= 80)", report.score));
    lines.join("\n")
}
```

### Step 2: `v747000_tests` モジュールを追加

`v746000_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v747000_tests {
    use super::{RuneValidationItem, RuneValidationReport, validate_rune_score, format_rune_validation_report};

    #[test]
    fn rune_validate_scoring() {
        let report = RuneValidationReport {
            rune_name: "my-rune".to_string(),
            items: vec![
                RuneValidationItem { name: "rune.toml".to_string(), passed: true, message: "valid".to_string() },
                RuneValidationItem { name: "implementation".to_string(), passed: true, message: "my-rune.fav (247 lines)".to_string() },
                RuneValidationItem { name: "tests".to_string(), passed: true, message: "3 test cases found".to_string() },
                RuneValidationItem { name: "documentation".to_string(), passed: true, message: "README.md exists".to_string() },
                RuneValidationItem { name: "example".to_string(), passed: false, message: "No example .fav file found".to_string() },
            ],
            score: 85,
        };
        assert_eq!(report.rune_name, "my-rune");
        assert_eq!(report.items.len(), 5);
        assert_eq!(report.score, 85);

        let output = format_rune_validation_report(&report);
        assert!(output.contains("✓"), "passed items should show ✓");
        assert!(output.contains("⚠"), "failed items should show ⚠");
        assert!(output.contains("Score:"), "score line missing");
        assert!(output.contains("85"), "score value missing");
        assert!(output.contains("80"), "publish threshold missing");
    }

    #[test]
    fn rune_validate_min_score_enforced() {
        let make_report = |score: u32| RuneValidationReport {
            rune_name: "test-rune".to_string(),
            items: vec![],
            score,
        };

        // 公開要件を満たす
        assert!(validate_rune_score(&make_report(100)), "score 100 should pass");
        assert!(validate_rune_score(&make_report(80)), "score 80 (border) should pass");

        // 公開要件を満たさない
        assert!(!validate_rune_score(&make_report(79)), "score 79 should fail");
        assert!(!validate_rune_score(&make_report(0)), "score 0 should fail");
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.6.0"` → `version = "74.7.0"`
- `driver.rs` 内の `version = \"74.6.0\"` を `version = \"74.7.0\"` に replace_all
- `version should be 74.6.0` を `version should be 74.7.0` に replace_all
- `cargo build` で `Cargo.lock` が自動更新される

### Step 4: テスト確認

- `cargo test v747000` で 2 件 pass を確認
- `cargo test` 全体で 3684 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.7.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-14 (v74.7.0)`
- 進行中: `v74.7.0`
- 次: `v74.8.0`
