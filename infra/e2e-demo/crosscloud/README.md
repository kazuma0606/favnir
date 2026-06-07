# Cross-Cloud Migration E2E Demo

Azure Functions -> Entra ID -> Cognito -> AWS API Gateway -> AWS Lambda verifier -> Favnir pipeline -> Azure Database for PostgreSQL

## Goal

Validate a realistic but still lightweight multi-cloud migration flow:

1. Azure Functions obtains an Entra ID bearer token.
2. AWS Cognito federates with Entra ID and becomes the AWS-side auth boundary.
3. Azure Functions signs the request body with HMAC using a shared secret.
4. AWS API Gateway validates the Cognito-backed token.
5. AWS Lambda verifier validates HMAC, timestamp, nonce, and claim/payload consistency.
6. Favnir reads from AWS RDS PostgreSQL, transforms the dataset, and writes to Azure Database for PostgreSQL.
7. Audit proof is stored on both clouds.

This demo is intentionally smaller than a full migration platform. The purpose is to validate trust boundaries, replay protection, and one concrete data path.

## Minimal Architecture

```text
Azure
  Resource Group: favnir-crosscloud-demo
    Azure Functions
      - gets Entra ID token
      - builds canonical request
      - adds HMAC headers
      - calls AWS API Gateway
    Key Vault
      - HMAC shared secret
      - AWS API audience / endpoint config
    Azure Database for PostgreSQL
      - target table: customers_migrated

AWS
  Cognito User Pool
    - federates with Entra ID
    - maps claims/groups for AWS-side authorization
  API Gateway HTTP API
    - Cognito/JWT authorizer
    - forwards to Lambda verifier
  Lambda verifier
    - checks HMAC/timestamp/nonce
    - rejects tampered or replayed requests
    - starts Favnir migration path
  RDS PostgreSQL
    - source table: customers
  S3
    - audit proofs
```

## Request Contract

The first version should keep the request schema small and explicit.

```json
{
  "job_type": "rds_to_azure_pg_customers_v1",
  "request_id": "01JY...ULID",
  "source": {
    "kind": "aws_rds_postgres",
    "database": "appdb",
    "table": "customers"
  },
  "target": {
    "kind": "azure_postgres",
    "database": "appdb",
    "table": "customers_migrated"
  },
  "options": {
    "mode": "upsert",
    "batch_size": 500
  }
}
```

### Required HTTP headers

- `Authorization: Bearer <cognito-access-or-id-token>`
- `X-Fav-Timestamp: 2026-06-07T20:00:00Z`
- `X-Fav-Nonce: <uuid-or-ulid>`
- `X-Fav-Content-Sha256: <hex>`
- `X-Fav-Signature: <base64url(hmac_sha256)>`
- `X-Fav-Key-Id: crosscloud-v1`

## Canonical String For HMAC

The HMAC must be computed over a stable string, not raw JSON formatting.

```text
METHOD
PATH
X-FAV-TIMESTAMP
X-FAV-NONCE
X-FAV-KEY-ID
JWT-SUBJECT
JWT-TENANT
X-FAV-CONTENT-SHA256
JOB-TYPE
```

Rules:

- `METHOD` is uppercase, e.g. `POST`
- `PATH` is exact API path, e.g. `/migrations/run`
- `JWT-SUBJECT` is taken from the validated Cognito token
- `JWT-TENANT` is the Entra tenant id claim propagated through Cognito
- `X-FAV-CONTENT-SHA256` is the SHA-256 of the exact request body bytes
- `JOB-TYPE` must also match the JSON payload field

This gives integrity for both the body and the execution intent.

## Lambda Verifier Responsibilities

The verifier should stay narrow. It is an authn/authz/integrity gate, not the data plane itself.

1. Trust API Gateway/Cognito to reject invalid tokens.
2. Re-read required claims from the request context:
   - `sub`
   - `tid`
   - `aud`
   - `azp` or `appid`
   - `cognito:groups` or mapped role claims
3. Recompute the body SHA-256 and compare with `X-Fav-Content-Sha256`.
4. Load the HMAC secret by `X-Fav-Key-Id`.
5. Recompute and constant-time compare `X-Fav-Signature`.
6. Reject if timestamp skew exceeds 5 minutes.
7. Reject if nonce has already been seen.
8. Verify that JWT claims are allowed to run the requested `job_type`.
9. Emit an audit record.
10. Start the migration execution.

For replay protection, the minimal store can be a DynamoDB table:

- partition key: `nonce`
- ttl attribute: 10 minutes

## Favnir Migration Scope

Keep the first migration path intentionally small:

- one source table
- one target table
- one deterministic transformation

### Source

AWS RDS PostgreSQL:

```sql
CREATE TABLE customers (
  customer_id UUID PRIMARY KEY,
  email TEXT NOT NULL,
  full_name TEXT NOT NULL,
  status TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
```

### Target

Azure Database for PostgreSQL:

```sql
CREATE TABLE customers_migrated (
  customer_id UUID PRIMARY KEY,
  email TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
  status TEXT NOT NULL,
  source_updated_at TIMESTAMPTZ NOT NULL,
  migrated_at TIMESTAMPTZ NOT NULL
);
```

### Minimal transformation

- `full_name` -> trimmed / normalized string
- `updated_at` -> `source_updated_at`
- append `migrated_at = now()`

### Execution mode

Use `upsert` from day one.

That keeps replays safe and makes idempotency testable.

## Proof And Audit

Store proof on both clouds.

### AWS S3

- `proof/crosscloud/requests/<request_id>.json`
- `proof/crosscloud/verifier/<request_id>.json`
- `proof/crosscloud/results/<request_id>.json`

### Azure Blob or Function logs

- `proof/crosscloud/functions/<request_id>.json`

Minimum proof fields:

- request id
- JWT subject
- tenant id
- job type
- source row count
- target row count
- started at / finished at
- verifier decision
- failure reason if any

## Terraform Requirements

### Azure

Terraform must create a dedicated resource group.

- resource group name: `favnir-crosscloud-demo`
- resources inside it:
  - Function App
  - Storage Account for Functions
  - Key Vault
  - Azure Database for PostgreSQL
  - optional Blob container for proof

### AWS

Terraform should create:

- Cognito User Pool
- Entra ID federation settings for Cognito
- API Gateway HTTP API
- Cognito or JWT authorizer configuration
- Lambda verifier
- DynamoDB nonce table
- S3 proof bucket or proof prefix in an existing bucket
- IAM role/policies
- source RDS connection config references

## First Test Matrix

### Success path

- valid Entra ID principal
- valid Cognito token
- valid HMAC
- fresh nonce
- 3 source rows
- 3 upserted target rows

### Failure path

- valid Entra ID principal, but no HMAC secret -> reject
- no Entra ID / no Cognito token, direct API call -> reject
- valid token, broken HMAC -> reject
- valid token, reused nonce -> reject
- valid token, expired timestamp -> reject
- valid token, unauthorized `job_type` -> reject
- target DB unavailable -> verifier passes, migration fails with proof

## Suggested Directory Layout

```text
infra/e2e-demo/crosscloud/
  README.md
  plan.md
  aws/
    terraform/
    lambda-verifier/
  azure/
    terraform/
    functions/
  fav/
    pipeline.fav
  scripts/
    run.sh
    verify.sh
```

## Out Of Scope For V1

- bidirectional sync
- CDC
- large-table chunking
- distributed job scheduler
- automatic secret rotation
- Azure -> AWS TiDB reverse path

The reverse path is a good follow-up, but should reuse the same verifier and proof model rather than be mixed into V1.
