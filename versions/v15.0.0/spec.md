# v15.0.0 Spec — CrossCloud E2E Demo（簡略版）

Date: 2026-06-12

---

## 目的

AWS RDS PostgreSQL → Favnir パイプライン → Azure DB for PostgreSQL のクロスクラウドマイグレーションを
E2E デモとして動作させる。

**v15.0.0 は簡略版**。crosscloud/plan.md 記載の Entra ID / Cognito 連携・HMAC 検証・Lambda verifier・
認可フローは v15.1.0 以降に持ち越し。本バージョンはパイプライン本体と型安全性の実証に集中する。

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `infra/e2e-demo/crosscloud/src/migrate.fav` | 5 ステージ Favnir パイプライン |
| `infra/e2e-demo/crosscloud/terraform/aws/` | RDS PostgreSQL + S3 proof + IAM |
| `infra/e2e-demo/crosscloud/terraform/azure/` | Azure DB for PostgreSQL + Storage Account + Blob |
| `infra/e2e-demo/crosscloud/scripts/seed.sh` | AWS RDS にサンプルデータ 1000 行投入 |
| `infra/e2e-demo/crosscloud/scripts/run.sh` | 移行実行スクリプト |
| `infra/e2e-demo/crosscloud/scripts/verify.sh` | 行数・証跡確認スクリプト |
| `infra/e2e-demo/crosscloud/README.md` | 更新（簡略版スコープ明記） |
| `v150000_tests` (5 件) | Favnir コード・構造検証 |
| `Cargo.toml` バージョン `15.0.0` | |

### Out of Scope（v15.1.0 以降）

| 項目 | 理由 |
|---|---|
| Entra ID → Cognito 連携 | 設定量が多い（Cognito federation endpoint, claims mapping） |
| AWS API Gateway + Lambda verifier | HMAC 検証・ nonce 管理が別スプリント級 |
| Azure AD アプリ登録 + AWS IAM OIDC 直接信頼 | v15.1.0 で Cognito 経由より簡潔な代替として検討 |
| 認可（job_type フィルタリング） | Lambda verifier に依存 |
| 冪等性保証（upsert / nonce テーブル） | DynamoDB nonce table が必要 |
| 双方向 Azure → AWS レプリケーション | out-of-scope（plan.md 明記） |

---

## パイプライン設計（5 ステージ）

### データモデル

```fav
// Source（AWS RDS PostgreSQL）
type CustomerRow = {
  customer_id: String
  email: String
  full_name: String
  status: String
  updated_at: String
}

// Target（Azure DB for PostgreSQL）
type MigratedRow = {
  customer_id: String
  email: String
  normalized_name: String
  status: String
  source_updated_at: String
}
```

### 5 ステージ

```
ExtractFromRds    → 読み取り: AWS RDS PostgreSQL（!Db）
TransformRows     → 変換: 純粋関数（エフェクトなし）
LoadToAzurePostgres → 書き込み: Azure DB for PostgreSQL（!AzureDb）
SaveProofToBlob   → 証跡: Azure Blob Storage（!AzureStorage）
VerifyRowCount    → 検証: source 件数 == target 件数（!AzureDb）
```

### エントリポイント

```fav
public fn main(ctx: AppCtx) -> Result<Unit, String> !Db !AzureDb !AzureStorage
```

実行方法:
```bash
fav run --legacy src/migrate.fav -- \
  "$RDS_CONN_STR" "$AZURE_CONN_STR" \
  "$AZURE_STORAGE_ACCOUNT" "$AZURE_STORAGE_KEY" "$AZURE_CONTAINER"
```

---

## インフラ設計

### AWS（Terraform: `terraform/aws/`）

| リソース | 用途 |
|---|---|
| `aws_db_instance` (PostgreSQL) | ソースDB（`customers` テーブル） |
| `aws_secretsmanager_secret` | RDS 接続文字列を保管 |
| `aws_s3_bucket` | proof/crosscloud/ 証跡保存 |
| `aws_iam_role` | ECS/EC2 実行ロール |
| `aws_security_group` | RDS アクセス制御 |
| `aws_db_subnet_group` | RDS サブネット設定 |

### Azure（Terraform: `terraform/azure/`）

| リソース | 用途 |
|---|---|
| `azurerm_resource_group` | `favnir-crosscloud-demo` |
| `azurerm_postgresql_flexible_server` | ターゲットDB（`customers_migrated` テーブル） |
| `azurerm_postgresql_flexible_server_database` | `appdb` |
| `azurerm_storage_account` | Blob 証跡用ストレージ |
| `azurerm_storage_container` | `proof` コンテナ |

---

## 完了条件（PASS=5）

| # | ステージ / チェック | 確認方法 |
|---|---|---|
| 1 | `ExtractFromRds` — AWS RDS から `customers` テーブル読み取り成功 | verify.sh で source 行数確認 |
| 2 | `TransformRows` — 全行の `normalized_name` が trim 済み | Azure 側クエリで確認 |
| 3 | `LoadToAzurePostgres` — Azure DB の `customers_migrated` に行が挿入されている | `SELECT COUNT(*)` で確認 |
| 4 | `SaveProofToBlob` — Azure Blob に証跡 JSON が存在する | `az storage blob list` で確認 |
| 5 | `VerifyRowCount` — source 行数 == target 行数 | パイプライン終了コード 0 |

---

## v15.1.0 以降の計画（Azure AD + AWS OIDC 直接信頼）

v15.0.0 完了後、認証フェーズとして以下を検討:

```
Azure AD アプリ登録
  → client_credentials フロー でトークン取得
  → AWS IAM: Entra ID を OIDC プロバイダーとして登録
  → sts:AssumeRoleWithWebIdentity
  → AWS IAM 一時認証情報
```

**crosscloud/plan.md の「Entra ID → Cognito 連携」よりシンプル**（Cognito User Pool 設定不要）。
v15.1.0 開始時に plan.md を「OIDC 直接信頼版」へ更新する。

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `infra/e2e-demo/crosscloud/plan.md` | 元の完全版設計（v15.1+ のための参照） |
| `versions/roadmap-v14.1-v15.0.md` | v15.0.0 の簡略版スコープ定義 |
| `infra/e2e-demo/fav2py/terraform/main.tf` | AWS RDS Terraform の参考実装 |
| `infra/e2e-demo/fav2py/src/pipeline.fav` | Favnir + Postgres/Azure パターンの参考 |
