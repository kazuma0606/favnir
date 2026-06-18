# Roadmap v15.1.0 〜 v16.0.0 — Production Multi-Cloud

Date: 2026-06-13

## 目標

v15.0.0 で完成した CrossCloud E2E Demo（簡略版）を出発点として、
認証層（Entra ID / Cognito / Lambda verifier）、新規クラウドデータソース（GCP BigQuery、Kafka / MSK）、
開発ツール強化（`fav test`、`fav deploy`）を段階的に実装し、
v16.0.0 で「Production Multi-Cloud」マイルストーンを宣言する。

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| CrossCloud 認証アーキテクチャ | Entra ID → Cognito Web Identity Federation → STS AssumeRoleWithWebIdentity（Cognito 直接信頼方式。API Gateway + Lambda verifier でゲート） |
| HMAC 整合性 | リクエストボディハッシュ + タイムスタンプ + nonce。nonce は DynamoDB テーブルで TTL 管理 |
| Kafka ターゲット | AWS MSK（Managed Streaming for Apache Kafka）。`rdkafka` crate（librdkafka ラッパー）で接続。IAM 認証または SASL/SCRAM |
| BigQuery 認証 | Service Account JSON キー（`google-auth` / `ureq` + OAuth2 token exchange）。既存 SigV4 実装パターンを踏襲 |
| `fav test` 実行モデル | 単一 `.fav` ファイル内に `test "..."` ブロックを書き、`fav test <file>` で実行。Mock 対応は `Ctx.mock` 流用 |
| `fav deploy` ターゲット v1 | AWS Lambda のみ（zip + `aws lambda update-function-code`）。Azure Function は v16.x 以降 |

---

## バージョン計画

### v15.1.0 — CrossCloud 認証層 基礎版（HMAC + Cognito）

**テーマ**: `infra/e2e-demo/crosscloud/plan.md` Phase 1 の実装。
v15.0.0 で省略した Entra ID / Cognito / Lambda verifier を追加し、
**Lambda が認証を通過したリクエストのみ ECS Fargate タスクを起動して fav を実行する**構成にする。
リクエスト整合性は **HMAC-SHA256 + 共有秘密鍵**（シンプルな対称暗号方式）で実装する。

**全体アーキテクチャ:**

```
[caller: scripts/run_with_auth.sh]
  ① Entra ID token 取得
  ② Cognito WebIdentity Exchange → AWS 一時認証情報
  ③ HMAC 署名生成
  ④ API Gateway POST /migrate
         |
   Cognito JWT Authorizer（token 検証）
         |
   Lambda verifier
     ⑤ タイムスタンプ検証（±5分）
     ⑥ DynamoDB nonce チェック（リプレイ防止）
     ⑦ HMAC-SHA256 署名検証
     ⑧ proof（allow/deny）→ S3 PUT
     ⑨ ECS RunTask（fav コンテナ起動）← 認証成功時のみ
            |
     ECS Fargate タスク
       fav run --legacy migrate.fav
         ├─ Postgres.query_raw     → AWS RDS
         ├─ AzurePostgres.execute_raw → Azure PostgreSQL
         └─ AzureBlob.put_raw      → Azure Blob
```

caller は API Gateway に POST するだけで、fav の実行はクラウド側に委ねられる。
認証が通らない限り ECS タスクは起動しない。

**HMAC 署名方式:**

```
StringToSign =
  HTTPMethod + "\n" +
  Path + "\n" +
  Timestamp（ISO8601）+ "\n" +
  Nonce（UUID v4）+ "\n" +
  SHA256(RequestBody)

Signature = HMAC-SHA256(HMAC_SECRET, StringToSign)
X-Signature: <base64(Signature)>
X-Timestamp: <ISO8601>
X-Nonce: <UUID v4>
```

`HMAC_SECRET`（32バイト以上のランダム文字列）は **AWS Secrets Manager** に保存。

**実装内容:**

- `infra/e2e-demo/crosscloud/docker/`:
  - `Dockerfile`: `fav` バイナリ（x86_64-unknown-linux-musl）+ `migrate.fav` を含む軽量イメージ
  - ベース: `debian:bookworm-slim`
  - エントリポイント: `fav run --legacy /app/migrate.fav`
  - 接続情報は ECS タスク定義の環境変数から取得（`DATABASE_URL` / `AZURE_CONN_STR` 等）

