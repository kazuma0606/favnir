# v15.1.0 完成アーキテクチャ — CrossCloud 認証層

Date: 2026-06-13

## 全体構成図

```
クライアント
    │
    │ POST /migrate
    │ Authorization: Bearer <Cognito IdToken>
    │ X-Timestamp / X-Nonce / X-Signature (HMAC-SHA256)
    │ Body: {"action":"migrate"}
    ▼
┌─────────────────────────────────────────────────────┐
│  Amazon API Gateway HTTP API                        │
│  ├── JWT Authorizer (Cognito User Pool)             │
│  │   → 無効トークン: 401 を返してLambda非呼び出し   │
│  └── POST /migrate ルート → Lambda 統合             │
└─────────────────────────────────────────────────────┘
    │ Lambda Proxy 統合 (API GW イベント形式)
    ▼
┌─────────────────────────────────────────────────────┐
│  AWS Lambda: favnir-crosscloud-verifier-dev          │
│  ├── ランタイム: Custom Runtime (provided:al2023)    │
│  ├── パッケージ: ECR コンテナイメージ                │
│  ├── メモリ: 128MB / タイムアウト: 30秒             │
│  └── bootstrap (シェルスクリプト)                   │
│       ├── Lambda Runtime API ループ                  │
│       ├── API GW イベント → 環境変数にマップ         │
│       └── fav run --legacy /var/task/verifier.fav   │
└─────────────────────────────────────────────────────┘
    │ 認証成功時
    ├──────────────────────────────────────────────────►
    │                                            ┌─────────────────┐
    │                                            │ DynamoDB        │
    │                                            │ nonce テーブル  │
    │                                            │ (TTL=5分)       │
    │                                            └─────────────────┘
    │
    ├──────────────────────────────────────────────────►
    │                                            ┌─────────────────┐
    │                                            │ ECS Fargate     │
    │                                            │ migrate タスク  │
    │                                            │ (migrate.fav)   │
    │                                            └─────────────────┘
    │
    └──────────────────────────────────────────────────►
                                                 ┌─────────────────┐
                                                 │ S3              │
                                                 │ auth-proof/     │
                                                 │ {req_id}.json   │
                                                 └─────────────────┘
```

---

## Lambda コンテナ内部構造

```
public.ecr.aws/lambda/provided:al2023 (ベースイメージ)
├── /usr/local/bin/fav          (Rust バイナリ, x86_64-glibc, ~78MB)
├── /var/runtime/bootstrap      (カスタムランタイム, シェルスクリプト)
└── /var/task/verifier.fav      (Favnir ソースコード)
```

### bootstrap フロー

```
while true; do
  1. GET /runtime/invocation/next  (Lambda Runtime API, ブロッキング)
  2. API GW イベントから各フィールドを env var に展開
     - VERIFY_METHOD, VERIFY_PATH, VERIFY_TIMESTAMP
     - VERIFY_NONCE, VERIFY_SIGNATURE, VERIFY_BODY
     - VERIFY_NONCE_TTL (= now + 300秒)
  3. fav run --legacy /var/task/verifier.fav
     - EXIT_CODE=0 → statusCode:200
     - EXIT_CODE!=0 & "invalid_signature" → statusCode:401
     - EXIT_CODE!=0 & "nonce_already_used" → statusCode:409
     - EXIT_CODE!=0 その他 → statusCode:500
  4. POST /runtime/invocation/{id}/response
done
```

---

## verifier.fav 処理フロー

```
main(ctx: AppCtx) -> Result<Unit, String> !Auth !AWS

Step 1: env var 読み込み（getenv_raw × 14本）
   method, path, ts, nonce, sig, body, req_id, nonce_ttl,
   secret_arn, nonce_tbl, cluster, task_def, subnets, sg,
   proof_bkt, stg_acct, stg_key, region

Step 2: HMAC シークレット取得
   chain hmac_secret <- AWS.secrets_get_raw(region, secret_arn)
   → Secrets Manager GetSecretValue

Step 3: HMAC-SHA256 署名検証
   bind sts <- build_sts(method, path, ts, nonce, SHA256(body))
   chain _ok <- verify_hmac(hmac_secret, sts, sig)
   → Result.err("invalid_signature") → 401

Step 4: Nonce チェック（リプレイ攻撃防止）
   chain _ok <- AWS.dynamo_put_item_cond_raw(
     table, "nonce_id", nonce, "expires_at", ttl,
     "attribute_not_exists(nonce_id)"
   )
   → ConditionalCheckFailedException → err("nonce_already_used") → 409

Step 5: ECS Fargate 移行タスク起動
   chain task_arn <- AWS.ecs_run_task_raw(
     cluster, task_def, subnets, sg, overrides_json
   )
   → RunTask API → task ARN

Step 6: S3 証跡保存
   chain _ok <- AWS.s3_put_object_raw(
     bucket, "auth-proof/{req_id}.json", proof_body
   )

Result.ok(())  → exit 0 → HTTP 200
```

---

## StringToSign（HMAC-SHA256 署名対象文字列）

