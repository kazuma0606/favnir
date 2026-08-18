# Spec: v80.7.0 — スキーマスナップショットテスト（`SchemaSnapshot`）

## Background

v80.1.0〜v80.6.0 でテストフレームワーク基盤を構築した。
本バージョンでは **スキーマの変更を検出する型** を追加する。
パイプラインのスキーマ（列名・型・nullable 制約）を `SchemaSnapshot` として記録し、
`compare_schema_snapshots` で現在のスキーマとベースラインを突き合わせて差分を返す。

ロードマップ: `versions/roadmap/roadmap-v80.1-v81.0.md`（v80.7.0 セクション）

> **テスト数補足**: ロードマップは 3821 + 2 = 3823 と記載しているが、
> v80.2.0〜v80.6.0 の code-reviewer 対応で累積 7 件追加されたため実際のベースは **3828**。
> 本バージョンの完了条件は **3828 + 2 = 3830**。

## Goals

- `ColumnSnapshot` 構造体を `test_framework.rs` に追加する
- `SchemaSnapshot` 構造体を追加する
- `SchemaSnapshotDiff` 構造体を追加する
- `compare_schema_snapshots(current: &SchemaSnapshot, baseline: &SchemaSnapshot) -> SchemaSnapshotDiff` を実装する
- `format_schema_diff(diff: &SchemaSnapshotDiff) -> String` を実装する
- `schema_diff_is_breaking(diff: &SchemaSnapshotDiff) -> bool` を実装する
- テスト 2 件を追加して **3830 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

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
///
/// - baseline の各列について current に同名列がなければ `removed` に追加。
/// - 同名列が存在し type_name または nullable が異なれば `changed` に追加。
/// - current の各列について baseline に同名列がなければ `added` に追加。
/// - 列の順序は問わず名前で突き合わせる。
pub fn compare_schema_snapshots(
    current: &SchemaSnapshot,
    baseline: &SchemaSnapshot,
) -> SchemaSnapshotDiff;

/// diff を人間が読める文字列に変換する。
/// - 差分なし: "OK: schema unchanged"
/// - 差分あり: "added=[...], removed=[...], changed=[...]"（空リストも出力する）
pub fn format_schema_diff(diff: &SchemaSnapshotDiff) -> String;

/// removed または changed が 1 件以上あれば true（破壊的変更）。
/// added のみであれば false（後方互換）。
pub fn schema_diff_is_breaking(diff: &SchemaSnapshotDiff) -> bool;
```

### `compare_schema_snapshots` の動作例

```rust
let baseline = SchemaSnapshot {
    pipeline_name: "orders".to_string(),
    columns: vec![
        ColumnSnapshot { name: "id".to_string(),    type_name: "Int".to_string(),    nullable: false },
        ColumnSnapshot { name: "amount".to_string(), type_name: "Float".to_string(), nullable: false },
    ],
};
let current = SchemaSnapshot {
    pipeline_name: "orders".to_string(),
    columns: vec![
        ColumnSnapshot { name: "id".to_string(),    type_name: "Int".to_string(),    nullable: false },
        ColumnSnapshot { name: "note".to_string(),  type_name: "String".to_string(), nullable: true },
    ],
};
let diff = compare_schema_snapshots(&current, &baseline);
// diff.added   == ["note"]
// diff.removed == ["amount"]
// diff.changed == []
// schema_diff_is_breaking(&diff) == true   ← removed があるため
// format_schema_diff(&diff) == "added=[note], removed=[amount], changed=[]"
```

### 差分なしの例

```rust
let diff = compare_schema_snapshots(&baseline, &baseline);
// diff.added == [], diff.removed == [], diff.changed == []
// format_schema_diff(&diff) == "OK: schema unchanged"
// schema_diff_is_breaking(&diff) == false
```

## Success Criteria

- `cargo test` が **3830 tests**, 0 failures
- `schema_snapshot_no_diff_when_equal`:
  - 同一スキーマを baseline / current として compare → 全フィールドが空
  - `format_schema_diff` が `"OK: schema unchanged"` を返す
  - `schema_diff_is_breaking` が `false` を返す
- `schema_snapshot_detects_removed_column`:
  - baseline に 2 列、current に baseline と同名列 1 + 新列 1
  - `diff.removed == ["amount"]`、`diff.added == ["note"]`
  - `schema_diff_is_breaking` が `true` を返す（removed があるため）

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `ColumnSnapshot` / `SchemaSnapshot` / `SchemaSnapshotDiff` / `compare_schema_snapshots` / `format_schema_diff` / `schema_diff_is_breaking` |
| `fav/src/driver.rs` | 追記 | `mod v80700_tests`（テスト 2 件） |

> `lib.rs` への変更は不要（`pub mod test_framework;` は v80.1.0 で宣言済み）。

## Error Codes

新規エラーコードなし。

## 注記

- 列の突き合わせは **名前（`name` フィールド）のみ** で行う。列順は問わない。
- `ColumnSnapshot` には `#[derive(PartialEq)]` を付けて型・nullable の比較を `!=` で行えるようにする。
- `schema_diff_is_breaking` は `removed` または `changed` が非空のときのみ `true`。列追加（`added`）は後方互換とみなす。
- MILESTONE.md / README.md / `site/content/docs/` の更新は v81.0.0 宣言バージョンでまとめて実施する。