- `infra/e2e-demo/crosscloud/scripts/build-and-push.sh`:
  - `cargo build --release --target x86_64-unknown-linux-musl`
  - `docker build` + `docker push` → ECR

- `infra/e2e-demo/crosscloud/terraform/aws/` 追加:
  - `aws_ecr_repository` crosscloud-fav
  - `aws_ecs_cluster` crosscloud
  - `aws_ecs_task_definition` migrate（ECR イメージ参照、環境変数、IAM タスクロール）
  - `aws_iam_role` ecs-task-role（RDS アクセス + Secrets Manager + AzureBlob へのアクセス）
  - `aws_cognito_user_pool` + `aws_cognito_identity_pool`（Entra ID OIDC プロバイダー）
  - `aws_apigatewayv2_api` + `aws_apigatewayv2_authorizer`（Cognito JWT Authorizer）
  - `aws_lambda_function` verifier
  - `aws_dynamodb_table` nonce（TTL = 300秒）
  - `aws_secretsmanager_secret` hmac-secret
  - Lambda IAM ロールに `ecs:RunTask` + `iam:PassRole`（ECS タスクロール用）権限追加

- `infra/e2e-demo/crosscloud/lambda/verifier/`:
  - `handler.py`:
    1. `X-Timestamp` → ±5分チェック
    2. `X-Nonce` → DynamoDB `put_item`（ConditionExpression: attribute_not_exists）でリプレイ検出
    3. Secrets Manager から `HMAC_SECRET` 取得（Lambda 起動時にキャッシュ）
    4. HMAC-SHA256 再計算 → `X-Signature` と比較
    5. deny の場合 → S3 に proof JSON PUT して 401 返却
    6. allow の場合 → `ecs.run_task(...)` 呼び出し → 202 Accepted 返却
  - `requirements.txt`: `boto3`（Lambda runtime に含まれるが明示）

- `infra/e2e-demo/crosscloud/scripts/run_with_auth.sh`:
  - Entra ID token → Cognito WebIdentity Exchange → STS AssumeRoleWithWebIdentity
  - Secrets Manager から `HMAC_SECRET` を取得
  - `StringToSign` 構築 + HMAC-SHA256 署名
  - API Gateway に署名付き POST → `202 Accepted` を確認
  - ECS タスクの完了を `aws ecs wait tasks-stopped` でポーリング待機
  - タスク終了コードを確認（0 = 成功）

- `infra/e2e-demo/crosscloud/scripts/reject_cases.sh`:
  - `[REJECT 1]` HMAC secret なし（X-Signature ヘッダーなし） → 401
  - `[REJECT 2]` 期限切れタイムスタンプ → 401
  - `[REJECT 3]` nonce リプレイ（同じ nonce を 2 回送信） → 401
  - `[REJECT 4]` Cognito トークンなし（直接 API 呼び出し） → 401

- テスト: `v151000_tests`:
  - `version_is_15_1_0`
  - `crosscloud_auth_structure`（lambda/verifier/ + docker/ + scripts/run_with_auth.sh + scripts/reject_cases.sh）
  - `crosscloud_terraform_has_cognito`（aws/main.tf に `aws_cognito` が含まれる）
  - `crosscloud_terraform_has_ecs`（aws/main.tf に `aws_ecs_cluster` が含まれる）
  - `crosscloud_terraform_has_dynamodb_nonce`（aws/main.tf に `aws_dynamodb_table` が含まれる）

**完了条件（PASS=5）:**
1. valid signed request → 202 Accepted、ECS タスク起動 → migrate.fav 完了（exit 0）
2. HMAC なし → 401（ECS タスク起動しない）
3. nonce リプレイ → 401（ECS タスク起動しない）
4. Cognito トークンなし → 401（ECS タスク起動しない）
5. allow/deny 両方の proof が S3 に保存されている

**既知の制約（v15.1.5 で解消予定）:**
- `HMAC_SECRET` は Azure Function 側と AWS 側の両方が知る必要がある（共有秘密鍵の配布問題）
- 秘密鍵が漏洩すると攻撃者が任意の正規署名を生成できる
- 鍵ローテーション時は両側を同時に更新する必要がある

---

### v15.1.5 — CrossCloud 認証層 セキュア版（KMS 非対称署名）

**テーマ**: v15.1.0 の HMAC（対称暗号）を AWS KMS の非対称署名（ECDSA P-256）に置き換える。
Favnir の機能拡充というより**クロスクラウド認証アーキテクチャの学習・比較**が主目的。

