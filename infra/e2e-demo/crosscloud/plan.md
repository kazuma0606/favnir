# Cross-Cloud Migration E2E Plan

Date: 2026-06-07

## Objective

Build a minimal but realistic cross-cloud E2E demo:

- Azure Functions as caller
- Entra ID -> Cognito federation
- HMAC request integrity
- AWS API Gateway + Lambda verifier
- AWS RDS PostgreSQL -> Favnir -> Azure Database for PostgreSQL
- Terraform-managed Azure resource group

## What To Validate At The Same Time

This scenario is heavy enough that a simple "request succeeded" check is not sufficient.
The first version should validate five concerns together:

1. Authentication
   - Entra ID principal can federate through Cognito
   - API Gateway only accepts Cognito-backed tokens

2. Integrity / tamper detection
   - HMAC covers request intent and body hash
   - modified payload, stale timestamp, or broken signature is rejected

3. Authorization
   - Cognito claims/groups are mapped to allowed `job_type`
   - valid identity does not automatically imply permission to run any migration

4. Idempotency
   - repeated migration request does not create duplicate rows
   - target write path is safe for retries and replay-safe for accepted jobs

5. Auditability
   - success and reject cases both leave usable proof
   - operator can distinguish auth failure, signature failure, authorization failure, and target-db failure

## Concrete V1 Test Cases

### Success case

- Entra ID principal is valid
- Cognito token is valid
- HMAC is valid
- nonce is fresh
- source `customers` rows are migrated to Azure PostgreSQL

### Reject cases

- Entra ID principal exists but does not know the HMAC secret
- direct API call without Entra ID / Cognito token
- valid token but broken HMAC
- valid token but replayed nonce
- valid token but unauthorized `job_type`

### Failure-but-observable cases

- verifier passes, but Azure Database for PostgreSQL is unavailable
- verifier passes, migration starts, but write phase fails with proof preserved

## Favnir 固有の価値

このデモの当初の設計は「マルチクラウド認証基盤の検証」が主軸であり、Favnir は ETL の実行エンジンに留まっていた。しかし以下の観点を組み込むと、Favnir でなければ成立しない部分が生まれる。

### 1. エフェクト宣言 = データフロー契約

```
fn migrate() -> Result<MigrationResult, String> !Postgres !Azure
```

この型シグネチャは「AWS RDS からしか読まない、Azure PG にしか書かない」ことをコンパイル時に静的に保証する。Python や Go ではこの保証は実行時にしか確認できない。

Lambda verifier が HMAC でリクエストの完全性を検証し、Favnir がエフェクト宣言でデータフローを保証するという二層構造になる。**認証はゲート、Favnir のパイプラインが信頼境界そのもの**、という位置づけに変わる。

### 2. 型状態パターンによる移行フェーズの保証

マイグレーションの各フェーズを型として定義する：

```
type Loaded(List<CustomerRow>)
type Validated(List<CustomerRow>)
type Transformed(List<MigratedRow>)

fn load()                  -> Result<Loaded, String>      !Postgres
fn validate(d: Loaded)     -> Result<Validated, String>
fn transform(d: Validated) -> Result<Transformed, String>
fn write(d: Transformed)   -> Result<Unit, String>        !Azure
```

`validate` には `Loaded` しか渡せない。`Transformed` を `validate` に渡すコードはコンパイルエラーになる。フェーズを飛ばした実行や順序の入れ替えが型レベルで不可能になる。

### 3. `seq` fail-fast による partial write の防止

```
seq Load |> Validate |> Transform |> Write
```

Stage 2（Validate）で行のスキーマ違反が検出された場合、Write は実行されない。audit proof には以下が自動的に記録される：

```
pipeline stopped at stage 2/4 'Validate': row 247 schema violation
```

「書き込みが起きたかどうか」をアプリコードで管理する必要がなく、Favnir の `seq` opcode がこれを保証する。

### 4. テストコードの読みやすさ

型状態パターンにより、テストの前提条件が型として自己文書化される。

- `Loaded` が来る = load は通過済み
- `Validated` が来る = load と validate は通過済み

Python では「このフィールドは validate 前提」というコメントを人間が書く必要がある。Favnir では型がそのコメントを兼ねる。各ステージは独立してユニットテスト可能で、前のステージの型を手で作って渡すだけでよい。モックフレームワークは不要。

**型が仕様書であり、仕様書がそのままテスト設計になる。** マルチクラウドのような複雑なシナリオほどこの恩恵が大きい。

### 5. `fav explain --lineage` をコンプライアンス証拠として使う

```
fav explain --lineage migration.fav
```

生成される lineage レポート（Source: `!Postgres(read)` → AWS RDS / Sink: `!Azure(write)` → Azure PG）を audit proof の一部として S3 に保存することで、「このパイプラインは RDS 以外からデータを読んでいない」をソースコードから静的に証明できる。

### 6. `fav transpile` を監査ドキュメントとして使う

`fav transpile migration.fav` で生成した Python を Azure 側パートナーへの監査提出資料として使える。Favnir と Python の両方で同一ロジックが保証されるため、二重実装なしにコードの人間可読版を提供できる。

---

## Explicitly Out Of Scope For V1

To keep this demo lightweight, the following are intentionally excluded:

- CDC / continuous sync
- bidirectional Azure -> AWS replication
- large-table chunking or parallel shard migration
- automatic secret rotation implementation
- Azure PostgreSQL -> AWS TiDB reverse path
- M365 / SharePoint / SaaS integration in the same flow

Those may become follow-up labs, but they should not be mixed into the first cross-cloud demo.

## Phase 1: Trust Boundary Only

Deliverables:

- Azure resource group created by Terraform
- Azure Function can obtain Entra ID token
- Cognito accepts Entra ID federation
- AWS API Gateway validates Cognito-backed token
- Lambda verifier validates HMAC/timestamp/nonce
- Proof written for allow/deny

Exit criteria:

- valid signed request returns accepted status
- Entra ID principal without HMAC secret is rejected
- direct API call without Entra ID/Cognito token is rejected
- replayed nonce is rejected

## Phase 2: Minimal Data Migration

Deliverables:

- AWS source table `customers`
- Azure target table `customers_migrated`
- Favnir pipeline that reads source and upserts target
- proof with row counts

Exit criteria:

- source rows appear in Azure target
- duplicate request does not duplicate rows

## Phase 3: Operational Checks

Deliverables:

- failure proof when Azure DB is unavailable
- failure proof when job type is unauthorized
- one verification script for both clouds

Exit criteria:

- every failure mode is observable without manual log archaeology

## Open Decisions

- whether Lambda verifier starts the Favnir pipeline directly or via queue
- where the Azure-side proof should live: Blob vs Function logs only
- whether AWS proof should reuse `favnir-e2e-demo` or get a dedicated bucket/prefix
- whether Cognito should expose groups, custom claims, or both for `job_type` authorization

## Recommendation

For V1:

- direct verifier -> migration invocation is acceptable
- use existing S3 proof bucket with `proof/crosscloud/` prefix
- add Azure Blob only if Function logs are not enough
- use Cognito as the AWS-side auth boundary, HMAC as the request integrity boundary
