# Plan: v81.3.0 — スキーマドリフト検出（`SchemaDriftDetector`）

## Step 1: 前提確認

- `cargo test` を実行し、3847 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.2.0 の `format_distribution_report` が定義済みであることを確認する
- `fav/src/test_framework.rs` に `SchemaSnapshot` / `SchemaSnapshotDiff` / `compare_schema_snapshots` が定義済みであることを確認する（v80.7.0 導入済み）
- `fav/src/test_framework.rs` に `RuleSeverity` が定義済みであることを確認する（v81.1.0 導入済み）

## Step 2: `fav/src/test_framework.rs` に追記

`format_distribution_report` の定義の直後に以下を追加する。

```rust
// ── v81.3.0: SchemaDriftDetector ──────────────────────────────────────────────

/// スキーマ変更の許容レベル。
///
/// - `Strict`: 追加・削除・変更をすべてドリフトとみなす
/// - `Additive`: 削除・変更のみドリフト（追加のみは `has_drift = false`）
/// - `Permissive`: 現バージョンでは `Additive` と同一動作
#[derive(Debug, Clone, PartialEq)]
pub enum DriftTolerance {
    Strict,
    Additive,
    Permissive,
}

/// ベースラインスキーマからのドリフトを検出する設定。
#[derive(Debug, Clone)]
pub struct SchemaDriftDetector {
    pub baseline: SchemaSnapshot,
    pub tolerance: DriftTolerance,
}

/// ドリフト検出結果。
///
/// `has_drift = false` のとき `severity` は `RuleSeverity::Warning`（参照するだけで意味はない）。
#[derive(Debug)]
pub struct DriftResult {
    pub has_drift: bool,
    pub severity: RuleSeverity,
    pub diff: SchemaSnapshotDiff,
}

/// ベースラインと current を比較してドリフト結果を返す。
pub fn detect_schema_drift(detector: &SchemaDriftDetector, current: &SchemaSnapshot) -> DriftResult {
    let diff = compare_schema_snapshots(current, &detector.baseline);
    let any_change = !diff.added.is_empty() || !diff.removed.is_empty() || !diff.changed.is_empty();
    let breaking = !diff.removed.is_empty() || !diff.changed.is_empty();
    let has_drift = match detector.tolerance {
        DriftTolerance::Strict => any_change,
        DriftTolerance::Additive | DriftTolerance::Permissive => breaking,
    };
    let severity = if has_drift { RuleSeverity::Error } else { RuleSeverity::Warning };
    DriftResult { has_drift, severity, diff }
}

/// ドリフト結果を人間向けの文字列に変換する。
///
/// ドリフトなし: `"OK: no schema drift detected"`
/// ドリフトあり: `"DRIFT: added=[...] removed=[...] changed=[...]"`
pub fn format_drift_report(result: &DriftResult) -> String {
    if !result.has_drift {
        return "OK: no schema drift detected".to_string();
    }
    format!(
        "DRIFT: added={:?} removed={:?} changed={:?}",
        result.diff.added, result.diff.removed, result.diff.changed
    )
}
```

## Step 3: `fav/src/driver.rs` に `mod v81300_tests` を追加

`mod v81200_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81300_tests {
    use fav_core::test_framework::*;

    fn make_snapshot(cols: &[(&str, &str, bool)]) -> SchemaSnapshot {
        SchemaSnapshot {
            pipeline_name: "pipe".to_string(),
            columns: cols.iter().map(|(name, ty, nullable)| ColumnSnapshot {
                name: name.to_string(),
                type_name: ty.to_string(),
                nullable: *nullable,
            }).collect(),
        }
    }

    #[test]
    fn drift_detector_strict_mode_catches_addition() {
        let baseline = make_snapshot(&[("id", "Int", false)]);
        let current  = make_snapshot(&[("id", "Int", false), ("name", "String", true)]);
        let detector = SchemaDriftDetector { baseline, tolerance: DriftTolerance::Strict };
        let result = detect_schema_drift(&detector, &current);
        assert!(result.has_drift, "Strict mode should detect added column");
        assert!(result.diff.added.contains(&"name".to_string()),
            "diff.added should contain 'name': {:?}", result.diff.added);
        let report = format_drift_report(&result);
        assert!(report.contains("DRIFT"),  "report should contain DRIFT: {report}");
        assert!(report.contains("added="), "report should contain added=: {report}");
        assert!(report.contains("name"),   "report should mention 'name': {report}");
    }

    #[test]
    fn drift_detector_additive_mode_allows_new_column() {
        let baseline = make_snapshot(&[("id", "Int", false)]);
        let current  = make_snapshot(&[("id", "Int", false), ("name", "String", true)]);
        // 追加のみ → Additive は許容（has_drift = false）
        let detector = SchemaDriftDetector { baseline: baseline.clone(), tolerance: DriftTolerance::Additive };
        let result = detect_schema_drift(&detector, &current);
        assert!(!result.has_drift, "Additive mode should allow new column: has_drift={}", result.has_drift);
        assert!(result.diff.added.contains(&"name".to_string()),
            "diff.added should still be populated: {:?}", result.diff.added);

        // 削除あり → Additive でも drift 検出
        let current_missing = make_snapshot(&[]);
        let detector2 = SchemaDriftDetector { baseline, tolerance: DriftTolerance::Additive };
        let result2 = detect_schema_drift(&detector2, &current_missing);
        assert!(result2.has_drift, "Additive mode should detect removed column");
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3849 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.3.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
