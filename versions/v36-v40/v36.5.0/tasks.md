# v36.5.0 タスクリスト — Data Contract 規約

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v36.5.0（「Data Contract 規約」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2676（v36.4.0 完了時点の実績値））し、実測値をここに記録: 2676
- [x] Cargo.toml バージョンが `36.4.0` であることを確認
- [x] `v36400_tests::cargo_toml_version_is_36_4_0` がライブアサーション（`assert!(cargo.contains("36.4.0"), ...)`）であることを確認
- [x] `driver.rs` に `v36500_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v36.5.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `driver.rs` に `validate_contract_file` が存在しないことを確認（今回追加）
- [x] `driver.rs` に `cmd_contract_check` が存在しないことを確認（今回追加）
- [x] `driver.rs` に `create_data_contract_project` が存在しないことを確認（今回追加）
- [x] `driver.rs` に `"data-contract"` アームが `try_cmd_new` 内に存在しないことを確認（今回追加）
- [x] `main.rs` に `Some("contract")` アームが存在しないことを確認（今回追加）
- [x] `versions/current.md` の最新安定版が `v36.4.0`・次バージョンが `v36.5.0` であることを確認
- [x] `v248000_tests::template_gallery_has_4_entries` が `assert_eq!(TEMPLATE_GALLERY.len(), 4, ...)` であることを確認（v36.5.0 で 5 に更新する）
- [x] `TEMPLATE_GALLERY` が現在 4 エントリであることを確認（`data-contract` 追加で 5 エントリになる）
- [x] `Parser` が driver.rs ファイル先頭で `use crate::frontend::parser::Parser;` としてインポート済みであることを確認（`validate_contract_file` 内でのローカル use は不要）
- [x] `Item` が driver.rs モジュールスコープに **インポートされていない** ことを確認（`validate_contract_file` 内でローカル `use crate::ast::Item;` が必要）
- [x] `driver.rs` の先頭に `use std::process;` が存在することを確認（`cmd_contract_check` 内の `process::exit` で使用）

## T1: CHANGELOG.md に [v36.5.0] エントリを追加

- [x] `## [v36.4.0]` の `---` セパレータ直後に `## [v36.5.0]` エントリを挿入

## T2: driver.rs — `validate_contract_file` と `cmd_contract_check` を追加

- [x] `// ── fav validate (v36.4.0)` セクション後（`cmd_validate` の `}` の後）に `// ── fav contract check (v36.5.0)` セクションを追加
- [x] `pub fn validate_contract_file(src: &str, file: &str) -> Vec<String>` を追加
  - [x] `Parser::parse_str` エラー時はパースエラーメッセージを返す
  - [x] `Item::SchemaDef` が存在しない場合にエラーメッセージを返す
  - [x] 正常時は空 `Vec` を返す
- [x] `pub fn cmd_contract_check(dir: Option<&str>)` を追加
  - [x] デフォルトディレクトリは `"contracts"`
  - [x] ディレクトリ非存在時は `eprintln!` + `process::exit(1)`
  - [x] `.fav` ファイルが 0 件の場合は `eprintln!` + `process::exit(1)`
  - [x] `fav_files.sort()` で結果の一貫性を保証

## T3: driver.rs — `create_data_contract_project` と TEMPLATE_GALLERY 更新

- [x] `create_distributed_etl_project` の `}` の後に `fn create_data_contract_project(root: &Path, name: &str) -> Result<(), String>` を追加
  - [x] `contracts/orders.fav`（schema Orders 定義）を生成
  - [x] `fav.toml` を生成
  - [x] `README.md` を生成
- [x] `TEMPLATE_GALLERY` に `("data-contract", "Data Contract スキーマ定義プロジェクト")` を追加（5 エントリに）
- [x] `try_cmd_new` に `"data-contract" => create_data_contract_project(&root, name),` を追加（`other =>` の直前）
- [x] `try_cmd_new` の `other =>` エラーメッセージに `data-contract` を追加

## T4: main.rs — ルーティング追加

- [x] `use driver::{ ... }` に `cmd_contract_check` を追加
- [x] `Some("contract") =>` アームを追加（`Some("validate") =>` の直後）
  - [x] `Some("check")` サブコマンドを処理
  - [x] 不明サブコマンドの場合は `eprintln!` + `process::exit(1)`
- [x] HELP 定数に `contract check [dir]` の説明を追加

## T5: driver.rs — `v36400_tests::cargo_toml_version_is_36_4_0` をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.5.0` に変更

## T6: driver.rs — `v248000_tests::template_gallery_has_4_entries` を更新

- [x] `assert_eq!(TEMPLATE_GALLERY.len(), 4, ...)` → `assert_eq!(TEMPLATE_GALLERY.len(), 5, ...)` に変更
- [x] 関数冒頭に `// v36.5.0 で data-contract を追加したため 5 エントリ` コメントを追加
- [x] `assert!(names.contains(&"data-contract"), "missing data-contract");` を追加
- [x] **関数名は変えない**（`template_gallery_has_4_entries` のまま）

## T7: driver.rs — `v36500_tests` モジュールを新規追加

- [x] `v36400_tests` の閉じ `}` の後に `v36500_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_5_0`
  - [x] `changelog_has_v36_5_0`
  - [x] `data_contract_template_in_try_cmd_new`（`"data-contract"` と `create_data_contract_project` の存在確認）
  - [x] `validate_contract_file_fires`（schema なし → エラー）
  - [x] `validate_contract_file_silent`（schema あり → エラーなし）

## T8: バージョン更新（T2〜T7 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `36.5.0` に更新（T2〜T7 すべて完了・コンパイルエラー解消の後）

## T9: テスト実行

- [x] `cargo test` 全通過 — ≥（T0 実測値 + 5）passed; 0 failed（v36500_tests 5 件）— 実測: 2681 passed
- [x] `v36500_tests` の 5 テストがすべて pass
- [x] `validate_contract_file_fires` が pass（schema なしでエラーが発行されること）
- [x] `validate_contract_file_silent` が pass（schema ありでエラーなし）
- [x] `v248000_tests::template_gallery_has_4_entries` が pass（5 エントリ版として）

## T10: ドキュメント更新

- [x] `versions/v36-v40/v36.5.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v36.5.0（最新安定版）・v36.6.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v36.5.0 を完了済みにマーク
- [x] `site/content/docs/cli/contract.mdx` 作成は v36.5.0 スコープ外（ロードマップ v36.5.0 完了条件に MDX 追加の記載なし。後続バージョンで対応）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `driver.rs` に `create_data_contract_project` と `validate_contract_file` が含まれる | `data_contract_template_in_try_cmd_new` テスト |
| 2 | `CHANGELOG.md` に `[v36.5.0]` が含まれる | `changelog_has_v36_5_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.5.0` | `cargo_toml_version_is_36_5_0` テスト |
| 4 | schema なし .fav でエラーが返る | `validate_contract_file_fires` テスト |
| 5 | schema あり .fav でエラーなし | `validate_contract_file_silent` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2681） | T9 実行結果 |
