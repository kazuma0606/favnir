# Plan: v80.7.0 — スキーマスナップショットテスト（`SchemaSnapshot`）

実装依存順（既存モジュール追記 → テスト追加）

> `lib.rs` 変更不要。`driver.rs` はバイナリクレートのため `fav_core::test_framework::*` を使用。
> `#[cfg(test)] mod v80700_tests` パターン（v80.1.0〜v80.6.0 の慣例）。

---

## Step 1: `fav/src/test_framework.rs` に型と実装を追加

`coverage_pct` の後ろに以下を追記する。

```rust
// ─── SchemaSnapshot ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSnapshot {
    pub name: String,
    pub type_name: String,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub struct SchemaSnapshot {
    pub pipeline_name: String,
    pub columns: Vec<ColumnSnapshot>,
}

#[derive(Debug)]
pub struct SchemaSnapshotDiff {
    /// current にあって baseline にない列名。
    pub added: Vec<String>,
    /// baseline にあって current にない列名。
    pub removed: Vec<String>,
    /// 両方に存在するが type_name または nullable が異なる列名。
    pub changed: Vec<String>,
}

/// current と baseline を比較してスキーマ差分を返す。
/// 列の突き合わせは名前（`name` フィールド）で行い、列順は問わない。
pub fn compare_schema_snapshots(
    current: &SchemaSnapshot,
    baseline: &SchemaSnapshot,
) -> SchemaSnapshotDiff {
    use std::collections::HashMap;

    let current_map: HashMap<&str, &ColumnSnapshot> =
        current.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let baseline_map: HashMap<&str, &ColumnSnapshot> =
        baseline.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    // baseline 側: removed / changed を検出
    for (name, base_col) in &baseline_map {
        match current_map.get(name) {
            None => removed.push((*name).to_string()),
            Some(cur_col) => {
                if cur_col.type_name != base_col.type_name || cur_col.nullable != base_col.nullable {
                    changed.push((*name).to_string());
                }
            }
        }
    }

    // current 側: added を検出
    for name in current_map.keys() {
        if !baseline_map.contains_key(name) {
            added.push((*name).to_string());
        }
    }

    // 出力の安定性のためにソートする
    added.sort();
    removed.sort();
    changed.sort();

    SchemaSnapshotDiff { added, removed, changed }
}

/// diff を "OK: schema unchanged" または "added=[...], removed=[...], changed=[...]" に変換する。
pub fn format_schema_diff(diff: &SchemaSnapshotDiff) -> String {
    if diff.added.is_empty() && diff.removed.is_empty() && diff.changed.is_empty() {
        return "OK: schema unchanged".to_string();
    }
    format!(
        "added=[{}], removed=[{}], changed=[{}]",
        diff.added.join(", "),
        diff.removed.join(", "),
        diff.changed.join(", "),
    )
}

/// removed または changed が 1 件以上あれば破壊的変更（true）。
/// added のみであれば後方互換（false）。
pub fn schema_diff_is_breaking(diff: &SchemaSnapshotDiff) -> bool {
    !diff.removed.is_empty() || !diff.changed.is_empty()
}
```

---

## Step 2: `fav/src/driver.rs` に `mod v80700_tests` を追加

`mod v80600_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v80700_tests {
    use fav_core::test_framework::*;

    fn baseline() -> SchemaSnapshot {
        SchemaSnapshot {
            pipeline_name: "orders".to_string(),
            columns: vec![
                ColumnSnapshot { name: "id".to_string(),     type_name: "Int".to_string(),   nullable: false },
                ColumnSnapshot { name: "amount".to_string(), type_name: "Float".to_string(), nullable: false },
            ],
        }
    }

    #[test]
    fn schema_snapshot_no_diff_when_equal() {
        let b = baseline();
        let diff = compare_schema_snapshots(&b, &b);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.changed.is_empty());
        assert_eq!(format_schema_diff(&diff), "OK: schema unchanged");
        assert!(!schema_diff_is_breaking(&diff));
    }

    #[test]
    fn schema_snapshot_detects_removed_column() {
        let b = baseline();
        let current = SchemaSnapshot {
            pipeline_name: "orders".to_string(),
            columns: vec![
                ColumnSnapshot { name: "id".to_string(),   type_name: "Int".to_string(),    nullable: false },
                ColumnSnapshot { name: "note".to_string(), type_name: "String".to_string(), nullable: true },
            ],
        };
        let diff = compare_schema_snapshots(&current, &b);
        assert_eq!(diff.removed, vec!["amount"]);
        assert_eq!(diff.added,   vec!["note"]);
        assert!(diff.changed.is_empty());
        assert!(schema_diff_is_breaking(&diff));
        assert_eq!(format_schema_diff(&diff), "added=[note], removed=[amount], changed=[]");
    }
}
```

---

## Step 3: `cargo test` で全 pass を確認

```bash
cargo test 2>&1 | tail -5
```

3830 tests, 0 failures であることを確認する。
（ロードマップ記載は 3823 だが、v80.2.0〜v80.6.0 の code-reviewer 対応で累積 +7 されているため実際の目標は 3830）
