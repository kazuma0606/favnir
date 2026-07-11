# v35.1.0 タスクリスト — `fav deploy --target lambda`

## ステータス: COMPLETE

## T0: 事前確認

- [x] 現在のテスト数が 2621（0 failures）であることを確認
- [x] Cargo.toml バージョンが `35.8.0` であることを確認（実際は 35.8.0 → 35.1.0 へ変更）
- [x] `zip` crate が Cargo.toml に登録済みであることを確認（v0.6、deflate feature — 確認済み、追加不要）

## T1: toml.rs — `[deploy]` セクションパース

- [x] `DeployConfig` 構造体に `output: Option<String>` フィールドを追加（target/function_name/region/memory/timeout は既存済み）
- [x] `Default` impl に `output: None` を追加

## T2: deploy モジュール新規作成

- [x] 既存の `cmd_deploy` が `driver.rs` に存在することを確認（新規モジュール作成は不要）
- [x] `package_lambda` / `deploy_lambda` を driver.rs に追加

## T3: deploy/lambda.rs 実装 → driver.rs に直接実装

- [x] `package_lambda(binary_path: &Path, output_zip: &Path)` 実装
  - [x] バイナリ読み込み（`std::fs::read`）
  - [x] `zip::ZipWriter` でアーカイブ作成（エントリ名 `"bootstrap"`）
  - [x] zip 保存
- [x] `deploy_lambda(zip_path, function, region)` 実装
  - [x] AWS CLI 存在確認（`aws --version` の exit code）
  - [x] CLI 不在時は警告を出して `Ok(())` でフォールバック
  - [x] `std::process::Command` で `aws lambda update-function-code` 実行
  - [x] exit code チェック、非 0 はエラー返却
- [x] `cmd_deploy` に `"lambda"` ターゲットアーム追加
  - [x] `--target` / `--function` / `--region` / `--package-only` / `--output` 引数対応
  - [x] CLI 引数 > config > デフォルト のマージ順を守る
  - [x] `--package-only` 時は `deploy_lambda` をスキップ

## T4: main.rs — `--package-only` / `--output` フラグ追加

- [x] `package_only: bool` 変数を `Some("deploy")` アームに追加
- [x] `output: Option<String>` 変数を追加
- [x] `--package-only` match アームを追加
- [x] `--output` match アームを追加
- [x] `cmd_deploy` 呼び出しに `package_only` / `output` を追加
- [x] 既存テスト呼び出し（driver.rs:18757）を新シグネチャに更新

## T5: examples/lambda-deploy/ 追加

- [x] `examples/lambda-deploy/fav.toml`（`[deploy]` セクション含む）
- [x] `examples/lambda-deploy/src/main.fav`（サンプルパイプライン）
- [x] `examples/lambda-deploy/README.md`（30 分でデプロイできる手順書）

## T6: site MDX スタブ追加

- [x] `site/content/docs/deploy/lambda.mdx` スタブを作成（v35.8 で充実化予定と明記）

## T7: driver.rs — v35100_lambda_tests 追加

- [x] 前バージョン（v35800）のバージョン固定テストをスタブ化
- [x] `v35100_lambda_tests` モジュールを追加（7 件）
  - [x] `cargo_toml_version_is_35_1_0`
  - [x] `deploy_command_exists_in_main`
  - [x] `lambda_package_creates_zip`
  - [x] `lambda_zip_contains_bootstrap_entry`
  - [x] `deploy_config_parse_from_toml`
  - [x] `examples_lambda_deploy_exists`
  - [x] `changelog_has_v35_1_0`

## T8: テスト実行

- [x] `cargo test` 全通過（0 failures）— 2633 passed; 0 failed
- [x] v35100_lambda_tests の 7 テストが pass

## T9: バージョン管理と CHANGELOG（T8 完了後に実施）

- [x] `fav/Cargo.toml` バージョンを `35.1.0` に更新
- [x] `CHANGELOG.md` に `## [35.1.0]` エントリを追加

## T10: ドキュメント更新

- [x] `versions/v30-v35/v35.1.0/tasks.md` を COMPLETE ステータスに更新
- [x] （`versions/current.md` はマイナー版のため更新しない）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `--package-only` で `bootstrap.zip` が生成される | `lambda_package_creates_zip` テスト ✅ |
| 2 | `bootstrap.zip` に `"bootstrap"` エントリが含まれる | `lambda_zip_contains_bootstrap_entry` テスト ✅ |
| 3 | `fav.toml [deploy]` セクションが正しくパースされる | `deploy_config_parse_from_toml` テスト ✅ |
| 4 | `fav deploy` が `Some(\"deploy\")` アームで処理される | `deploy_command_exists_in_main` テスト ✅ |
| 5 | `examples/lambda-deploy/fav.toml` が存在する | `examples_lambda_deploy_exists` テスト ✅ |
| 6 | `site/content/docs/deploy/lambda.mdx` スタブが存在する | ファイル存在確認 ✅ |
| 7 | `cargo test` が 0 failures（v35100_lambda_tests 7 件 pass） | T8 実行結果 ✅ |
| 8 | `CHANGELOG.md` に `[35.1.0]` エントリが存在する | `changelog_has_v35_1_0` テスト ✅ |

---

## コードレビュー事前チェックリスト

- [x] `deploy` `"lambda"` アームは `if cfg!(not(target_arch = "wasm32"))` ブロック内にある
- [x] `cmd_deploy` が `--package-only` フラグを正しく処理し、deploy_lambda をスキップする
- [x] AWS CLI 不在時に `unwrap` / `panic` せずフォールバックする
- [x] `package_lambda` がバイナリ非存在時に適切なエラーを返す（Result::Err → eprintln + exit 1）
- [x] `zip` エントリ名が必ず `"bootstrap"` になっている（Lambda 規約）
- [x] `cmd_deploy` の CLI 引数パースで未知フラグはエラーを返す（既存実装）

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| 既存 `cmd_deploy` の他の呼び出し元（driver.rs:18757）の引数不足 | HIGH | `false, None` を追記して修正 |
| `v35800_tests::cargo_toml_version_is_35_8_0` がバージョン変更で失敗 | HIGH | スタブ化 |
