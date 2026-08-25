# v81.9.0 実装計画

## 方針

**前提**: v81.8.0 完了済み（3,859 tests pass）。

バグ修正のみ。新規 API / struct の追加なし。
`test_framework.rs` の `#[cfg(test)]` モジュールに統合テスト 2 件を追加する。

---

## 実装ステップ

### Step 1: 既存テストの全通過確認

```bash
cargo test 2>&1 | grep "test result"
```

3,859 tests pass、0 failures であることを確認する。

`quality_report_text_format` および `quality_report_json_format`（v81.7.0 追加）が通過していることで、
`fav quality report` コマンド（`cmd_quality_report`）の E2E 動作を確認済みとみなす。

### Step 2: `data_quality_full_sprint_all_stable` テスト追加

`fav/src/test_framework.rs` の `#[cfg(test)]` ブロック末尾に追加する。

テストの内容:
- `QualityRule`（NotNull）を 1 件作成
- `QualityCheck` に詰めて `run_quality_check` を呼び出す
- 行データとして `["25"]`（正常）と `[""]`（違反）を渡す
- `violations.len() == 1` を確認
- `compute_quality_score` で `QualityScore` を生成
- `build_quality_report`（Text フォーマット）でレポートを生成し `"violations"` を含むことを確認
- `evaluate_quality_gate`（permissive）で `Pass` を確認

### Step 3: `quality_gate_and_drift_detector_integrated` テスト追加

同じく `#[cfg(test)]` ブロック末尾に追加する。

テストの内容:
- `ColumnSnapshot` 2 件（`id`, `name`）でベースライン `SchemaSnapshot` を作成
- `name` を削除した `current` を作成
- `SchemaDriftDetector`（Strict）で `detect_schema_drift` を呼び出し `has_drift == true` を確認
- `compute_quality_score`（Completeness=0.5）→ `QualityGate::strict()` → `evaluate_quality_gate` で `Fail` を確認

### Step 4: `cargo test` 全通過確認

```bash
cargo test 2>&1 | grep "test result"
```

3,861 tests pass（+2）、0 failures であることを確認する。

### Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.9.0 のエントリを追加する。
