# v36.4.0 タスクリスト — `fav validate` コマンド

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.4.0（「`fav validate` コマンド」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2671 以上）し、実測値をここに記録: 2671
- [x] Cargo.toml バージョンが `36.3.0` であることを確認
- [x] `v36300_tests::cargo_toml_version_is_36_3_0` がライブアサーション（`assert!(cargo.contains("36.3.0"), ...)`）であることを確認
- [x] `driver.rs` に `v36400_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.4.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `driver.rs` に `cmd_validate` が存在しないことを確認（今回追加）
- [x] `driver.rs` に `validate_schema_against_headers` が存在しないことを確認（今回追加）
- [x] `main.rs` に `Some("validate")` アームが存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v36.3.0`・次バージョンが `v36.4.0` であることを確認
- [x] `load_file` が `driver.rs` 内のプライベート関数（`fn load_file`）であることを確認（`cmd_validate` は必ず driver.rs に追加する）
- [x] `Parser::parse_str` のシグネチャを確認（`parse_str(src: &str, file: &str) -> Result<Program, ...>`）
- [x] `Parser` が driver.rs ファイル先頭で `use crate::frontend::parser::Parser;` としてインポート済みであることを確認（`cmd_validate` 内でのローカル use は不要）
- [x] 既存テストモジュール（`v36300_tests`）が `super::` パスを使っているか `crate::driver::` パスを使っているかを確認し、`v36400_tests` のテストコードと統一する

## T1: CHANGELOG.md に [v36.4.0] エントリを追加

- [x] `## [v36.3.0]` の `---` セパレータ直後に `## [v36.4.0]` エントリを挿入

## T2: driver.rs — `cmd_validate` 実装

- [x] `validate_schema_against_headers(schema_field_names: &[String], csv_headers: &[String]) -> Vec<String>` を `pub fn` として追加
- [x] `read_csv_headers(path: &str) -> Vec<String>` をプライベートヘルパーとして追加
- [x] `cmd_validate(schema_file: Option<&str>, data_file: Option<&str>)` を `pub fn` として追加
- [x] `cmd_validate` 内で `SchemaDef` を収集し CSV ヘッダーと照合することを確認
- [x] 欠損カラムがある場合に `eprintln!` でエラーを出力し `process::exit(1)` することを確認

## T3: main.rs — ルーティング追加

- [x] `use driver::{ ... }` ブロックに `cmd_validate` を追加
- [x] `Some("validate") =>` アームを追加（`Some("lint") =>` の直後）
- [x] `HELP` 定数に `validate` コマンドの説明行を追加（例: `  validate    Validate CSV data against a schema definition`）

## T4: driver.rs — v36300_tests::cargo_toml_version_is_36_3_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.4.0` に変更

## T5: driver.rs — v36400_tests モジュールを新規追加

- [x] driver.rs ファイル末尾（`v36300_tests` モジュールの閉じ `}` の後）に `v36400_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_4_0`
  - [x] `changelog_has_v36_4_0`
  - [x] `cmd_validate_in_driver_rs`
  - [x] `validate_missing_column_reported`
  - [x] `validate_all_columns_present_ok`

## T6: バージョン更新（T2・T3・T4・T5 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.4.0` に更新（T2〜T5 すべて完了・コンパイルエラー解消の後）

## T7: テスト実行

- [x] `cargo test` 全通過 — ≥（T0 実測値 + 5）passed; 0 failed（v36400_tests 5 件）— 実測: 2676 passed
- [x] `v36400_tests` の 5 テストがすべて pass
- [x] `validate_missing_column_reported` が pass（欠損カラムでエラーが発行されること）
- [x] `validate_all_columns_present_ok` が pass（全カラム揃いでエラーなし）

## T8: ドキュメント更新

- [x] `versions/v36-v40/v36.4.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.4.0（最新安定版）・v36.5.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.4.0 を完了済みにマーク
- [x] `site/content/docs/cli/validate.mdx` の作成は v36.4.0 スコープ外（ロードマップ v36.4.0 完了条件に MDX 追加の記載なし。v36.5.0 以降で対応）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `driver.rs` に `cmd_validate` と `validate_schema_against_headers` が含まれる | `cmd_validate_in_driver_rs` テスト |
| 2 | `CHANGELOG.md` に `[v36.4.0]` が含まれる | `changelog_has_v36_4_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.4.0` | `cargo_toml_version_is_36_4_0` テスト |
| 4 | 欠損カラムがあるとエラーが報告される | `validate_missing_column_reported` テスト |
| 5 | 全カラムが揃うとエラーなし | `validate_all_columns_present_ok` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2676） | T7 実行結果 |
