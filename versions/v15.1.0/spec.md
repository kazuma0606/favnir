# v15.1.0 Spec — CrossCloud 認証層（HMAC + Cognito + ECS）

Date: 2026-06-13

---

## 目的

v15.0.0 で実装した CrossCloud E2E Demo（AWS RDS → Favnir → Azure PostgreSQL）に認証層を追加する。

認証を通過したリクエストのみが ECS Fargate タスクを起動し `fav run` を実行する構成にすることで、
**「認証ゲートと実行エンジンの分離」**をアーキテクチャとして実証する。

リクエスト整合性は HMAC-SHA256（対称暗号・共有秘密鍵）で実装する。
共有秘密鍵の配布問題は v15.1.5 で KMS 非対称署名に置き換えて解消する。

---

## 全体アーキテクチャ

```
[caller: scripts/run_with_auth.sh]
  ① Cognito JWT 取得（Cognito User Pool テストユーザー）
  ② HMAC 署名生成（StringToSign = Method+Path+Timestamp+Nonce+SHA256(Body)）
  ③ API Gateway POST /migrate
         │
   ┌─────┴──────────────────────────────────────┐
   │  API Gateway HTTP API                       │
   │  Cognito JWT Authorizer                     │
   └─────┬──────────────────────────────────────┘
         │ (JWT 検証通過)
   ┌─────┴──────────────────────────────────────┐
   │  Lambda verifier                            │
   │  ④ タイムスタンプ検証（±5分）               │
   │  ⑤ DynamoDB nonce チェック（リプレイ防止）  │
   │  ⑥ HMAC-SHA256 署名検証                    │
   │  ⑦ proof（allow/deny）→ S3 PUT             │
   │  ⑧ ECS RunTask（認証成功時のみ）            │
   └─────┬──────────────────────────────────────┘
         │ (202 Accepted + task_arn)
   ┌─────┴──────────────────────────────────────┐
   │  ECS Fargate タスク                         │
   │  fav run --legacy migrate.fav               │
   │    ├─ Postgres.query_raw   → AWS RDS        │
   │    ├─ AzurePostgres.*      → Azure PG       │
   │    └─ AzureBlob.put_raw    → Azure Blob     │
   └─────────────────────────────────────────────┘
```

caller は `202 Accepted` を受け取ったら `aws ecs wait tasks-stopped` でタスク完了を待つ。
認証が通らない限り ECS タスクは起動しない。

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `infra/e2e-demo/crosscloud/docker/Dockerfile` | fav バイナリ + migrate.fav を含む ECS 実行イメージ |
| `infra/e2e-demo/crosscloud/terraform/aws/ecs.tf` | ECR・ECS クラスター・タスク定義・IAM タスクロール |
| `infra/e2e-demo/crosscloud/terraform/aws/auth.tf` | Cognito・API Gateway・Lambda verifier・DynamoDB nonce・HMAC Secret |
| `infra/e2e-demo/crosscloud/terraform/aws/variables.tf` | 追加変数（azure_conn_str / hmac_secret 等） |
| `infra/e2e-demo/crosscloud/terraform/aws/outputs.tf` | 追加出力（api_gateway_endpoint / ecr_repository_url 等） |
| `infra/e2e-demo/crosscloud/lambda/verifier/handler.py` | HMAC 検証 + nonce + ECS RunTask |
| `infra/e2e-demo/crosscloud/scripts/build-and-push.sh` | fav クロスコンパイル → ECR push |
| `infra/e2e-demo/crosscloud/scripts/run_with_auth.sh` | 署名生成 → API GW POST → ECS 完了待機 |
| `infra/e2e-demo/crosscloud/scripts/reject_cases.sh` | 4 ケースの拒否テスト |
| `v151000_tests`（5件） | 認証層ファイル構造・Terraform 内容の確認 |
| `Cargo.toml` バージョン `15.1.0` | |

### Out of Scope（v15.1.5 以降）

| 項目 | 理由 |
|---|---|
| Entra ID → Cognito WebIdentity federation | Azure AD テナント設定が必要（別スプリント） |
| KMS 非対称署名（ECDSA P-256） | 共有秘密鍵問題の解消は v15.1.5 |
| Azure Function からの呼び出し | スクリプトで代替（本質は同じ） |
| job_type ベースの認可 | 認証と認可の分離は v15.2.x 以降 |

---

## HMAC 署名仕様

### StringToSign

```
{HTTPMethod}\n
{Path}\n
{Timestamp（ISO8601: 2026-06-13T12:00:00Z）}\n
{Nonce（UUID v4）}\n
{SHA256(RequestBody)（hex）}
```

