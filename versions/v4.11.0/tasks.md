# Favnir v4.11.0 タスクリスト — AWS SDK Rune & fav deploy

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.11.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.11.0` に更新

---

## Phase 1: checker.rs — `!AWS` エフェクト

- [x] `BUILTIN_EFFECTS` に `"AWS"` を追加
- [x] `require_aws_effect(&mut self, span: &Span)` を実装（E0313 エラー）
- [x] `check_builtin_apply` の `("AWS", _)` / `("S3", _)` / `("Sqs", _)` / `("Dynamo", _)` アームで `require_aws_effect` を呼ぶ
- [x] `check_test_def` の `current_effects` に `Effect::Unknown("AWS")` を追加

---

## Phase 2: `AwsConfig` 構造体と thread_local

- [x] `fav/src/backend/vm.rs` に `AwsConfig` 構造体を追加（region / endpoint_url / access_key / secret_key / session_token）
- [x] `AWS_CONFIG` thread_local（`RefCell<AwsConfig>`）を vm.rs に追加
- [x] `set_aws_config(config: AwsConfig)` 関数を追加
- [x] `AwsConfig::from_env()` — 環境変数から読み込むデフォルトコンストラクタ

---

## Phase 3: SigV4 署名実装

- [x] `sign_request(config, service, region, method, url, body, headers)` を vm.rs に実装
  - [x] Canonical Request 生成（canonical headers / signed headers / body hash）
  - [x] StringToSign 生成（日付 / scope / canonical hash）
  - [x] Signing Key 導出（HMAC-SHA256 × 4: date / region / service / "aws4_request"）
  - [x] Authorization ヘッダー生成
- [x] LocalStack 時（`endpoint_url` が設定されている場合）は署名スキップしてダミー credentials を使用

---

## Phase 4: S3 VM プリミティブ

- [x] `s3_get_object_raw(bucket, key)` — GET `/{bucket}/{key}` → `Result<String, String>`
- [x] `s3_put_object_raw(bucket, key, body)` — PUT `/{bucket}/{key}` → `Result<Unit, String>`
- [x] `s3_delete_object_raw(bucket, key)` — DELETE `/{bucket}/{key}` → `Result<Unit, String>`
- [x] `s3_list_objects_raw(bucket, prefix)` — GET `/{bucket}?list-type=2&prefix={prefix}` → `Result<List<String>, String>`（XML パース: `<Key>` 要素抽出）
- [x] `s3_head_bucket_raw(bucket)` — HEAD `/{bucket}` → `Result<Bool, String>`
- [x] `check_builtin_apply` に `("AWS", "s3_get_object_raw")` 等のアームを追加（引数型・返り値型）

---

## Phase 5: SQS VM プリミティブ

- [x] `sqs_send_message_raw(queue_url, body)` — POST + `Action=SendMessage` → `Result<String, String>`（MessageId）
- [x] `sqs_receive_messages_raw(queue_url, max)` — POST + `Action=ReceiveMessage` → `Result<List<Map<String,String>>, String>`
- [x] `sqs_delete_message_raw(queue_url, receipt_handle)` — POST + `Action=DeleteMessage` → `Result<Unit, String>`
- [x] `sqs_get_queue_url_raw(queue_name)` — POST + `Action=GetQueueUrl` → `Result<String, String>`
- [x] `check_builtin_apply` に SQS アームを追加

---

## Phase 6: DynamoDB VM プリミティブ

- [x] `dynamo_get_item_raw(table, key)` — POST `GetItem` → `Result<Option<Map<String,String>>, String>`
- [x] `dynamo_put_item_raw(table, item)` — POST `PutItem` → `Result<Unit, String>`
- [x] `dynamo_delete_item_raw(table, key)` — POST `DeleteItem` → `Result<Unit, String>`
- [x] `dynamo_query_raw(table, condition, values)` — POST `Query` → `Result<List<Map<String,String>>, String>`
- [x] `dynamo_scan_raw(table)` — POST `Scan` → `Result<List<Map<String,String>>, String>`
- [x] DynamoDB 属性変換ヘルパー: `to_dynamo_attrs(map)` / `from_dynamo_attrs(json)` — `{"S": "value"}` 形式
- [x] `check_builtin_apply` に DynamoDB アームを追加

---

## Phase 7: `fav.toml` — `[aws]` セクション

- [x] `fav/src/toml.rs` に `AwsTomlConfig` 構造体追加（region / endpoint_url / profile）
- [x] `FavToml` に `aws: Option<AwsTomlConfig>` フィールド追加
- [x] `cmd_run` / `cmd_test` で `fav.toml` の `[aws]` セクションを読み込み `set_aws_config` を呼ぶ
- [x] 優先順位: `fav.toml` > 環境変数（`AWS_REGION` / `AWS_ENDPOINT_URL` / `AWS_ACCESS_KEY_ID` 等）

---

## Phase 8: Rune ファイル

- [x] `runes/aws/s3.fav` 作成（get_object / put_object / delete_object / list_objects / bucket_exists）
- [x] `runes/aws/sqs.fav` 作成（send_message / receive_messages / delete_message / get_queue_url）
- [x] `runes/aws/dynamodb.fav` 作成（get_item / put_item / delete_item / query / scan）
- [x] `runes/aws/aws.fav` 作成（バレルモジュール: `use s3.*`, `use sqs.*`, `use dynamodb.*`）
- [x] `runes/aws/aws.test.fav` 作成（s3/sqs/dynamo の無効ホストエラーテスト 3 件）

---

## Phase 9: `fav deploy` コマンド

- [x] `fav/src/toml.rs` に `DeployConfig` / `DeployEnvConfig` 構造体追加
- [x] `FavToml` に `deploy: Option<DeployConfig>` フィールド追加
- [x] `fav/src/driver.rs` に `cmd_deploy(env, function, dry_run, region)` 実装
  - [x] Step 1: `.fav` ファイル zip 圧縮 → `/tmp/<project>-<timestamp>.zip`（`zip = "0.6"` を Cargo.toml に追加）
  - [x] Step 2: S3 に zip をアップロード（`s3_put_object_raw` 相当の直接 HTTP）
  - [x] Step 3: Lambda `UpdateFunctionCode` / `CreateFunction` API 呼び出し
  - [x] `--dry-run`: 全ステップをプリントして終了（実際の AWS 呼び出しなし）
- [x] `--dry-run` 出力フォーマット（spec の例に従う）

---

## Phase 10: CLI 配線

- [x] `fav/src/main.rs` の `Some("deploy")` アームを追加
  - [x] `--env <env>` パース（デフォルト: `"production"`）
  - [x] `--function <name>` パース（デフォルト: プロジェクト名）
  - [x] `--dry-run` フラグパース
  - [x] `--region <region>` パース（デフォルト: fav.toml の region）
- [x] HELP テキストに `deploy` コマンドを記載

---

## Phase 11: テスト

### ユニットテスト（`vm_stdlib_tests.rs` — 目標 9 件）

- [x] `aws_s3_get_object_raw_returns_err_on_bad_host`
- [x] `aws_s3_put_object_raw_returns_err_on_bad_host`
- [x] `aws_s3_list_objects_raw_returns_err_on_bad_host`
- [x] `aws_sqs_send_message_raw_returns_err_on_bad_host`
- [x] `aws_sqs_receive_messages_raw_returns_err_on_bad_host`
- [x] `aws_dynamo_get_item_raw_returns_err_on_bad_host`
- [x] `aws_dynamo_put_item_raw_returns_err_on_bad_host`
- [x] `aws_dynamo_scan_raw_returns_err_on_bad_host`
- [x] `aws_config_respects_endpoint_url`

### 統合テスト（`driver.rs` — 目標 5 件）

- [x] `aws_rune_test_file_passes` — `runes/aws/aws.test.fav` が全テスト pass
- [x] `aws_s3_get_returns_err_in_favnir_source` — Favnir ソースから S3 Err を扱える
- [x] `aws_sqs_send_returns_err_in_favnir_source` — Favnir ソースから SQS Err を扱える
- [x] `aws_dynamo_scan_returns_err_in_favnir_source` — Favnir ソースから DynamoDB Err を扱える
- [x] `deploy_dry_run_prints_steps` — `--dry-run` が全ステップを表示する

---

## 完了条件

- [x] `cargo build` が通る
- [x] 既存テスト（906 件）が全て pass
- [x] 新規テスト 14 件が pass（ユニット 9 + 統合 5）
- [x] `fav deploy --dry-run` が全ステップを表示する
- [x] `runes/aws/aws.test.fav` の 3 テストが pass
- [x] `!AWS` エフェクトなしで AWS 関数を呼ぶと E0313 が出る
- [x] LocalStack（`endpoint_url = "http://localhost:4566"`）で S3/SQS/DynamoDB の基本操作が動く

---

## 実装メモ

- **`zip` クレート**: `zip = "0.6"` を Cargo.toml に追加（`fav deploy` の zip パッケージ用）
- **SigV4 日付形式**: `%Y%m%dT%H%M%SZ`（UTC）— `chrono::Utc::now().format(...)`
- **DynamoDB エンドポイント**: `https://dynamodb.<region>.amazonaws.com/` — POST、`X-Amz-Target` ヘッダーでアクション指定
- **S3 エンドポイント**: `https://s3.<region>.amazonaws.com/<bucket>/<key>` または `https://<bucket>.s3.<region>.amazonaws.com/<key>`（パス形式を使用）
- **SQS エンドポイント**: `https://sqs.<region>.amazonaws.com/` — POST、クエリパラメータでアクション指定
- **LocalStack スキップ条件**: `config.endpoint_url.is_some()` の場合は SigV4 署名せず `Authorization: AWS4-HMAC-SHA256 Credential=test/...` のダミーを使用
- **`chrono` は既存依存**: `chrono = { version = "0.4", features = ["std"] }` — 日付フォーマットに使用
