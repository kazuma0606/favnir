# Spec: v81.3.0 — スキーマドリフト検出（`SchemaDriftDetector`）

## Background

v80.7.0 で `SchemaSnapshot` / `SchemaSnapshotDiff` / `ColumnSnapshot` が導入済みである。
v81.1.0 で `RuleSeverity` が導入済みである。
本バージョンでは、ベースラインスキーマとの差分を自動検出し品質ゲートをトリガーする
`SchemaDriftDetector` 型を追加する。

ロードマップ: `versions/roadmap/roadmap-v81.1-v82.0.md`（v81.3.0 セクション）

> **テスト数**: v81.2.0 code-reviewer drift +2 補正済み。実際のベースは **3847**。
> 本バージョンの完了条件は **3847 + 2 = 3849**。

## Goals

- `DriftTolerance` enum を `test_framework.rs` に追加する
- `SchemaDriftDetector` 構造体を追加する
- `DriftResult` 構造体を追加する
- `detect_schema_drift(detector: &SchemaDriftDetector, current: &SchemaSnapshot) -> DriftResult` を実装する
- `format_drift_report(result: &DriftResult) -> String` を実装する
- テスト 2 件を追加して **3849 tests** を達成する

## API / Type Definitions

```rust
// fav/src/test_framework.rs（既存ファイルに追記）

/// スキーマ変更の許容レベル。
///
/// - `Strict`: 追加・削除・変更をすべてドリフトとみなす（`has_drift = true`）
/// - `Additive`: 削除・変更のみドリフト。追加のみの場合は `has_drift = false`
/// - `Permissive`: 削除・変更のみドリフト。追加は常に許容（Additive と同じ動作）
///
/// 現バージョンでは `Additive` と `Permissive` は同一動作とする。
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
///
/// - `DriftTolerance::Strict`: diff に追加・削除・変更が 1 件でもあれば `has_drift = true`
/// - `DriftTolerance::Additive` / `Permissive`: 削除・変更がある場合のみ `has_drift = true`
///   （追加のみの場合は `has_drift = false`）
///
/// `has_drift = true` のとき `severity = RuleSeverity::Error`。
/// `has_drift = false` のとき `severity = RuleSeverity::Warning`。
pub fn detect_schema_drift(detector: &SchemaDriftDetector, current: &SchemaSnapshot) -> DriftResult;

/// ドリフト結果を人間向けの文字列に変換する。
///
/// ドリフトなし: `"OK: no schema drift detected"`
/// ドリフトあり: `"DRIFT: added=[...] removed=[...] changed=[...]"`
pub fn format_drift_report(result: &DriftResult) -> String;
```

### 出力例

```rust
bind baseline <- SchemaSnapshot {
    pipeline_name: "pipe".to_string(),
    columns: vec![ColumnSnapshot { name: "id".to_string(), type_name: "Int".to_string(), nullable: false }],
};
bind current <- SchemaSnapshot {
    pipeline_name: "pipe".to_string(),
    columns: vec![
        ColumnSnapshot { name: "id".to_string(), type_name: "Int".to_string(), nullable: false },
        ColumnSnapshot { name: "name".to_string(), type_name: "String".to_string(), nullable: true },
    ],
};

// Strict モード: 追加もドリフト
bind detector_strict <- SchemaDriftDetector { baseline: baseline.clone(), tolerance: DriftTolerance::Strict };
bind result_strict <- detect_schema_drift(&detector_strict, &current);
// result_strict.has_drift == true
// result_strict.diff.added == ["name"]

// Additive モード: 追加は許容
bind detector_additive <- SchemaDriftDetector { baseline: baseline.clone(), tolerance: DriftTolerance::Additive };
bind result_additive <- detect_schema_drift(&detector_additive, &current);
// result_additive.has_drift == false
// result_additive.diff.added == ["name"]  (diff は常に populate される)
```

## Success Criteria

- `cargo test` が **3849 tests**, 0 failures
- `drift_detector_strict_mode_catches_addition`:
  - baseline に列 `"id"` のみ、current に `"id"` と `"name"` がある場合
  - `DriftTolerance::Strict` で `has_drift = true` を確認する
  - `diff.added` に `"name"` が含まれることを確認する
  - `format_drift_report` の出力に `"DRIFT"` と `"name"` が含まれることを確認する
- `drift_detector_additive_mode_allows_new_column`:
  - 同じデータで `DriftTolerance::Additive` を使うと `has_drift = false` を確認する
  - `diff.added` に `"name"` が含まれることを確認する（diff は常に計算される）
  - 削除ありデータで `DriftTolerance::Additive` を使うと `has_drift = true` を確認する

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/test_framework.rs` | 追記 | `DriftTolerance` / `SchemaDriftDetector` / `DriftResult` / `detect_schema_drift` / `format_drift_report` |
| `fav/src/driver.rs` | 追記 | `mod v81300_tests`（テスト 2 件） |

## Error Codes

新規エラーコードなし。

## 注記

- `detect_schema_drift` は内部で `compare_schema_snapshots` を呼んで `SchemaSnapshotDiff` を取得する。
- `compare_schema_snapshots(current, baseline)` の引数順序: 第 1 引数 `current` に存在して第 2 引数 `baseline` に存在しない列が `added`、逆が `removed`。
- `DriftResult.diff` はモード（tolerance）によらず常に populate する（ドリフトなしでも diff は返す）。
- `Additive` と `Permissive` は現バージョンでは同一動作。将来的に `Permissive` は型変更も許容する方向を想定。
- `format_drift_report` の成功条件確認: `report.contains("DRIFT")` と `report.contains("name")` に加え、`report.contains("added=")` でフォーマット形式も確認する。
