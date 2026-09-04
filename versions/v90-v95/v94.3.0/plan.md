# Plan: v94.3.0 — Lambda SnapStart 対応 Terraform

## 実装ステップ

### Step 1: `infra/lambda/sap-sync/` ディレクトリを作成する

`infra/lambda/` が存在しない場合は作成し、`sap-sync/` サブディレクトリを追加する。

### Step 2: `infra/lambda/sap-sync/main.tf` を作成する

以下の内容を含む Terraform ファイルを作成する:
- `terraform` ブロック（required_providers: aws）
- `aws_lambda_function.sap_sync` リソース（`snap_start { apply_on = "PublishedVersions" }`）
- 環境変数: `SAP_BASE_URL` / `SAP_CLIENT_ID` / `SAP_USER` / `SAP_PASS`

既存の infra パターン（`infra/sap/` 等）を参考にスタイルを統一する。

### Step 3: `infra/lambda/sap-sync/variables.tf` を作成する

以下の変数を定義する:
- `sap_base_url` (string) — SAP OData ベース URL
- `sap_client_id` (string, default = "100") — SAP クライアント ID
- `sap_user` (string, sensitive = true) — SAP ユーザー名
- `sap_pass` (string, sensitive = true) — SAP パスワード
- `lambda_role_arn` (string) — Lambda 実行 IAM ロール ARN

### Step 4: `infra/lambda/sap-sync/outputs.tf` を作成する

以下の出力値を定義する:
- `lambda_arn` — Lambda 関数 ARN
- `lambda_function_name` — Lambda 関数名

### Step 5: `fav/src/driver.rs` に `mod v94300_tests` を追加する

`mod v94200_tests { ... }` の直後に追加。

テスト 2 件:
- `lambda_sap_sync_infra_exists`: `std::path::Path::new("../infra/lambda/sap-sync").exists()` を assert
- `lambda_sap_sync_has_snap_start`: `std::fs::read_to_string("../infra/lambda/sap-sync/main.tf")` で `snap_start` の存在を確認

### Step 6: `CHANGELOG.md` に v94.3.0 エントリを追記する

先頭に v94.3.0 エントリを追加する。

### Step 7: `cargo build` でコンパイル確認

driver.rs への `mod v94300_tests` 追加がコンパイルエラーなく通ることを確認する。
（Terraform ファイル自体は Rust ビルドに影響しないが、テスト内のパス文字列や assert が壊れていないかここで検出できる）

### Step 8: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,148 tests, 0 failures を確認する。

### Step 9: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
