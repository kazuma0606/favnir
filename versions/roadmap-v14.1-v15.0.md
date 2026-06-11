# Roadmap v14.1.0 〜 v15.0.0 — CrossCloud E2E Demo

Date: 2026-06-11

## 目標

`infra/e2e-demo/crosscloud/plan.md` の企画を実現する。
AWS RDS Postgres（ソース）→ Favnir パイプライン → Azure DB for PostgreSQL（ターゲット）のクロスクラウドマイグレーションを E2E デモとして動作させる。

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| Azure Postgres 認証 | パスワードベース接続文字列（`postgresql://user:password@host/db`） |
| 証跡ストレージ | Azure Blob Storage（モダンアプローチ。S3 ではなく Blob に保存） |
| CrossCloud 関数シグネチャ | `fn migrate(aws_ctx: AwsCtx, azure_ctx: AzureCtx)` を基本形。`CrossCloudCtx` 統合型もコメントで示す。 |
| AWS Rune | `runes/aws/` に型付きラッパーとして正式実装（VM ビルトイン直呼びより rich） |

---

## バージョン計画

### v14.1.0 — Azure PostgreSQL Rune

**テーマ**: Azure DB for PostgreSQL への接続・操作をサポート

**実装内容:**

- `fav/src/vm/builtins.rs`: Azure Postgres VM プリミティブ追加
  - `AzurePostgres.connect_raw(conn_str: String) -> Result<Unit, String>`
  - `AzurePostgres.execute_raw(conn_str, sql, params) -> Result<Int, String>`
  - `AzurePostgres.query_raw(conn_str, sql, params) -> Result<String, String>`
  - `AzurePostgres.close_raw(conn_str) -> Result<Unit, String>`
  - Crate: `tokio-postgres`（既存依存）+ `tokio-postgres-openssl` for TLS
  - 接続文字列形式: `postgresql://user:password@host:5432/db?sslmode=require`

- `fav/src/middle/checker.rs`: `AzurePostgres` namespace 追加（builtin_ret_ty / BUILTIN_EFFECTS）
  - `!AzureDb` エフェクト追加

- `fav/src/lineage.rs`: `!AzureDb(read/write)` 区別追加

- `runes/azure-postgres/rune.fav`: 型付きラッパー
  ```
  fn execute(ctx: AzureDbCtx, sql: String) -> Result<Int, String> !AzureDb
  fn query<T>(ctx: AzureDbCtx, sql: String) -> Result<List<T>, String> !AzureDb
  fn with_transaction(ctx: AzureDbCtx, f: fn(AzureDbCtx) -> Result<T, String>) -> Result<T, String> !AzureDb
  ```

- `fav/src/middle/checker.rs`: `AzureDbCtx` type alias 追加

- テスト: `v141000_tests` — `azure_postgres_primitives_registered`, `azure_db_effect_in_checker`, `azure_db_lineage_tracked`

---

### v14.2.0 — AzureCtx / AwsCtx + fav.toml [azure]

**テーマ**: クロスクラウド用 Context 型と設定ファイル拡張

**実装内容:**

- `fav/src/config.rs`: `fav.toml` に `[azure]` セクション追加
  ```toml
  [azure]
  postgres_url = "${AZURE_POSTGRES_URL}"
  storage_account = "${AZURE_STORAGE_ACCOUNT}"
  storage_key = "${AZURE_STORAGE_KEY}"
  container = "favnir-proof"
  ```

- `fav/src/vm/builtins.rs`: `Ctx.build_aws_raw` / `Ctx.build_azure_raw` プリミティブ追加

- `runes/ctx/rune.fav`: `AwsCtx` / `AzureCtx` / `CrossCloudCtx` 型追加
  ```
  type AwsCtx(Record)    // aws_region, s3_bucket, db_host, ...
  type AzureCtx(Record)  // postgres_url, storage_account, container, ...

  fn build_aws(region: String, s3_bucket: String, db_url: String) -> Result<AwsCtx, String>
  fn build_azure(postgres_url: String, storage_account: String, storage_key: String, container: String) -> Result<AzureCtx, String>

  // CrossCloudCtx を使う場合（コメントで代替案として提示）:
  // type CrossCloudCtx(Record)  // aws: AwsCtx, azure: AzureCtx
  // fn build_crosscloud(aws: AwsCtx, azure: AzureCtx) -> CrossCloudCtx
  ```

- `fav/src/driver.rs`: `inject_azure_config` / `inject_aws_crosscloud_config` 追加（`fav.toml` の [azure] セクションを env var 展開して Ctx に注入）

