# Favnir v11.4.0 Tasks

Date: 2026-06-06
Theme: AWS エフェクト → boto3 変換

---

## Phase A — Emitter フラグ追加

- [x] A-1: `Emitter` 構造体に以下を追加
  - `needs_boto3: bool`
  - `needs_base64: bool`
  - `needs_aws_s3: bool`
  - `needs_aws_dynamo: bool`
  - `needs_aws_sqs: bool`
- [x] A-2: `Emitter::new()` で全フラグ `false` 初期化
- [x] A-3: `emit_program` の Phase 3（フラグコピー）に5フラグを追加

---

## Phase B — emit_apply 更新

- [x] B-1: S3 プリミティブの個別ケース追加（7種）
  - `s3_put_object_raw` / `s3_get_object_raw` / `s3_list_objects_raw`
  - `s3_delete_object_raw` / `s3_get_object_base64_raw` / `s3_put_bytes_raw` / `s3_head_bucket_raw`
  - 各ケースで `needs_boto3 = needs_aws_s3 = true`
- [x] B-2: DynamoDB プリミティブの個別ケース追加（5種）
  - `dynamo_scan_raw` / `dynamo_get_item_raw` / `dynamo_put_item_raw` / `dynamo_delete_item_raw` / `dynamo_query_raw`
  - 各ケースで `needs_boto3 = needs_aws_dynamo = true`
- [x] B-3: SQS プリミティブの個別ケース追加（4種）
  - `sqs_send_message_raw` / `sqs_receive_messages_raw` / `sqs_delete_message_raw` / `sqs_get_queue_url_raw`
  - 各ケースで `needs_boto3 = needs_aws_sqs = true`
- [x] B-4: フォールバック `("AWS", name)` で `needs_boto3 = true` をセット

---

## Phase C — S3 ヘルパー関数 emit

- [x] C-1: `emit_aws_s3_helpers()` メソッド追加（以下7関数）
  - `_aws_s3_put_object_raw(bucket, key, body)` — put_object + Ok/Err
  - `_aws_s3_get_object_raw(bucket, key)` — get_object + decode + Ok/Err
  - `_aws_s3_list_objects_raw(bucket, prefix)` — list_objects_v2 + Ok/Err
  - `_aws_s3_delete_object_raw(bucket, key)` — delete_object + Ok/Err
  - `_aws_s3_head_bucket_raw(bucket)` — head_bucket + Ok/Err
  - `_aws_s3_get_object_base64_raw(bucket, key)` — get_object + base64.b64encode + Ok/Err
  - `_aws_s3_put_bytes_raw(bucket, key, body_list)` — put_object(Body=bytes(body_list)) + Ok/Err

---

## Phase D — DynamoDB ヘルパー関数 emit

- [x] D-1: `emit_aws_dynamo_helpers()` メソッド追加（以下7関数）
  - `_dynamo_serialize(d)` — Python dict → DynamoDB 型付き dict
  - `_dynamo_deserialize(item)` — DynamoDB 型付き dict → Python dict
  - `_aws_dynamo_scan_raw(table)` — scan + deserialize list + Ok/Err
  - `_aws_dynamo_get_item_raw(table, key_dict)` — get_item + deserialize + Some/None + Ok/Err
  - `_aws_dynamo_put_item_raw(table, item_dict)` — serialize + put_item + Ok/Err
  - `_aws_dynamo_delete_item_raw(table, key_dict)` — serialize + delete_item + Ok/Err
  - `_aws_dynamo_query_raw(table, filter_expr)` — query + deserialize list + Ok/Err

---

## Phase E — SQS ヘルパー関数 emit

- [x] E-1: `emit_aws_sqs_helpers()` メソッド追加（4関数）
  - `_aws_sqs_send_message_raw(url, body)` — send_message + MessageId + Ok/Err
  - `_aws_sqs_receive_messages_raw(url, max_count)` — receive_message + Messages list + Ok/Err
  - `_aws_sqs_delete_message_raw(url, receipt)` — delete_message + Ok/Err
  - `_aws_sqs_get_queue_url_raw(name)` — get_queue_url + QueueUrl + Ok/Err

---

## Phase F — emit_prelude / emit_program 更新

- [x] F-1: `emit_prelude` に import 追加
  - `needs_boto3` → `import boto3`
  - `needs_base64` → `import base64 as _base64_mod`
- [x] F-2: `emit_program` Phase 7 に呼び出し追加
  - `needs_aws_s3` → `emit_aws_s3_helpers()`
  - `needs_aws_dynamo` → `emit_aws_dynamo_helpers()`
  - `needs_aws_sqs` → `emit_aws_sqs_helpers()`

---

## Phase G — pyproject.toml 生成

- [x] G-1: `driver.rs` の `cmd_transpile` に pyproject.toml 生成処理を追加
  - `.py` と同ディレクトリに生成
  - boto3 使用時は `dependencies = ["boto3>=1.34"]`
  - 既存ファイルがある場合は上書きしない
  - `generated: <path>` を stdout に出力

---

## Phase H — テスト（6件）

- [x] H-1: `v11400_tests` モジュール追加
  - [x] `transpile_aws_s3_put` — `import boto3` + `_aws_s3_put_object_raw` ヘルパー
  - [x] `transpile_aws_s3_get` — `_aws_s3_get_object_raw` ヘルパー + decode
  - [x] `transpile_aws_s3_list` — `_aws_s3_list_objects_raw` + `list_objects_v2`
  - [x] `transpile_aws_dynamo_scan` — `_dynamo_deserialize` + `_aws_dynamo_scan_raw`
  - [x] `transpile_aws_sqs_send` — `_aws_sqs_send_message_raw` ヘルパー
  - [x] `transpile_analyze_fav_aws_smoke` — boto3 コード含む + 内容アサーション
- [x] H-2: `cargo test v11400 --lib` — 6 件通過
- [x] H-3: `cargo test --lib` — 705 件通過

---

## Phase I — バージョン更新 + コミット

- [x] I-1: `fav/Cargo.toml` version → `"11.4.0"`
- [x] I-2: `cargo build` で `Cargo.lock` 更新
- [ ] I-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `AWS.s3_put_object_raw(...)` → `boto3.client("s3").put_object(...)` ヘルパー | ✅ |
| `AWS.dynamo_scan_raw(...)` → `_dynamo_deserialize` + `scan(...)` ヘルパー | ✅ |
| `AWS.sqs_send_message_raw(...)` → `send_message(...)` ヘルパー | ✅ |
| `import boto3` が自動追加される（AWS 使用時のみ） | ✅ |
| `pyproject.toml` が生成される（boto3 依存付き） | ✅ |
| `analyze.fav` → boto3 コード含む（内容アサーション通過） | ✅ |
| `cargo test v11400 --lib` 6 件通過 | ✅ |
| `cargo test --lib` 705 件通過 | ✅ |