### 署名計算

```
Signature = HMAC-SHA256(HMAC_SECRET, StringToSign)
X-Signature: base64(Signature)
X-Timestamp: 2026-06-13T12:00:00Z
X-Nonce: <uuid-v4>
Authorization: Bearer <Cognito JWT>
```

### 検証ルール（Lambda verifier）

| チェック | 条件 | 失敗時 |
|---|---|---|
| Cognito JWT | API Gateway で事前検証済み | 401（API GW が返す） |
| タイムスタンプ | 現在時刻との差が ±5分以内 | 401 |
| nonce | DynamoDB に未存在（ConditionExpression: attribute_not_exists） | 401 |
| HMAC 署名 | 期待値と定数時間比較（hmac.compare_digest） | 401 |

---

## インフラ追加設計（AWS 側）

### `terraform/aws/ecs.tf` 新規追加

| リソース | 内容 |
|---|---|
| `aws_ecr_repository` crosscloud-fav | fav Docker イメージ格納 |
| `aws_ecs_cluster` crosscloud | ECS クラスター |
| `aws_ecs_task_definition` migrate | fav コンテナ定義（Fargate、256CPU/512MB） |
| `aws_iam_role` ecs-task-execution | ECR pull + CloudWatch logs |
| `aws_iam_role` ecs-task | RDS + Secrets Manager + Azure 接続情報アクセス |
| `aws_security_group` ecs-tasks | ECS タスク用 SG（RDS SG へのアクセス許可） |

### `terraform/aws/auth.tf` 新規追加

| リソース | 内容 |
|---|---|
| `aws_cognito_user_pool` crosscloud | JWT 発行（デモ用テストユーザー管理） |
| `aws_cognito_user_pool_client` crosscloud | `USER_PASSWORD_AUTH` フロー有効 |
| `aws_apigatewayv2_api` crosscloud | HTTP API（`POST /migrate`） |
| `aws_apigatewayv2_authorizer` cognito | Cognito JWT Authorizer |
| `aws_lambda_function` verifier | verifier handler.py |
| `aws_lambda_permission` apigw | API GW → Lambda 呼び出し許可 |
| `aws_dynamodb_table` nonce | PK=nonce_id(S)、TTL attribute=expires_at |
| `aws_secretsmanager_secret` hmac-secret | `HMAC_SECRET` 格納（favnir/crosscloud/hmac-secret） |
| `aws_secretsmanager_secret_version` hmac-secret | 初期値設定（var.hmac_secret） |

---

## `migrate.fav` の変更

**変更なし。** v15.0.0 のコードをそのまま使う。
認証層は fav パイプラインの外側にあり、ECS タスクの起動条件として機能する。

---

## 完了条件

### PASS=5（E2E）

| # | チェック | 確認方法 |
|---|---|---|
| 1 | 正常署名 → 202、ECS タスク起動 → migrate.fav 完了（exit 0） | run_with_auth.sh の終了コード |
| 2 | HMAC なし → 401（ECS タスク起動しない） | reject_cases.sh REJECT 1 |
| 3 | nonce リプレイ → 401（ECS タスク起動しない） | reject_cases.sh REJECT 3 |
| 4 | Cognito JWT なし → 401（API GW で弾く） | reject_cases.sh REJECT 4 |
| 5 | allow/deny 両方の proof が S3 に存在 | `aws s3 ls` で確認 |

### `cargo test v151000` 5件パス

---

## v15.1.5 への引き継ぎ事項

| 項目 | 内容 |
|---|---|
| 置き換え対象 | Lambda verifier の HMAC 検証部分のみ |
| `aws_secretsmanager_secret` hmac-secret | → `aws_kms_key`（ECC_NIST_P256）に置き換え |
| `handler.py` の変更 | `hmac.compare_digest` → `kms.verify` または `cryptography.ec.ECDSA` |
| caller スクリプト変更 | `openssl dgst -hmac` → `aws kms sign` |
| DynamoDB nonce / Cognito / API GW | 変更なし（そのまま流用） |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/roadmap-v15.1-v16.0.md` | v15.1.0〜v16.0.0 ロードマップ |
| `infra/e2e-demo/crosscloud/plan.md` | CrossCloud フル版設計（Phase 1 仕様） |
| `infra/e2e-demo/crosscloud/src/migrate.fav` | v15.0.0 fav パイプライン（変更なし） |
| `infra/e2e-demo/crosscloud/terraform/aws/main.tf` | 既存 AWS インフラ（RDS / S3 / Secrets Manager） |
| `infra/e2e-demo/fav2py/terraform/main.tf` | ECS / ECR Terraform パターン参考 |
