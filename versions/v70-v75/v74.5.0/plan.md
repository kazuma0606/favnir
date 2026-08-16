# v74.5.0 実装計画 — Pipeline Scheduling（`fav schedule`）

Date: 2026-08-14

---

## 実装ステップ

### Step 1: 構造体 + 関数を `driver.rs` に追加

```rust
// --- v74.5.0: Pipeline Scheduling（fav schedule） ---

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleEntry {
    pub name: String,
    pub cron: String,
    pub pipeline: String,
    pub notify: String,
}

/// cron 式を簡易バリデーションする（スペース区切り 5 フィールドのみチェック）
pub fn validate_cron_expr(expr: &str) -> bool {
    expr.split_whitespace().count() == 5
}

/// スケジュール一覧をテキスト形式で返す
pub fn cmd_schedule_list(entries: &[ScheduleEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}    {}    {}", e.name, e.cron, e.pipeline))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Step 2: `v745000_tests` モジュールを追加

`v744000_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v745000_tests {
    use super::{ScheduleEntry, validate_cron_expr, cmd_schedule_list};

    #[test]
    fn schedule_add_parses_cron() {
        let entry = ScheduleEntry {
            name: "daily-report".to_string(),
            cron: "0 9 * * *".to_string(),
            pipeline: "pipelines/daily_report.fav".to_string(),
            notify: "slack://my-channel".to_string(),
        };
        assert_eq!(entry.name, "daily-report");
        assert_eq!(entry.cron, "0 9 * * *");
        assert_eq!(entry.pipeline, "pipelines/daily_report.fav");

        // cron バリデーション
        assert!(validate_cron_expr("0 9 * * *"), "valid cron should pass");
        assert!(validate_cron_expr("0 * * * *"), "hourly cron should pass");
        assert!(!validate_cron_expr("invalid"), "invalid cron should fail");
        assert!(!validate_cron_expr("0 9 * *"), "4-field cron should fail");
        assert!(!validate_cron_expr(""), "empty cron should fail");
    }

    #[test]
    fn schedule_list_returns_entries() {
        let entries = vec![
            ScheduleEntry {
                name: "daily-report".to_string(),
                cron: "0 9 * * *".to_string(),
                pipeline: "pipelines/daily_report.fav".to_string(),
                notify: "".to_string(),
            },
            ScheduleEntry {
                name: "hourly-sync".to_string(),
                cron: "0 * * * *".to_string(),
                pipeline: "pipelines/hourly_sync.fav".to_string(),
                notify: "".to_string(),
            },
        ];
        let output = cmd_schedule_list(&entries);
        assert!(output.contains("daily-report"), "daily-report missing");
        assert!(output.contains("0 9 * * *"), "cron missing");
        assert!(output.contains("hourly-sync"), "hourly-sync missing");

        // 空スライス → 空文字列
        let empty = cmd_schedule_list(&[]);
        assert_eq!(empty, "", "empty entries should return empty string");
    }
}
```

### Step 3: バージョン更新

- `fav/Cargo.toml`: `version = "74.4.0"` → `version = "74.5.0"`
- `driver.rs` 内の `version = "74.4.0"` 参照を `version = "74.5.0"` に replace_all（コメント・セクションヘッダーは置換不要）
- `version should be 74.4.0` を `version should be 74.5.0` に replace_all（アサートメッセージのみ）
- `cargo build` で `Cargo.lock` が自動更新される

### Step 4: テスト確認

- `cargo test v745000` で 2 件 pass を確認
- `cargo test` 全体で 3680 tests pass を確認

### Step 5: `CHANGELOG.md` 更新

v74.5.0 エントリを先頭に追加。

### Step 6: `versions/current.md` 更新

- 最終更新: `2026-08-14 (v74.5.0)`
- 進行中: `v74.5.0`
- 次: `v74.6.0`
