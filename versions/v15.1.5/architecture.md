# v15.1.5 アーキテクチャ — CrossCloud KMS ECDSA 認証層

Date: 2026-06-14

> このドキュメントは **AWS 側の認証層**（API Gateway + Lambda verifier_v2）のアーキテクチャです。
> Azure は ECS migrate タスク（migrate.fav）が起動後に接続する先であり、verifier_v2.fav 自身は Azure に接続しません。

---

## 全体フロー

```
[クライアント / 署名者]
    │  1. aws kms sign → DER 署名 (base64)
    │  2. Cognito IdToken 取得
    │  3. POST /migrate-kms
    │     Authorization: Bearer <IdToken>
    │     X-Timestamp / X-Nonce / X-Signature / X-KMS-Key-Id
    ▼
[API Gateway HTTP API]
    │  JWT オーソライザー（Cognito）で IdToken を検証
    │  → 認証 OK → Lambda 起動
    │  → 認証 NG → 401（API GW が返す、Lambda 不到達）
    ▼
[Lambda: favnir-crosscloud-verifier-v2-dev]
  ┌─────────────────────────────────────────────────────────────┐
  │ bootstrap (sh) — Lambda Runtime API ループ                  │
  │   API GW イベント (JSON) を解析して env var に展開:          │
  │   VERIFY_METHOD / VERIFY_PATH / VERIFY_TIMESTAMP            │
  │   VERIFY_NONCE / VERIFY_SIGNATURE / VERIFY_BODY             │
  │   VERIFY_REQUEST_ID / VERIFY_NONCE_TTL                      │
  │   VERIFY_KMS_KEY_ID  ← X-KMS-Key-Id ヘッダー               │
  │                                                             │
  │   fav run --legacy verifier_v2.fav を実行                   │
  │   exit 0 → 200 / "invalid_signature" → 401 / else → 500   │
  └────────────────┬────────────────────────────────────────────┘
                   │ fav run --legacy verifier_v2.fav
                   ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ verifier_v2.fav (Favnir, --legacy モード)                   │
  │                                                             │
  │  Step 1: env var から全フィールド読み取り                    │
  │                                                             │
  │  Step 2: KMS GetPublicKey                                   │
  │    AWS.kms_get_public_key_raw(region, kms_key_id)           │
  │    → DER (SPKI) → PEM (64 文字/行 折り返し) → String       │
  │                                                             │
  │  Step 3: ECDSA P-256 検証（ローカル）                       │
  │    StringToSign = Method\nPath\nTimestamp\nNonce\nSHA256(Body)│
  │    Crypto.ecdsa_verify_raw(pem, sts, sig_b64)               │
  │    → PEM parse → DER decode → p256::verify                  │
  │    失敗: err("ecdsa_verify_failed") → bootstrap が 401 に変換│
  │                                                             │
  │  Step 4: Nonce チェック（リプレイ防止）                     │
  │    AWS.dynamo_put_item_cond_raw(table, nonce, ttl,          │
  │      "attribute_not_exists(nonce_id)")                      │
  │    重複: err("nonce_already_used") → bootstrap が 409 に変換 │
  │                                                             │
  │  Step 5: ECS Fargate 移行タスク起動                         │
  │    containerOverrides に AZURE_STORAGE_ACCOUNT/KEY を渡す   │
  │    AWS.ecs_run_task_raw(cluster, task_def, subnets, sg,     │
  │      overrides_json)                                        │
  │    → task_arn を取得                                        │
  │                                                             │
  │  Step 6: S3 証跡保存                                        │
  │    AWS.s3_put_object_raw(bucket,                            │
  │      "auth-proof/{req_id}.json",                            │
  │      {status, request_id, task_arn})                        │
  └─────────────────────────────────────────────────────────────┘
         │ KMS API      │ DynamoDB      │ ECS RunTask  │ S3 PutObject
         ▼              ▼               ▼              ▼
      KMS key      nonce テーブル    Fargate タスク  proof バケット
  (alias/crosscloud-  (TTL 5 分)    (migrate.fav)  (auth-proof/*.json)
   signer)
                                         │
                                         ▼ 起動後に migrate.fav が接続
                                      Azure Blob Storage
                                      Azure Database (PostgreSQL)
```