- テスト: `v142000_tests` — `fav_toml_azure_section_parsed`, `aws_ctx_build_raw_registered`, `azure_ctx_build_raw_registered`

---

### v14.3.0 — Azure lineage + fav explain 出力改善

**テーマ**: CrossCloud パイプラインのリネージ可視化

**実装内容:**

- `fav/src/lineage.rs`:
  - `EffectKind::AzureDbRead` / `EffectKind::AzureDbWrite` 追加
  - `EffectKind::AzureBlobRead` / `EffectKind::AzureBlobWrite` 追加
  - `collect_azure_call_kinds` 関数追加

- `fav/src/cli.rs` (`cmd_explain`):
  - `--lineage` 出力に Azure エフェクトを表示
  - CrossCloud パイプライン向けフォーマット: `[AWS RDS] → stage → [Azure Postgres]` 形式

- `self/cli.fav`: `run_explain` の lineage 表示に azure effect 追加

- テスト: `v143000_tests` — `azure_db_lineage_collected`, `crosscloud_lineage_format`

---

### v14.4.0 — AWS Rune 正式パッケージング (runes/aws/)

**テーマ**: AWS VM ビルトインを型付き Rune ラッパーとして正式公開

**実装内容:**

- `runes/aws/rune.fav`:
  ```
  // S3
  fn s3_put(ctx: AwsStorageCtx, key: String, body: String) -> Result<Unit, String> !Storage
  fn s3_get(ctx: AwsStorageCtx, key: String) -> Result<String, String> !Storage
  fn s3_delete(ctx: AwsStorageCtx, key: String) -> Result<Unit, String> !Storage
  fn s3_list(ctx: AwsStorageCtx, prefix: String) -> Result<List<String>, String> !Storage
  fn s3_exists(ctx: AwsStorageCtx, key: String) -> Result<Bool, String> !Storage

  // SQS
  fn sqs_send(ctx: AwsQueueCtx, message: String) -> Result<Unit, String> !Queue
  fn sqs_receive(ctx: AwsQueueCtx, max: Int) -> Result<List<String>, String> !Queue
  fn sqs_delete(ctx: AwsQueueCtx, receipt: String) -> Result<Unit, String> !Queue

  // Secrets Manager（新規追加）
  fn secrets_get(ctx: AwsCtx, secret_name: String) -> Result<String, String> !AWS
  ```

- `fav/src/vm/builtins.rs`: `AWS.secrets_get_raw` プリミティブ追加
  - Crate: `aws-sdk-secretsmanager`

- `fav/src/middle/checker.rs`: `AWS.secrets_get_raw` の builtin_ret_ty 追加

- `rune.toml` in `runes/aws/`: メタデータ追加

- テスト: `v144000_tests` — `aws_rune_file_parses`, `secrets_get_raw_registered`, `aws_rune_s3_functions_present`

---

### v14.5.0 — Azure Blob Storage Rune

**テーマ**: Azure Blob Storage への証跡保存をサポート

**実装内容:**

- `fav/src/vm/builtins.rs`: Azure Blob VM プリミティブ追加
  - `AzureBlob.put_raw(account, key, container, blob_name, body) -> Result<Unit, String>`
  - `AzureBlob.get_raw(account, key, container, blob_name) -> Result<String, String>`
  - `AzureBlob.list_raw(account, key, container, prefix) -> Result<String, String>`（JSON 配列）
  - `AzureBlob.delete_raw(account, key, container, blob_name) -> Result<Unit, String>`
  - Crate: `azure_storage` + `azure_storage_blobs`（`azure-sdk-for-rust`）

- `fav/src/middle/checker.rs`: `AzureBlob` namespace + `!AzureStorage` エフェクト追加

- `fav/src/lineage.rs`: `EffectKind::AzureBlobRead` / `EffectKind::AzureBlobWrite` 実装（v14.3.0 で定義済み）

- `runes/azure-blob/rune.fav`:
  ```
  fn put(ctx: AzureStorageCtx, blob_name: String, body: String) -> Result<Unit, String> !AzureStorage
  fn get(ctx: AzureStorageCtx, blob_name: String) -> Result<String, String> !AzureStorage
  fn list(ctx: AzureStorageCtx, prefix: String) -> Result<List<String>, String> !AzureStorage
  fn delete(ctx: AzureStorageCtx, blob_name: String) -> Result<Unit, String> !AzureStorage
  ```

- テスト: `v145000_tests` — `azure_blob_primitives_registered`, `azure_storage_effect_in_checker`, `azure_blob_rune_parses`

---

### v15.0.0 — CrossCloud E2E Demo

