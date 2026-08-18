# Plan: v80.2.0 — `GoldenDataset` / ゴールデンデータセット比較

実装依存順（既存モジュール追記 → テスト追加）

> `lib.rs` は変更不要（`pub mod test_framework;` はすでに宣言済み）。
> `driver.rs` はバイナリクレート（`fav`）に属するため `fav_core::test_framework::*` でインポートする。
> 外部クレートのインポートを持つテストモジュールは `#[cfg(test)]` を付与すること（clippy 対策）。

---

## Step 1: `fav/src/test_framework.rs` に型と関数を追加

既存の `format_test_suite_result` の後ろに追記する。

```rust
// ─── GoldenDataset ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct GoldenDataset {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct GoldenCompareResult {
    pub matches: bool,
    pub diff_rows: Vec<usize>,
}

/// 実際の出力と期待値を行単位で比較する。
pub fn compare_golden(actual: &GoldenDataset, expected: &GoldenDataset) -> GoldenCompareResult {
    let mut diff_rows = Vec::new();
    let max_len = actual.rows.len().max(expected.rows.len());
    for i in 0..max_len {
        let a = actual.rows.get(i);
        let e = expected.rows.get(i);
        if a != e {
            diff_rows.push(i);
        }
    }
    let matches = diff_rows.is_empty();
    GoldenCompareResult { matches, diff_rows }
}

/// "OK: datasets match" or "DIFF: N row(s) differ: [0, 2, ...]"
pub fn format_golden_diff(result: &GoldenCompareResult) -> String {
    if result.matches {
        "OK: datasets match".to_string()
    } else {
        let indices: Vec<String> = result.diff_rows.iter().map(|i| i.to_string()).collect();
        format!("DIFF: {} row(s) differ: [{}]", result.diff_rows.len(), indices.join(", "))
    }
}

/// CSV ファイルを読み込んで GoldenDataset を構築する（WASM 非対応）。
#[cfg(not(target_arch = "wasm32"))]
pub fn load_golden_dataset(path: &str) -> Result<GoldenDataset, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to load golden dataset '{}': {}", path, e))?;
    let rows: Vec<Vec<String>> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|s| s.to_string()).collect())
        .collect();
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string();
    Ok(GoldenDataset { name, rows })
}
```

---

## Step 2: `fav/src/driver.rs` に `mod v80200_tests` を追加

`mod v80100_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v80200_tests {
    // v80.1.0 の慣例に合わせてワイルドカードインポートを使用する。
    // load_golden_dataset は #[cfg(not(wasm32))] が付いているがテスト環境は非 WASM のため使用可能。
    use fav_core::test_framework::*;

    #[test]
    fn golden_dataset_compare_pass() {
        let rows = vec![
            vec!["alice".to_string(), "30".to_string()],
            vec!["bob".to_string(), "25".to_string()],
        ];
        let actual = GoldenDataset { name: "actual".to_string(), rows: rows.clone() };
        let expected = GoldenDataset { name: "expected".to_string(), rows };
        let result = compare_golden(&actual, &expected);
        assert!(result.matches);
        assert!(result.diff_rows.is_empty());
        assert_eq!(format_golden_diff(&result), "OK: datasets match");
    }

    #[test]
    fn golden_dataset_compare_fail_shows_diff() {
        let actual = GoldenDataset {
            name: "actual".to_string(),
            rows: vec![
                vec!["alice".to_string(), "30".to_string()],
                vec!["bob".to_string(), "99".to_string()],  // 行 1 が異なる
            ],
        };
        let expected = GoldenDataset {
            name: "expected".to_string(),
            rows: vec![
                vec!["alice".to_string(), "30".to_string()],
                vec!["bob".to_string(), "25".to_string()],  // 行 1 が異なる
            ],
        };
        let result = compare_golden(&actual, &expected);
        assert!(!result.matches);
        assert_eq!(result.diff_rows, vec![1]);
        let diff_str = format_golden_diff(&result);
        assert!(diff_str.contains("DIFF"));
        assert!(diff_str.contains("1"));
    }
}
```

---

## Step 3: `cargo test` で全 pass を確認

```bash
cargo test 2>&1 | grep "test result" | head -3
```

3813 tests, 0 failures であることを確認する。