---

## コンポーネント詳細

### API Gateway

- プロトコル: HTTP API (v2)
- ルート: `POST /migrate-kms`（v15.1.5 追加）/ `POST /migrate`（v15.1.0 HMAC 版）
- JWT オーソライザー: Cognito User Pool (`ap-northeast-1_aTkU4j9ez`)
- Lambda integration: payload_format_version = "2.0"

ヘッダーは API GW HTTP v2 によって**すべて小文字化**される。
例: `X-KMS-Key-Id` → `x-kms-key-id`（bootstrap の `jq` クエリはこれに合わせる）

### Lambda verifier_v2

| 項目 | 値 |
|---|---|
| 関数名 | `favnir-crosscloud-verifier-v2-dev` |
| パッケージ形式 | Container Image（ECR） |
| ベースイメージ | `public.ecr.aws/lambda/provided:al2023` |
| ランタイム | Custom Runtime（bootstrap シェルスクリプト） |
| タイムアウト | 30 秒 |
| IAM | CloudWatch Logs + DynamoDB PutItem + S3 PutObject + ECS RunTask + KMS GetPublicKey |
| 環境変数（注入）| NONCE_TABLE / ECS_CLUSTER_ARN / ECS_TASK_DEF_ARN / ECS_SUBNETS / ECS_SECURITY_GROUP / S3_PROOF_BUCKET / AZURE_STORAGE_ACCOUNT / AZURE_STORAGE_KEY |
| 環境変数（実行時）| VERIFY_* （bootstrap が API GW イベントから展開） |

v15.1.0 の verifier（HMAC 版）との違い: `HMAC_SECRET_ARN` 環境変数がなく、代わりに `VERIFY_KMS_KEY_ID` を使用する。

### KMS キー

| 項目 | 値 |
|---|---|
| エイリアス | `alias/crosscloud-signer` |
| キー仕様 | ECC_NIST_P256 |
| キー用途 | SIGN_VERIFY |
| 削除ウィンドウ | 7 日 |

署名者のみ `kms:Sign` が必要。Lambda verifier_v2 は `kms:GetPublicKey` のみ（秘密鍵に触れない）。

### verifier_v2.fav 内の Favnir Primitive

| Primitive | シグネチャ | 説明 |
|---|---|---|
| `AWS.kms_get_public_key_raw` | `(region, key_id) -> Result<String, String> !AWS` | KMS TrentService.GetPublicKey → DER → PEM 変換 |
| `Crypto.ecdsa_verify_raw` | `(pub_key_pem, message, sig_der_b64) -> Result<Unit, String> !Auth` | p256 クレートで ECDSA P-256 ローカル検証 |
| `AWS.dynamo_put_item_cond_raw` | `(table, key_attr, key_val, ttl_attr, ttl, cond) -> Result<Unit, String> !AWS` | 条件付き PutItem（nonce 重複防止） |
| `AWS.ecs_run_task_raw` | `(cluster, task_def, subnets, sg, overrides) -> Result<String, String> !AWS` | ECS Fargate RunTask → task ARN |
| `AWS.s3_put_object_raw` | `(bucket, key, body) -> Result<Unit, String> !AWS` | S3 PutObject（証跡保存） |

### StringToSign 形式

```
{METHOD}\n{PATH}\n{TIMESTAMP}\n{NONCE}\n{SHA256(BODY)}
```

例:
```
POST
/migrate-kms
2026-06-13T21:47:28Z
9a630c6d-28fb-4b81-987a-2942a7292378
3c1c3c81c45b8fbe42e74cfef2bd4e63c3fa5c7d6a19e9b8c3a70b1e0f62a9c
```

この文字列を KMS で署名し（`--message-type RAW`）、DER base64 として `X-Signature` ヘッダーに付与する。

---

## Terraform リソース一覧（v15.1.5 追加分）

