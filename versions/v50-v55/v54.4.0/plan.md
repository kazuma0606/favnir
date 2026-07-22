# Plan: v54.4.0 — fav dq-report データ品質レポートコマンド

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3191 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54400_tests が未存在を確認
rg -n "v54400_tests" fav/src/driver.rs  # → 0 件

# v54300_tests の行番号を確認（挿入位置）
rg -n "v54300_tests" fav/src/driver.rs

# cmd_dq_report_collect が未存在を確認
rg -n "cmd_dq_report" fav/src/driver.rs  # → 0 件

# dq-report コマンドが未存在を確認
grep "dq-report" fav/src/main.rs  # → 0 件

# Cargo.toml が 54.3.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.3.0"
```

---

## ステップ 2: `driver.rs` — `cmd_dq_report_collect` 追加

`v54300_tests` の直前（テストブロック開始行の直前）に追加:

```rust
// ── v54.4.0: fav dq-report ───────────────────────────────────────────────────

/// Generate a data-quality Markdown report from an audit-log JSONL string.
///
/// Each line of `audit_log` is expected to be a JSON object with at least
/// `"ts"` and `"op"` fields (written by `fav run --audit-log`).
/// Lines with `"op":"schema_error"` count as validation errors.
/// Lines with `"latency_ms"` > 200 count as SLA violations.
///
/// Note: a single audit log line may contribute to **both** schema statistics
/// and SLA statistics.
pub fn cmd_dq_report_collect(audit_log: &str) -> String {
    let mut total_rows: u64 = 0;
    let mut error_rows: u64 = 0;
    let mut schema_counts: HashMap<String, (u64, u64)> = HashMap::new();
    let mut sla_violations: Vec<String> = Vec::new();

    for line in audit_log.lines() { ... }

    // Markdown 生成
    // "# Data Quality Report\n\n"
    // "Schema validation:  N rows checked, E errors (P%)\n"
    // "  Schema: ok / total OK|ERRORS\n"
    // "\nSLA violations:  none\n" or "\nSLA violations:\n  ...\n"
}
```

重要実装ポイント:
- `schema` は `entry()` に直接 move（`.clone()` 不要）
- スキーマ名はソート済みで出力
- `error_pct` は `total_rows == 0` のとき `"0.00%"` を返す

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 3: `main.rs` — `fav dq-report` コマンド追加

`Some("watch")` の直前に追加:

```rust
// ── v54.4.0: fav dq-report ───────────────────────────────────────────────────
Some("dq-report") => {
    let audit_log_path = args.iter().position(|a| a == "--audit-log")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            eprintln!("error: fav dq-report requires --audit-log <path>");
            process::exit(1);
        });
    let audit_log_content = std::fs::read_to_string(audit_log_path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {audit_log_path}: {e}");
        process::exit(1);
    });
    let report = driver::cmd_dq_report_collect(&audit_log_content);
    println!("{report}");
}
```

`cargo build` → コンパイルエラーなし確認。

---

## ステップ 4: `driver.rs` — `v54400_tests` 追加

`v54300_tests` の直前に追加:

```rust
// -- v54400_tests (v54.4.0) -- fav dq-report データ品質レポートコマンド --
#[cfg(test)]
mod v54400_tests {
    use super::*;

    const SAMPLE_AUDIT_LOG: &str = r#"
{"ts":"2026-07-22T10:00:00Z","op":"schema_check","schema":"OrderRow","stage":"Parse"}
{"ts":"2026-07-22T10:00:01Z","op":"schema_check","schema":"OrderRow","stage":"Parse"}
{"ts":"2026-07-22T10:00:02Z","op":"schema_error","schema":"OrderRow","stage":"Parse"}
{"ts":"2026-07-22T10:00:03Z","op":"schema_check","schema":"PaymentRow","stage":"Validate"}
{"ts":"2026-07-22T10:00:04Z","op":"write","effect":"Snowflake","sql":"INSERT ..."}
{"ts":"2026-07-22T10:01:00Z","op":"schema_check","schema":"OrderRow","stage":"Parse","latency_ms":250}
"#;

    #[test]
    fn cmd_dq_report_generates() {
        let report = cmd_dq_report_collect(SAMPLE_AUDIT_LOG);
        assert!(!report.is_empty(), ...);
        assert!(report.contains("Data Quality Report"), ...);
    }

    #[test]
    fn cmd_dq_report_has_schema_stats() {
        let report = cmd_dq_report_collect(SAMPLE_AUDIT_LOG);
        assert!(report.contains("Schema validation"), ...);
        assert!(report.contains("rows checked"), ...);
        assert!(report.contains("OrderRow"), ...);
        assert!(report.contains("SLA violations"), ...);
        // latency_ms:250 > 200 → SLA violation expected
        assert!(!report.contains("SLA violations:  none"), ...);
    }
}
```

---

## ステップ 5: `fav/Cargo.toml` バージョン更新

`version = "54.3.0"` → `version = "54.4.0"`

---

## ステップ 6: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

期待値: 3193 passed, 0 failed

```bash
cargo clippy -- -D warnings
```

---

## ステップ 7: 後処理

- `CHANGELOG.md`: v54.4.0 エントリ追加（v54.3.0 の直上）
- `versions/current.md` を v54.4.0（3193 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.4.0 実績欄を COMPLETE に更新
- `tasks.md` を COMPLETE に更新（T0〜T7 全 `[x]`）

コードレビュー対応（実施済み）:
- [MED] `!report.contains("none")` → `!report.contains("SLA violations:  none")` に精度向上
- [MED] `schema.clone()` → `schema` move に変更
- [LOW] doc コメントに「1行が schema 統計と SLA 統計の両方に寄与しうる」旨を追記
- [LOW] `(u64, u64)` 構造体化は将来バージョンに委ねる（現状コメント補完済み）