**HMAC との違い:**

| 項目 | v15.1.0 HMAC | v15.1.5 KMS 非対称署名 |
|---|---|---|
| 秘密鍵の所在 | Secrets Manager（両側が知る） | KMS 内（外に出ない） |
| 検証側が必要なもの | 秘密鍵（共有） | 公開鍵のみ（平文 OK） |
| 署名偽造のリスク | 秘密鍵漏洩で偽造可能 | 秘密鍵は KMS の外に出ないため不可 |
| 鍵ローテーション | 両側同時更新が必要 | KMS 側だけ。公開鍵の再取得のみ |
| Azure → AWS の権限 | Secrets Manager `GetSecretValue` | KMS `Sign`（+ `AssumeRoleWithWebIdentity`） |
| 実装複雑度 | 低 | 中（KMS API 呼び出し + OIDC 信頼設定） |

**署名フロー:**

```
[Azure Function]
  1. Entra ID token 取得
  2. Cognito WebIdentity Exchange → AWS 一時認証情報
  3. AWS 一時認証情報で KMS Sign API 呼び出し
     → kms:Sign(key_id, StringToSign, ECDSA_SHA_256)
     → 秘密鍵は KMS の外に出ない
  4. 署名 + 公開鍵識別子を X-Signature / X-KMS-Key-Id ヘッダーに付与

[Lambda verifier]
  5. X-KMS-Key-Id から kms:GetPublicKey で公開鍵（PEM）取得（or キャッシュ）
  6. ECDSA 署名をローカル検証（Python `cryptography` ライブラリ）
  7. タイムスタンプ・nonce・Cognito JWT は v15.1.0 と同じ
```

**実装内容:**

- `infra/e2e-demo/crosscloud/terraform/aws/` 追加:
  - `aws_kms_key`（非対称、`key_usage = "SIGN_VERIFY"`, `customer_master_key_spec = "ECC_NIST_P256"`）
  - `aws_kms_alias` crosscloud-signer
  - Lambda IAM ロールに `kms:GetPublicKey` 権限追加
  - Azure Function が使う IAM ロールに `kms:Sign` 権限追加
  - `aws_secretsmanager_secret` hmac-secret は **削除**（不要になる）

- `infra/e2e-demo/crosscloud/lambda/verifier_v2/`:
  - `handler.py`（v15.1.5 版）: KMS 公開鍵取得 + ECDSA 検証
  - `boto3` で `kms.get_public_key()` → DER → PEM 変換
  - `cryptography` ライブラリで `ec.ECDSA(hashes.SHA256())` 検証

- `infra/e2e-demo/crosscloud/scripts/run_with_kms.sh`:
  - `aws kms sign --key-id alias/crosscloud-signer --signing-algorithm ECDSA_SHA_256 --message-type RAW --message <StringToSign>`
  - 署名を base64 エンコードしてヘッダーに付与

- `infra/e2e-demo/crosscloud/docs/auth-comparison.md`:
  - HMAC vs KMS 非対称署名の比較表
  - それぞれのユースケース・トレードオフ
  - 「どちらを本番で使うべきか」のガイダンス

- テスト: `v15150_tests`:
  - `version_is_15_1_5`
  - `crosscloud_kms_terraform_has_ecc_key`（main.tf に `ECC_NIST_P256` が含まれる）
  - `crosscloud_verifier_v2_exists`（lambda/verifier_v2/ の存在確認）
  - `crosscloud_auth_comparison_doc_exists`（docs/auth-comparison.md の存在確認）

**完了条件（比較デモ PASS=2）:**
1. `run_with_kms.sh` で valid request → accepted（migrate.fav 実行）
2. 改ざんリクエスト（body を変更）→ 401 rejected（ECDSA 検証失敗）

---

### v15.2.0 — GCP BigQuery Rune（`!Gcp` エフェクト）

**テーマ**: Snowflake 統合（v10.x）と同じパターンで GCP BigQuery を追加する。
AWS/Azure に続く 3 クラウド目のデータソースサポート。

**実装内容:**

- `fav/src/backend/vm.rs`:
  - `BigQuery.query_raw(project_id, dataset, sql, params) -> Result<String, String>` プリミティブ
  - `BigQuery.execute_raw(project_id, dataset, sql, params) -> Result<Int, String>` プリミティブ
  - 認証: Service Account JSON キー → OAuth2 Bearer token（`ureq` + Google token endpoint）
  - 接続情報は `GOOGLE_APPLICATION_CREDENTIALS` 環境変数（JSON キーファイルパス）または `GCP_PROJECT_ID` から取得