```
Method\n
Path\n
Timestamp\n
Nonce\n
SHA256(Body)
```

例:
```
POST
/migrate
2026-06-13T12:00:00Z
550e8400-e29b-41d4-a716-446655440000
bf5d3affbd0b3bca0a79e3c0da1a22f48e9c53a25ba87...
```

署名: `HMAC-SHA256(secret, StringToSign)` → hex string

---

## AWS IAM 権限（Lambda ロール）

| Action | Resource |
|---|---|
| `logs:CreateLogGroup/Stream/PutEvents` | `arn:aws:logs:*:*:*` |
| `secretsmanager:GetSecretValue` | HMAC シークレット ARN のみ |
| `dynamodb:PutItem` | Nonce テーブル ARN のみ |
| `s3:PutObject` | `{proof-bucket}/auth-proof/*` |
| `ecs:RunTask` | `*` |
| `iam:PassRole` | ECS execution/task ロール ARN |

---

## Terraform リソース構成

### `terraform/aws/auth.tf`

```
aws_cognito_user_pool            favnir-crosscloud-dev
aws_cognito_user_pool_client     crosscloud-dev (USER_PASSWORD_AUTH)
aws_secretsmanager_secret        favnir/crosscloud/hmac-secret-dev
aws_secretsmanager_secret_version (HMAC 秘密鍵を保存)
aws_dynamodb_table               favnir-crosscloud-nonce-dev (TTL有効)
aws_s3_bucket                    favnir-crosscloud-proof-dev
aws_iam_role                     lambda-verifier
aws_iam_role_policy              lambda-verifier-policy
aws_ecr_repository               crosscloud-verifier
aws_lambda_function              favnir-crosscloud-verifier-dev
aws_apigatewayv2_api             favnir-crosscloud-dev (HTTP)
aws_apigatewayv2_authorizer      cognito-authorizer (JWT)
aws_apigatewayv2_integration     verifier
aws_apigatewayv2_route           POST /migrate
aws_apigatewayv2_stage           $default
aws_lambda_permission            apigw-invoke
aws_security_group_rule          egress (ECS タスク SG への ingress)
```

### `terraform/aws/ecs.tf`

```
aws_ecr_repository               crosscloud-fav
aws_ecs_cluster                  favnir-crosscloud
aws_iam_role                     ecs-execution / ecs-task
aws_cloudwatch_log_group         /ecs/favnir-crosscloud-migrate
aws_security_group               ecs-tasks
aws_ecs_task_definition          migrate (Fargate 256CPU/512MB)
aws_db_instance                  source RDS (PostgreSQL 16)
aws_db_subnet_group / sg         RDS ネットワーク設定
```

---

## vm.rs に追加した新 Primitive（v15.1.0）

| Primitive | シグネチャ | 動作 |
|---|---|---|
| `AWS.dynamo_put_item_cond_raw` | `(table, key_attr, key_val, ttl_attr, ttl_epoch, cond_expr) -> Result<Unit, String>` | DynamoDB PutItem（条件付き）、ConditionalCheckFailed → err("nonce_already_used") |
| `AWS.ecs_run_task_raw` | `(cluster, task_def, subnets_csv, sg, overrides_json) -> Result<String, String>` | ECS RunTask（SigV4）、task ARN を返す |

---

## Favnir エフェクトシステムとの対応

| エフェクト | 対象 Primitive |
|---|---|
| `!Auth` | `Crypto.sha256_raw`, `Crypto.hmac_sha256_raw` |
| `!AWS` | `AWS.secrets_get_raw`, `AWS.dynamo_put_item_cond_raw`, `AWS.ecs_run_task_raw`, `AWS.s3_put_object_raw` |
| `!IO` | `IO.println`, `IO.getenv_raw` |

`verifier.fav` の `main` シグネチャ:
```fav
public fn main(ctx: AppCtx) -> Result<Unit, String> !Auth !AWS
```

---

## E2E テスト結果（2026-06-13 実証）

### reject_cases.sh（PASS=5 FAIL=0）

| ケース | 条件 | 期待 | 結果 |
|---|---|---|---|
| 1 | 正しい HMAC 署名 | 200 | **PASS** |
| 2 | 無効な署名 | 401 | **PASS** |
| 3 | Nonce 再利用（リプレイ攻撃） | 409 | **PASS** |
| 4 | Cognito トークンなし | 401 | **PASS** |
| 5 | 無効な Cognito トークン | 401 | **PASS** |

### S3 証跡確認

```
s3://favnir-crosscloud-proof-dev/auth-proof/
├── 3e8142f8-1eb1-438e-9084-fc89c2d5344e.json (176 bytes)
└── 790cddfb-3461-4c0d-b460-1d0901df03bf.json (176 bytes)
```

証跡 JSON 形式:
```json
{"status":"ok","request_id":"<uuid>","task_arn":"<ecs-task-arn>"}
```

### ECS タスク起動確認

```
arn:aws:ecs:ap-northeast-1:847333136058:task/favnir-crosscloud/b4f67567488a4cbb8b120d3abac4582f
```
