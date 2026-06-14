# v15.5.0 Plan — `fav deploy`

## Phase A: バージョン更新
- Cargo.toml version → "15.5.0"

## Phase B: DeployConfig 拡張（toml.rs）
- `target: String` フィールド追加（default: "aws-lambda"）
- `function_name: Option<String>` フィールド追加
- parse_fav_toml の "deploy" セクション解析に対応キー追加

## Phase C: scripts/build-lambda-layer.sh 作成
- `scripts/build-lambda-layer.sh` 新規作成
- cross-compile + bootstrap スクリプト同梱 + zip 生成

## Phase D: site/content/docs/deploy.mdx 作成
- `fav deploy` ユーザーガイド MDX 新規作成

## Phase E: v155000_tests 追加（driver.rs）
- `version_is_15_5_0`
- `deploy_toml_schema_parses`
- `deploy_cmd_exists`

## Phase F: テスト実行・コミット
- `cargo test v155000` → 3/3 PASS
- `cargo test` → リグレッションなし
- git commit