- `fav/src/middle/checker.rs`:
  - `Effect::Gcp` 追加
  - `builtin_ret_ty` に `BigQuery.*` 追加
  - E0318: `!Gcp` エフェクトなし呼び出しエラー

- `fav/src/ast.rs`: `Effect::Gcp` 追加

- `fav/src/lineage.rs`: `EffectKind::GcpRead` / `GcpWrite` 追加

- checker.fav 更新:
  - `bigquery_fn` スキーム追加
  - `ns_to_effect` に `"BigQuery"` → `"Gcp"` 追加
  - `builtin_ret_ty` に `BigQuery.*` 追加

- `runes/bigquery/bigquery.fav`: `query<T>` / `execute` ラッパー

- `fav.toml [gcp]` セクション: `project_id` / `credentials_file` / `dataset`

- `fav infer --from bigquery --table <table>`:
  - `BigQuery.infer_table_raw` プリミティブ（INFORMATION_SCHEMA クエリ）
  - BigQuery 型 → Favnir 型マッピング（STRING/INT64/FLOAT64/BOOL/TIMESTAMP）

- `infra/e2e-demo/bigquery/`:
  - `src/demo.fav`: GCS → BigQuery → Snowflake の 3 クラウド横断デモ
  - `terraform/gcp/main.tf`: BigQuery dataset + GCS bucket
  - `scripts/seed.sh` / `scripts/run.sh` / `scripts/verify.sh`

- 新規 Cargo 依存:
  - なし（`ureq` + `serde_json` で既存依存内に収める）

- テスト: `v152000_tests`（5件）

---

### v15.3.0 — `fav test` DSL（ネイティブテストフレームワーク）

**テーマ**: Favnir ファイル内に `test "..."` ブロックを書けるようにし、
`fav test <file>` で実行・レポートできるようにする。
現在 Rust `#[test]` で書いているパイプライン検証を Favnir ネイティブに移行する。

**構文:**

```fav
test "transform trims whitespace" {
  let row = { full_name: "  Alice  ", email: "a@example.com" }
  let result = transform_row(row)
  assert_eq(result.normalized_name, "Alice")
}

test "extract returns ok with mock db" {
  chain rows <- extract_from_rds()
  assert_ok(rows)
}
```

**実装内容:**

- `fav/src/frontend/parser.rs`:
  - `test "description" { ... }` 構文追加（TopLevel::TestDef）
  - `assert_eq(a, b)` / `assert_ok(r)` / `assert_err(r)` / `assert_true(b)` をキーワードとして認識

- `fav/src/ast.rs`:
  - `TopLevel::TestDef { name: String, body: Vec<Stmt> }` 追加

- `fav/src/middle/compiler.rs`:
  - `TestDef` のコンパイル: テストごとに独立した関数として IR 生成
  - `assert_eq` / `assert_ok` / `assert_err` を IR プリミティブとして実装

- `fav/src/backend/vm.rs`:
  - `AssertEq` / `AssertOk` / `AssertErr` / `AssertTrue` opcode 追加
  - 失敗時: `TestFailure { test_name, message }` エラー型

- `fav/src/driver.rs`:
  - `cmd_test(path: &str)` 実装
  - テスト収集 → 順次実行 → PASS/FAIL 集計 → `cargo test` スタイルの出力

- `fav/src/cli.fav`（Favnir 側）:
  - `cmd_test` ハンドラ追加

- `site/content/docs/language/testing.mdx` 新規作成

- テスト: `v153000_tests`（5件）

---

### v15.4.0 — Kafka / MSK Rune（`!Stream` エフェクト）

**テーマ**: AWS MSK（Managed Streaming for Apache Kafka）を Favnir から操作できるようにする。
`!Stream` エフェクトを追加し、CDC・ストリーミングパイプラインの基礎を作る。

**実装内容:**

