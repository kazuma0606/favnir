# v81.9.0 タスクリスト

Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` が 3,859 tests pass、0 failures であることを確認する（前提: v81.8.0 完了済み）
- [x] v81.1〜v81.8 のテスト名がすべて通過していることを確認する
- [x] v81.7.0 テスト `quality_report_text_format` / `quality_report_json_format` が pass していることで `fav quality report` コマンドの E2E 動作を確認済みとみなす

## T1: 統合テスト追加（`data_quality_full_sprint_all_stable`）

- [x] `fav/src/driver.rs` の v81800_tests モジュール末尾に `data_quality_full_sprint_all_stable` テストを追加する
  - `QualityRule`（column="0", NotNull）→ `QualityCheck` → `run_quality_check`（違反 1 件を検出）
  - `compute_quality_score`（Completeness=0.8）→ `build_quality_report`（Text）→ `"violations"` を含むことを確認
  - `evaluate_quality_gate`（permissive）→ `Pass` を確認

## T2: 統合テスト追加（`quality_gate_and_drift_detector_integrated`）

- [x] `fav/src/driver.rs` の v81800_tests モジュール末尾に `quality_gate_and_drift_detector_integrated` テストを追加する
  - `ColumnSnapshot`（id, name）→ ベースライン `SchemaSnapshot` を作成
  - `name` を削除した `current` を作成 → `SchemaDriftDetector`（Strict）→ `has_drift == true` を確認
  - `compute_quality_score`（Completeness=0.5）→ `QualityGate::strict()` → `Fail` を確認

## T3: テスト通過確認

- [x] `cargo test` が 3,861 tests pass（+2）、0 failures であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v81.9.0 エントリを追加する

## T5: 最終確認（CI チェック）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

---

## コードレビュー指摘と対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | テストが `v81800_tests` に混入していた | `v81900_tests` モジュールに分離 ✅ |
| [MED] | spec.md の Files to Modify が `test_framework.rs` と誤記 | `driver.rs` に修正 ✅ |
| [LOW] | spec.md サンプルコードの `column: "age"` が実装と不一致 | `"0"` に修正（`run_quality_check` は数値インデックス文字列を使用）✅ |