**テーマ**: AWS → Azure クロスクラウドマイグレーションの E2E デモ完成

**実装内容:**

- `infra/e2e-demo/crosscloud/src/migrate.fav`:
  ```
  // 基本シグネチャ: 引数を分離（CrossCloudCtx に統合する方法もある）
  fn migrate(aws_ctx: AwsCtx, azure_ctx: AzureCtx) -> Result<Unit, String>
  // 代替: fn migrate(ctx: CrossCloudCtx) -> Result<Unit, String>

  seq MigrationPipeline [
    ExtractFromRds    |> aws_ctx
    TransformRows
    LoadToAzurePostgres |> azure_ctx
    SaveProofToBlob   |> azure_ctx
    VerifyRowCount
  ]

  public fn main(ctx: AppCtx) -> Result<Unit, String>
  ```

- `infra/e2e-demo/crosscloud/src/validate.fav`:
  - HMAC 整合性検証（AWS Lambda verifier 相当）
  - 行数チェック: `source_count == target_count`
  - 証跡 JSON を Azure Blob に保存

- `infra/e2e-demo/crosscloud/terraform/`:
  - `aws/main.tf`: RDS Postgres, Secrets Manager, IAM, S3（ログ用）
  - `azure/main.tf`: Azure DB for PostgreSQL, Storage Account, Blob Container

- `infra/e2e-demo/crosscloud/scripts/`:
  - `run.sh`: AWS credentials + Azure credentials を取得し `fav run` を実行
  - `seed.sh`: AWS RDS にサンプルデータ投入（1000行の txn テーブル）
  - `verify.sh`: Azure Postgres の行数 + Blob の証跡 JSON を確認

- `infra/e2e-demo/crosscloud/README.md`: セットアップ手順・前提条件

- テスト: `v150000_tests`:
  - `crosscloud_fav_parses`
  - `crosscloud_lineage_aws_and_azure`
  - `crosscloud_effects_declared`
  - `crosscloud_main_has_ctx_param`
  - `crosscloud_e2e_demo_structure`（ファイル存在確認）

- **完了条件 (PASS=5)**:
  1. `ExtractFromRds` — AWS RDS から 1000 行取得
  2. `TransformRows` — スキーマ変換（id/amount/ts → azure 形式）
  3. `LoadToAzurePostgres` — Azure DB for PostgreSQL に INSERT 完了
  4. `SaveProofToBlob` — Azure Blob に証跡 JSON 保存
  5. `VerifyRowCount` — source 行数 == target 行数 検証

---

## 依存関係

```
v14.1.0 (Azure Postgres VM + checker)
    ↓
v14.2.0 (AwsCtx / AzureCtx + fav.toml [azure])
    ↓
v14.3.0 (lineage)   ←→   v14.4.0 (runes/aws/)   ←→   v14.5.0 (Azure Blob Rune)
              ↘                   ↓                   ↙
                          v15.0.0 (E2E Demo)
```

v14.3.0・v14.4.0・v14.5.0 は並行開発可能。すべて v15.0.0 の前提。

---

## 新規 Cargo 依存予定

| Crate | 用途 | バージョン目標 |
|---|---|---|
| `tokio-postgres-openssl` | Azure Postgres TLS | v14.1.0 |
| `aws-sdk-secretsmanager` | Secrets Manager | v14.4.0 |
| `azure_storage` | Azure Blob | v14.5.0 |
| `azure_storage_blobs` | Azure Blob | v14.5.0 |

---

## 実装ノート

- **Azure Postgres の SSL**: Azure DB for PostgreSQL は SSL 必須。`sslmode=require` を接続文字列に含める。`tokio-postgres` + `tokio-postgres-openssl` + `openssl` crate を使用。
- **Azure Blob の認証**: Shared Key（account + key）を使う。SAS トークンは v15.x 以降で検討。
- **HMAC 整合性**: `plan.md` では Lambda verifier として設計されているが、v15.0.0 では Favnir 内でハッシュ計算し Blob に保存する形でシンプル化。Lambda verifier は v15.1.0 以降。
- **CrossCloudCtx の選択**: `fn migrate(aws_ctx, azure_ctx)` を基本形とする。呼び出し元での `let ctx = CrossCloudCtx { aws: aws_ctx, azure: azure_ctx }` 形式は `runes/ctx/rune.fav` にコメントで残す。
- **Windows dev 環境**: Azure SDK の TLS は OpenSSL が必要。Windows では `OPENSSL_DIR` 環境変数設定が必要になる可能性あり。
- **fav2py（Python トランスパイラ）**: CrossCloud デモは Favnir ネイティブのみ。Python トランスパイルは対象外。