- `fav/src/backend/vm.rs`:
  - `Kafka.produce_raw(brokers, topic, key, value) -> Result<Unit, String>` プリミティブ
  - `Kafka.consume_one_raw(brokers, topic, group_id) -> Result<String, String>` プリミティブ
    （バッチ消費・オフセット管理は v15.5.x 以降）
  - 認証: SASL/SCRAM（`sasl_username` / `sasl_password`）または IAM（MSK IAM プラグイン）
  - 接続情報: `KAFKA_BOOTSTRAP_BROKERS` / `KAFKA_SASL_USERNAME` / `KAFKA_SASL_PASSWORD` 環境変数

- 新規 Cargo 依存:
  - `rdkafka = { version = "0.36", features = ["cmake-build", "sasl", "ssl"] }`
    （librdkafka の Rust ラッパー。MSK の SASL/SCRAM + TLS をサポート）

- `fav/src/middle/checker.rs`:
  - `Effect::Stream` 追加
  - `builtin_ret_ty` に `Kafka.*` 追加
  - E0319: `!Stream` エフェクトなし呼び出しエラー

- `fav/src/ast.rs`: `Effect::Stream` 追加

- `fav/src/lineage.rs`: `EffectKind::StreamRead` / `StreamWrite` 追加

- checker.fav 更新:
  - `kafka_fn` スキーム追加
  - `ns_to_effect` に `"Kafka"` → `"Stream"` 追加

- `runes/kafka/kafka.fav`: `produce<T>` / `consume_one` ラッパー

- `fav.toml [kafka]` セクション: `bootstrap_brokers` / `sasl_mechanism` / `sasl_username` / `sasl_password`

- `infra/e2e-demo/kafka/`:
  - `src/pipeline.fav`: RDS → Kafka → Azure Postgres の CDC 的デモ（1000行バッチ）
  - `terraform/aws/main.tf`: `aws_msk_cluster`（`kafka_version = "3.6.0"`, `instance_type = "kafka.t3.small"`）
  - `scripts/seed.sh` / `scripts/run.sh` / `scripts/verify.sh`
  - `README.md`

- テスト: `v154000_tests`（5件）

---

### v15.5.0 — `fav deploy`（AWS Lambda デプロイ CLI）

**テーマ**: `.fav` パイプラインを AWS Lambda として直接デプロイできるようにする。
「書いたパイプラインをそのままクラウドで動かせる」体験を実現する。

**実装内容:**

- `fav/src/driver.rs`:
  - `cmd_deploy(config: &FavToml)` 実装
  - 手順: `fav build` → zip 生成 → `aws lambda create-function` or `update-function-code`

- `fav.toml [deploy]` セクション:
  ```toml
  [deploy]
  target = "aws-lambda"          # "aws-lambda" | "azure-function"（後者は v16.x）
  function_name = "my-pipeline"
  role_arn = "arn:aws:iam::..."
  runtime = "provided.al2023"    # custom runtime（fav バイナリ同梱）
  region = "ap-northeast-1"
  memory_mb = 512
  timeout_sec = 300
  ```

- `fav/src/cli.fav`（Favnir 側）:
  - `cmd_deploy` ハンドラ追加

- `scripts/build-lambda-layer.sh`:
  - `fav` バイナリを Lambda 用に cross-compile（`x86_64-unknown-linux-musl`）
  - zip にパッケージング

- `fav deploy --dry-run`:
  - 生成 zip の内容をリスト表示（実デプロイなし）

- サイト: `site/content/docs/deploy.mdx` 新規作成

- テスト: `v155000_tests`（3件）:
  - `version_is_15_5_0`
  - `deploy_toml_schema_parses`（`[deploy]` セクション解析）
  - `deploy_cmd_exists`（`fav --help` に `deploy` が含まれる）

---

### v16.0.0 — "Production Multi-Cloud" マイルストーン宣言

**テーマ**: v15.x シリーズの集大成。多クラウド対応・認証・テスト・デプロイが揃ったことを宣言。

**実装内容:**

- `CHANGELOG.md`: v15.1.0〜v15.5.0 の全エントリ追加

- `README.md`:
  - 「現在の状態」を v16.0.0 に更新
  - 対応クラウド一覧表（AWS/Azure/GCP/Snowflake + Kafka/MSK）
  - `fav test` / `fav deploy` を機能一覧に追加

- `site/content/docs/`:
  - `runes/bigquery.mdx` 新規作成
  - `runes/kafka.mdx` 新規作成
  - `deploy.mdx`（v15.5.0 作成済み）最終更新
  - `language/testing.mdx`（v15.3.0 作成済み）最終更新