| リソース | 名前 |
|---|---|
| `aws_kms_key` | `crosscloud_signer`（ECC_NIST_P256） |
| `aws_kms_alias` | `alias/crosscloud-signer` |
| `aws_ecr_repository` | `crosscloud-verifier-v2` |
| `aws_lambda_function` | `favnir-crosscloud-verifier-v2-dev` |
| `aws_lambda_permission` | `apigw_v2` |
| `aws_apigatewayv2_integration` | `verifier_v2` |
| `aws_apigatewayv2_route` | `POST /migrate-kms` |

---

## v15.1.0（HMAC）との比較

| 観点 | v15.1.0 HMAC | v15.1.5 KMS ECDSA |
|---|---|---|
| API ルート | `POST /migrate` | `POST /migrate-kms` |
| 署名方式 | HMAC-SHA256（対称） | ECDSA P-256（非対称） |
| 秘密鍵の保管場所 | Secrets Manager（Lambda と署名者が共有） | KMS 内（外に出ない） |
| Lambda が必要な権限 | `secretsmanager:GetSecretValue` | `kms:GetPublicKey` |
| 署名者が必要な権限 | Secrets Manager 読み取り（または事前共有） | `kms:Sign` |
| DER/PEM 変換 | 不要 | あり（64 文字/行 折り返し必須） |
| Favnir ファイル | `verifier.fav` | `verifier_v2.fav` |

---

## デプロイ手順（再現）

```bash
# 1. Rust バイナリビルド（--no-cache 必須）
cd fav/
docker build --no-cache -f Dockerfile.builder --tag fav-builder:latest .

# 2. バイナリ抽出
docker create --name tmp fav-builder:latest
docker cp tmp:/build/target/release/fav ../infra/e2e-demo/crosscloud/lambda/verifier_v2/fav
docker rm tmp

# 3. Terraform（ECR リポジトリのみ先行作成）
cd infra/e2e-demo/crosscloud/terraform/aws/
terraform apply -auto-approve -var="hmac_secret=<val>" \
  -var="azure_storage_account=<val>" -var="azure_storage_key=<val>" \
  -var="azure_conn_str=<val>" -var="azure_container=<val>" \
  -var="rds_password=<val>"

# 4. ECR push
aws ecr get-login-password --region ap-northeast-1 \
  | docker login --username AWS --password-stdin <ACCOUNT>.dkr.ecr.ap-northeast-1.amazonaws.com
cd infra/e2e-demo/crosscloud/lambda/verifier_v2/
docker buildx build --platform linux/amd64 --provenance=false \
  -t <ACCOUNT>.dkr.ecr.ap-northeast-1.amazonaws.com/crosscloud-verifier-v2:latest --push .

# 5. Terraform 再実行（Lambda 作成）
cd ../terraform/aws/
terraform apply -auto-approve ...（同じ変数）

# 6. Cognito ユーザー作成
aws cognito-idp admin-create-user --user-pool-id <pool_id> --username testuser \
  --temporary-password TempPass1234!
aws cognito-idp admin-set-user-password --user-pool-id <pool_id> \
  --username testuser --password TestPass1234! --permanent

# 7. E2E 検証
bash scripts/reject_kms.sh <endpoint> alias/crosscloud-signer <client_id> testuser TestPass1234!
bash scripts/run_with_kms.sh <endpoint> alias/crosscloud-signer <client_id> testuser TestPass1234!

# 8. 後片付け
terraform destroy -auto-approve \
  -var="hmac_secret=dummy" -var="azure_storage_key=dummy" \
  -var="azure_storage_account=dummy" -var="azure_conn_str=dummy" \
  -var="azure_container=dummy" -var="rds_password=dummy"
```

---

## E2E 結果（2026-06-14）

| テスト | 結果 |
|---|---|
| 改ざんボディ → 401 | PASS |
| ランダム署名（不正 DER）→ 401 | PASS |
| 正当リクエスト → 200 | PASS |
| S3 証跡保存 | `auth-proof/327c9085-cd05-481f-b6d5-83c88f6bf9dd.json` |
| ECS タスク起動 | `task/favnir-crosscloud/b942d15df51e41b5a5cb6d7d255a2b93` |
| terraform destroy | 42 resources destroyed |
