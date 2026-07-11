# v36.7.0 タスクリスト — Great Expectations 互換エクスポート

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.7.0（「Great Expectations 互換エクスポート」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2686（v36.6.0 完了時点の実績値））し、実測値をここに記録: 2686
- [x] Cargo.toml バージョンが `36.6.0` であることを確認
- [x] `v36600_tests::cargo_toml_version_is_36_6_0` がライブアサーション（`assert!(cargo.contains("36.6.0"), ...)`）であることを確認
- [x] `driver.rs` に `v36700_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `driver.rs` に `export_ge_suite` が存在しないことを確認（今回追加）
- [x] `CHANGELOG.md` に `[v36.7.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `driver.rs` に `write_text_file` ヘルパーが存在することを確認（line 448、`fn write_text_file(path: &Path, contents: &str) -> Result<(), String>`）
- [x] `cmd_validate` の呼び出し箇所が `main.rs` のみであることを Grep で確認（main.rs L1413）
- [x] `cmd_validate` の現在のシグネチャ（`schema_file: Option<&str>, data_file: Option<&str>`）を確認
- [x] `main.rs` の既存 `cmd_validate(schema_file.as_deref(), data_file.as_deref())` 呼び出し（2引数版）の行番号を確認し T4 で漏れなく更新する（L1413 → T4 で更新済み）
- [x] `v36600_tests` の閉じ `}` の行番号を確認し、ここに記録: 42781（スタブ化後 42825 に移動）
- [x] `versions/current.md` の最新安定版が `v36.6.0`・次バージョンが `v36.7.0` であることを確認

## T1: CHANGELOG.md に [v36.7.0] エントリを追加

- [x] `## [v36.6.0]` の `---` セパレータ直後に `## [v36.7.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-08）

## T2: driver.rs — `export_ge_suite` 追加

- [x] `validate_schema_against_headers` の `}` の後（`// ── fav validate` セクション内）に `export_ge_suite` を追加
- [x] `pub fn export_ge_suite` として宣言する（テストから `use super::export_ge_suite;` で参照するため `pub` 必須）
- [x] 純粋関数（ファイル I/O なし）として実装
- [x] JSON フォーマット: `expectation_suite_name` / `expectations`（`expect_column_to_exist` のみ）/ `meta`

## T3: driver.rs — `cmd_validate` シグネチャ変更

- [x] `export_fmt: Option<&str>` と `output_file: Option<&str>` パラメータを追加
- [x] `if !has_errors` の後に GE エクスポートロジックを追加（`export_fmt == Some("ge")` のときのみ）
- [x] `out_path` のデフォルトは `"suite.json"`
- [x] `write_text_file(std::path::Path::new(out_path), &json).unwrap_or_else(...)` を呼び出す
- [x] `println!("exported GE suite to {}", out_path)` を出力

## T4: main.rs — `--export` / `--output` フラグ追加

- [x] `Some("validate") =>` アームの変数宣言に `export_fmt: Option<String>` と `output_file: Option<String>` を追加
- [x] フラグ解析ループに `"--export"` と `"--output"` アームを追加
- [x] `cmd_validate` 呼び出しに `export_fmt.as_deref()` と `output_file.as_deref()` を追加
- [x] HELP 定数に `validate ... [--export ge] [--output suite.json]` を追記

## T5: driver.rs — `v36600_tests::cargo_toml_version_is_36_6_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 36.7.0` に変更

## T6: driver.rs — `v36700_tests` モジュールを新規追加

- [x] `v36600_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（line 42825）
- [x] `v36600_tests` の閉じ `}` の後に `v36700_tests` モジュールを追加
  - [x] `use super::export_ge_suite;` インポート
  - [x] `cargo_toml_version_is_36_7_0`
  - [x] `changelog_has_v36_7_0`
  - [x] `ge_suite_export_generates_json`（6 項目 assert）

## T7: バージョン更新（T2〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.7.0` に更新（T2〜T6 すべて完了・コンパイルエラー解消の後）

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2687 passed; 0 failed — 実測: 2689 passed
- [x] `v36700_tests` の 3 テストがすべて pass
- [x] `ge_suite_export_generates_json` が pass

## T9: ドキュメント更新

- [x] `versions/v36-v40/v36.7.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.7.0（最新安定版）・v36.8.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.7.0 を完了済みにマーク（✅）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `export_ge_suite` が GE 互換 JSON を生成する | `ge_suite_export_generates_json` テスト ✅ |
| 2 | `CHANGELOG.md` に `[v36.7.0]` が含まれる | `changelog_has_v36_7_0` テスト ✅ |
| 3 | `Cargo.toml` バージョンが `36.7.0` | `cargo_toml_version_is_36_7_0` テスト ✅ |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2687） | 実測: 2689 passed, 0 failed ✅ |