- テスト: `v160000_tests`（5件）:
  - `version_is_16_0_0`
  - `changelog_has_v15_5_0_entry`
  - `readme_mentions_bigquery`
  - `readme_mentions_kafka`
  - `all_e2e_demo_dirs_exist`（airgap / fav2py / snowflake / crosscloud / bigquery / kafka の 6 件）

---

## 依存関係

```
v15.0.0（CrossCloud E2E 簡略版）✅
    |
    v15.1.0（CrossCloud 認証層 HMAC 基礎版）
    |
    v15.1.5（CrossCloud 認証層 KMS セキュア版）   ← 主に学習目的
    |
    v15.2.0（GCP BigQuery）   v15.3.0（fav test DSL）  ← 並列実施可能
    |                          |
    v15.4.0（Kafka / MSK）
    |
    v15.5.0（fav deploy）
    |
    v16.0.0（マイルストーン）
```

v15.1.5 は v15.1.0 の発展版（インフラのみ、Favnir コード変更なし）。
v15.2.0 と v15.3.0 は独立しているため並列実施可能。
v15.1.0 は v15.0.0 の直接の続き（plan.md Phase 1）で最優先。

---

## 新規 Cargo 依存（予定）

| Crate | 用途 | 追加バージョン |
|---|---|---|
| `rdkafka 0.36` | Kafka / MSK クライアント（librdkafka ラッパー） | v15.4.0 |
| `zip 2.x` | Lambda デプロイ zip 生成 | v15.5.0 |
| その他 GCP/BigQuery 関連 | OAuth2 token exchange のみ `ureq` 流用で対応 | v15.2.0 |

---

## 実装ノート

- **実行フロー**: caller → API Gateway → Lambda verifier（認証）→ ECS RunTask → fav。caller は 202 を受け取ったら `ecs wait tasks-stopped` でポーリング。fav の exit code をタスク終了コードとして確認。
- **ECS タスクの環境変数渡し**: `DATABASE_URL` / `AZURE_CONN_STR` / `AZURE_STORAGE_ACCOUNT` / `AZURE_STORAGE_KEY` は ECS タスク定義の `secrets`（Secrets Manager 参照）または `environment` で渡す。Lambda から `overrides.containerOverrides.environment` で動的に渡すことも可能。
- **ECR イメージのビルド**: `fav` バイナリは `x86_64-unknown-linux-musl` でクロスコンパイル（Windows 環境では `cross` crate または Docker buildx を使用）。
- **Entra ID → Cognito 連携方式**: `sts:AssumeRoleWithWebIdentity` を使う「Azure AD アプリ登録 + AWS IAM OIDC 直接信頼」方式。Cognito Identity Pool 経由より簡潔。
- **MSK IAM 認証 vs SASL/SCRAM**: MSK Serverless は IAM のみ。MSK Provisioned は SASL/SCRAM 推奨（`rdkafka` の SASL サポートが安定しているため）。
- **`fav deploy` の Lambda runtime**: `provided.al2023`（custom runtime）で `fav` バイナリを直接実行。`bootstrap` スクリプトが `fav run --legacy pipeline.fav` を呼ぶ形。
- **`fav test` と `fav run` の分離**: `test "..."` ブロックは `fav run` では無視される（TopLevel として実行対象から除外）。
- **BigQuery 認証の Windows 開発環境**: `GOOGLE_APPLICATION_CREDENTIALS` に JSON キーファイルのフルパスを指定。Windows パス対応（バックスラッシュ）が必要になる場合は `canonicalize()` で正規化。
- **v15.1.0 コスト注意**: MSK の Provisioned クラスターは常時課金。v15.4.0 E2E 完了後は必ず `terraform destroy` を実施すること。
- **rdkafka の cmake-build**: Windows 環境では `cmake` が必要。CI（Linux）では問題なし。

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `infra/e2e-demo/crosscloud/plan.md` | CrossCloud フル版設計（v15.1.0 の Phase 1 仕様） |
| `versions/v15.0.0/tasks.md` | v15.1.0 積み残し候補（末尾セクション） |
| `versions/roadmap-v14.1-v15.0.md` | 直前ロードマップ（形式参照） |
| `infra/e2e-demo/airgap/` | Airgap E2E パターン参考 |
| `infra/e2e-demo/fav2py/` | fav2py E2E パターン参考 |
| `infra/e2e-demo/snowflake/` | Snowflake E2E パターン参考（rdkafka と同系統の外部サービス接続） |
