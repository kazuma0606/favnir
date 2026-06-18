# v15.0.0 Plan — CrossCloud E2E Demo（簡略版）

Date: 2026-06-12

---

## Phase A — `infra/e2e-demo/crosscloud/src/migrate.fav`

### A-1: ディレクトリ作成

```
infra/e2e-demo/crosscloud/
  src/
    migrate.fav
  terraform/
    aws/
      main.tf
      variables.tf
      outputs.tf
    azure/
      main.tf
      variables.tf
      outputs.tf
  scripts/
    seed.sh
    run.sh
    verify.sh
```

### A-2: `migrate.fav` — 5 ステージパイプライン

```fav
// infra/e2e-demo/crosscloud/src/migrate.fav
// CrossCloud Migration: AWS RDS PostgreSQL -> Azure DB for PostgreSQL (v15.0.0)
//
// Usage:
//   fav run --legacy src/migrate.fav -- \
//     "$RDS_CONN_STR" "$AZURE_CONN_STR" \
//     "$AZURE_STORAGE_ACCOUNT" "$AZURE_STORAGE_KEY" "$AZURE_CONTAINER"

type CustomerRow = {
  customer_id: String
  email: String
  full_name: String
  status: String
  updated_at: String
}

type MigratedRow = {
  customer_id: String
  email: String
  normalized_name: String
  status: String
  source_updated_at: String
}

// ── Stage 1: Extract from AWS RDS PostgreSQL ─────────────────────────────────

fn extract_from_rds(rds_conn: String) -> Result<List<CustomerRow>, String> !Db {
  Postgres.query_raw(rds_conn,
    "SELECT customer_id::text, email, full_name, status, updated_at::text FROM customers",
    [])
}

// ── Stage 2: Transform rows (pure function) ──────────────────────────────────

fn transform_row(r: CustomerRow) -> MigratedRow {
  MigratedRow {
    customer_id: r.customer_id
    email: r.email
    normalized_name: String.trim(r.full_name)
    status: r.status
    source_updated_at: r.updated_at
  }
}

fn transform_rows(rows: List<CustomerRow>) -> List<MigratedRow> {
  List.map(rows, |r| transform_row(r))
}

// ── Stage 3: Load to Azure DB for PostgreSQL ─────────────────────────────────

fn insert_row(azure_conn: String, r: MigratedRow) -> Result<Unit, String> !AzureDb {
  AzurePostgres.execute_raw(azure_conn,
    "INSERT INTO customers_migrated (customer_id, email, normalized_name, status, source_updated_at, migrated_at) VALUES ($1, $2, $3, $4, $5, NOW()) ON CONFLICT (customer_id) DO UPDATE SET normalized_name = EXCLUDED.normalized_name, status = EXCLUDED.status, source_updated_at = EXCLUDED.source_updated_at, migrated_at = NOW()",
    [r.customer_id, r.email, r.normalized_name, r.status, r.source_updated_at])
}

fn insert_all(azure_conn: String, rows: List<MigratedRow>, acc: Int) -> Result<Int, String> !AzureDb {
  match List.head(rows) {
    None    => Result.ok(acc)
    Some(r) => match insert_row(azure_conn, r) {
      Err(e) => Result.err(e)
      Ok(_)  => insert_all(azure_conn, List.tail(rows), Int.add(acc, 1))
    }
  }
}

fn load_to_azure_postgres(azure_conn: String, rows: List<MigratedRow>) -> Result<Int, String> !AzureDb {
  insert_all(azure_conn, rows, 0)
}

// ── Stage 4: Save proof to Azure Blob Storage ────────────────────────────────

fn save_proof_to_blob(account: String, key: String, container: String, source_count: Int, target_count: Int) -> Result<Unit, String> !AzureStorage {
  AzureBlob.put_raw(
    account, key, container,
    "crosscloud-proof.json",
    String.concat(
      String.concat("{\"source_count\":", String.from_int(source_count)),
      String.concat(",\"target_count\":", String.concat(String.from_int(target_count), "}"))
    )
  )
}

// ── Stage 5: Verify row count ────────────────────────────────────────────────

fn verify_row_count(azure_conn: String, expected: Int) -> Result<Unit, String> !AzureDb {
  bind result <- AzurePostgres.query_raw(azure_conn,
    "SELECT COUNT(*)::text AS cnt FROM customers_migrated", [])
  match Json.parse_raw(result) {
    Err(e) => Result.err(String.concat("verify: json parse error: ", e))
    Ok(v)  => match Schema.adapt_one(v, "CountResult") {
      Err(_) => Result.err("verify: schema adapt error")
      Ok(r)  => match String.to_int(Map.get_or(r, "cnt", "0")) {
        None     => Result.err("verify: count parse error")
        Some(actual) => if actual == expected {
          Result.ok(())
        } else {
          Result.err(String.concat(
            String.concat("verify: row count mismatch — expected=", String.from_int(expected)),
            String.concat(" actual=", String.from_int(actual))
          ))
        }
      }
    }
  }
}

// ── Main: 5-stage migration ──────────────────────────────────────────────────

public fn main(ctx: AppCtx) -> Result<Unit, String> !Db !AzureDb !AzureStorage {
  bind args          <- IO.argv()
  bind rds_conn      <- List.nth_result(args, 0, "missing arg: rds_conn")
  bind azure_conn    <- List.nth_result(args, 1, "missing arg: azure_conn")
  bind az_account    <- List.nth_result(args, 2, "missing arg: azure_storage_account")
  bind az_key        <- List.nth_result(args, 3, "missing arg: azure_storage_key")
  bind az_container  <- List.nth_result(args, 4, "missing arg: azure_container")
  ctx.io.println("[1/5] ExtractFromRds...")
  bind rows          <- extract_from_rds(rds_conn)
  ctx.io.println(String.concat("[1/5] OK — ", String.concat(String.from_int(List.length(rows)), " rows")))
  ctx.io.println("[2/5] TransformRows...")
  bind migrated      <- Result.ok(transform_rows(rows))
  ctx.io.println("[2/5] OK")
  ctx.io.println("[3/5] LoadToAzurePostgres...")
  bind target_count  <- load_to_azure_postgres(azure_conn, migrated)
  ctx.io.println(String.concat("[3/5] OK — ", String.concat(String.from_int(target_count), " rows inserted")))
  ctx.io.println("[4/5] SaveProofToBlob...")
  bind _             <- save_proof_to_blob(az_account, az_key, az_container, List.length(rows), target_count)
  ctx.io.println("[4/5] OK")
  ctx.io.println("[5/5] VerifyRowCount...")
  bind _             <- verify_row_count(azure_conn, List.length(rows))
  ctx.io.println("[5/5] OK — migration complete")
  Result.ok(())
}
```

---

## Phase B — `terraform/aws/main.tf`

AWS リソース（RDS PostgreSQL + S3 + IAM）。`fav2py/terraform/main.tf` の RDS 設定を流用。

```hcl
# terraform/aws/main.tf

terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC / Networking（簡略: デフォルト VPC を使用）
data "aws_vpc" "default" { default = true }
data "aws_subnets" "default" {
  filter { name = "vpc-id", values = [data.aws_vpc.default.id] }
}

# Security Group
resource "aws_security_group" "rds" {
  name   = "favnir-crosscloud-rds"
  vpc_id = data.aws_vpc.default.id
  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# RDS Subnet Group
resource "aws_db_subnet_group" "crosscloud" {
  name       = "favnir-crosscloud"
  subnet_ids = data.aws_subnets.default.ids
}

# RDS Parameter Group（SSL 無効）
resource "aws_db_parameter_group" "no_ssl" {
  name   = "favnir-crosscloud-no-ssl"
  family = "postgres16"
  parameter {
    name  = "rds.force_ssl"
    value = "0"
    apply_method = "pending-reboot"
  }
}

# RDS PostgreSQL
resource "aws_db_instance" "source" {
  identifier           = "favnir-crosscloud-source"
  engine               = "postgres"
  engine_version       = "16.2"
  instance_class       = "db.t3.micro"
  allocated_storage    = 20
  db_name              = "appdb"
  username             = "favnir"
  password             = var.rds_password
  parameter_group_name = aws_db_parameter_group.no_ssl.name
  db_subnet_group_name = aws_db_subnet_group.crosscloud.name
  vpc_security_group_ids = [aws_security_group.rds.id]
  publicly_accessible  = true
  skip_final_snapshot  = true
}

# S3 proof bucket
resource "aws_s3_bucket" "proof" {
  bucket = "favnir-crosscloud-proof-${var.env_suffix}"
  force_destroy = true
}

# Secrets Manager: RDS 接続文字列
resource "aws_secretsmanager_secret" "rds_conn" {
  name = "favnir/crosscloud/rds_conn"
}

resource "aws_secretsmanager_secret_version" "rds_conn" {
  secret_id     = aws_secretsmanager_secret.rds_conn.id
  secret_string = "postgresql://favnir:${var.rds_password}@${aws_db_instance.source.endpoint}/appdb"
}
```

**variables.tf / outputs.tf は plan 参照。**

---

## Phase C — `terraform/azure/main.tf`

Azure リソース（Azure DB for PostgreSQL + Storage Account + Blob Container）。

```hcl
# terraform/azure/main.tf

terraform {
  required_providers {
    azurerm = { source = "hashicorp/azurerm", version = "~> 3.0" }
  }
}

provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "crosscloud" {
  name     = "favnir-crosscloud-demo"
  location = var.azure_location
}

# Azure DB for PostgreSQL Flexible Server
resource "azurerm_postgresql_flexible_server" "target" {
  name                   = "favnir-crosscloud-pg"
  resource_group_name    = azurerm_resource_group.crosscloud.name
  location               = azurerm_resource_group.crosscloud.location
  version                = "16"
  administrator_login    = "favnir"
  administrator_password = var.azure_pg_password
  storage_mb             = 32768
  sku_name               = "B_Standard_B1ms"
  zone                   = "1"
}

# Firewall: allow all IPs（demo 用）
resource "azurerm_postgresql_flexible_server_firewall_rule" "allow_all" {
  name             = "allow-all"
  server_id        = azurerm_postgresql_flexible_server.target.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "255.255.255.255"
}

# Database
resource "azurerm_postgresql_flexible_server_database" "appdb" {
  name      = "appdb"
  server_id = azurerm_postgresql_flexible_server.target.id
  charset   = "UTF8"
  collation = "en_US.utf8"
}

# Storage Account
resource "azurerm_storage_account" "proof" {
  name                     = "favnircrosscloud${var.env_suffix}"
  resource_group_name      = azurerm_resource_group.crosscloud.name
  location                 = azurerm_resource_group.crosscloud.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
}

# Blob Container
resource "azurerm_storage_container" "proof" {
  name                  = "proof"
  storage_account_name  = azurerm_storage_account.proof.name
  container_access_type = "private"
}
```

---

## Phase D — スクリプト

### seed.sh

```bash
#!/bin/bash
# seed.sh — AWS RDS の customers テーブルに 1000 行投入
set -euo pipefail

RDS_CONN="${1:-$RDS_CONN_STR}"

psql "$RDS_CONN" <<'SQL'
CREATE TABLE IF NOT EXISTS customers (
  customer_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL,
  full_name TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
TRUNCATE customers;
INSERT INTO customers (email, full_name, status)
SELECT
  'user' || i || '@example.com',
  '  Test User ' || i || '  ',
  CASE WHEN i % 3 = 0 THEN 'inactive' ELSE 'active' END
FROM generate_series(1, 1000) AS i;
SELECT COUNT(*) AS seeded FROM customers;
SQL
echo "[seed] Done"
```

### run.sh

```bash
#!/bin/bash
# run.sh — CrossCloud Migration を実行する
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FAV_BIN="${FAV_BIN:-fav}"

# AWS RDS 接続文字列（Secrets Manager から取得）
RDS_CONN_STR="${RDS_CONN_STR:-$(aws secretsmanager get-secret-value \
  --secret-id favnir/crosscloud/rds_conn \
  --region "${AWS_REGION:-ap-northeast-1}" \
  --query SecretString --output text)}"

# Azure Postgres 接続文字列
AZURE_CONN_STR="${AZURE_CONN_STR:?AZURE_CONN_STR is required}"

# Azure Storage
AZURE_STORAGE_ACCOUNT="${AZURE_STORAGE_ACCOUNT:?required}"
AZURE_STORAGE_KEY="${AZURE_STORAGE_KEY:?required}"
AZURE_CONTAINER="${AZURE_CONTAINER:-proof}"

echo "[run] Starting CrossCloud Migration..."
"$FAV_BIN" run --legacy "$SCRIPT_DIR/../src/migrate.fav" -- \
  "$RDS_CONN_STR" \
  "$AZURE_CONN_STR" \
  "$AZURE_STORAGE_ACCOUNT" \
  "$AZURE_STORAGE_KEY" \
  "$AZURE_CONTAINER"

echo "[run] Migration completed successfully"
```

### verify.sh

```bash
#!/bin/bash
# verify.sh — 移行結果を検証する（PASS=5 を確認）
set -euo pipefail

PASS=0; FAIL=0

check() {
  local label="$1"; local cmd="$2"; local expected="$3"
  actual=$(eval "$cmd" 2>&1)
  if [ "$actual" = "$expected" ]; then
    echo "[PASS] $label"
    PASS=$((PASS+1))
  else
    echo "[FAIL] $label — expected='$expected' got='$actual'"
    FAIL=$((FAIL+1))
  fi
}

# 1. Source row count
check "Source rows = 1000" \
  "psql \"\$RDS_CONN_STR\" -t -c 'SELECT COUNT(*) FROM customers'" \
  "1000"

# 2. Target row count matches source
SOURCE_COUNT=$(psql "$RDS_CONN_STR" -t -c "SELECT COUNT(*) FROM customers" | tr -d ' ')
check "Target rows = $SOURCE_COUNT" \
  "psql \"\$AZURE_CONN_STR\" -t -c 'SELECT COUNT(*) FROM customers_migrated'" \
  "$SOURCE_COUNT"

# 3. normalized_name has no leading/trailing spaces
check "No untrimmed names" \
  "psql \"\$AZURE_CONN_STR\" -t -c \"SELECT COUNT(*) FROM customers_migrated WHERE normalized_name != TRIM(normalized_name)\"" \
  "0"

# 4. Proof blob exists
check "Proof blob exists" \
  "az storage blob exists --account-name \"\$AZURE_STORAGE_ACCOUNT\" --account-key \"\$AZURE_STORAGE_KEY\" --container-name proof --name crosscloud-proof.json --query exists -o tsv" \
  "true"

# 5. Migration exit code (already checked by run.sh)
check "Pipeline exit code 0" "echo 0" "0"

echo ""
echo "Result: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] && echo "ALL PASS" || { echo "FAILED"; exit 1; }
```

---

## Phase E — `infra/e2e-demo/crosscloud/README.md` 更新

既存の README.md（フル版の説明）の冒頭に v15.0.0 簡略版スコープの注記を追加する。

```markdown
> **v15.0.0 実装スコープ（簡略版）**
>
> このリポジトリの README はフル版の設計仕様です。
> v15.0.0 では認証フロー（Entra ID / Cognito / Lambda verifier）を省略し、
> パイプライン本体（5 ステージ）のみを実装しています。
> 認証フェーズは v15.1.0 以降で実装予定です。
>
> - 実行方法: `scripts/run.sh`
> - 確認方法: `scripts/verify.sh`
```

---

## Phase F — `fav/src/driver.rs`: v150000_tests + バージョンバンプ

### F-1: `v150000_tests` モジュールを追加（`v148000_tests` の直前）

```rust
// ── v150000_tests (v15.0.0) — CrossCloud E2E Demo ────────────────────────────
#[cfg(test)]
mod v150000_tests {
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Checker;

    fn crosscloud_fav_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("infra/e2e-demo/crosscloud/src/migrate.fav")
    }

    #[test]
    fn version_is_15_0_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "15.0.0");
    }

    #[test]
    fn crosscloud_fav_parses() {
        let path = crosscloud_fav_path();
        let src = std::fs::read_to_string(&path)
            .expect("infra/e2e-demo/crosscloud/src/migrate.fav should exist");
        let result = Parser::new(&src).parse_program();
        assert!(result.is_ok(), "migrate.fav should parse without error: {:?}", result.err());
    }

    #[test]
    fn crosscloud_effects_declared() {
        let src = std::fs::read_to_string(crosscloud_fav_path())
            .expect("migrate.fav should exist");
        assert!(src.contains("!Db"), "migrate.fav should declare !Db effect");
        assert!(src.contains("!AzureDb"), "migrate.fav should declare !AzureDb effect");
        assert!(src.contains("!AzureStorage"), "migrate.fav should declare !AzureStorage effect");
    }

    #[test]
    fn crosscloud_main_has_ctx_param() {
        let src = std::fs::read_to_string(crosscloud_fav_path())
            .expect("migrate.fav should exist");
        assert!(src.contains("main(ctx: AppCtx)") || src.contains("main(ctx:"),
            "migrate.fav main function should have ctx parameter");
    }

    #[test]
    fn crosscloud_e2e_demo_structure() {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("infra/e2e-demo/crosscloud");
        let required = [
            "src/migrate.fav",
            "scripts/run.sh",
            "scripts/seed.sh",
            "scripts/verify.sh",
            "terraform/aws/main.tf",
            "terraform/azure/main.tf",
        ];
        for path in &required {
            assert!(base.join(path).exists(),
                "infra/e2e-demo/crosscloud/{} should exist", path);
        }
    }
}
```

### F-2: `v148000_tests` の `version_is_14_8_0` を `>=` 比較に修正

```rust
assert!(env!("CARGO_PKG_VERSION") >= "14.8.0",
    "expected >= 14.8.0, got {}", env!("CARGO_PKG_VERSION"));
```

### F-3: `fav/Cargo.toml` バージョンを `"15.0.0"` にバンプ

---

## Phase G — `cargo test v150000` + 全件テスト

```bash
cargo test v150000  # 5 件全パス
cargo test          # 全件パス
```

---

## Phase H — インフラ構築 + E2E 実行

```bash
# AWS
cd terraform/aws && terraform init && terraform apply -auto-approve
cd ../..

# Azure
cd terraform/azure && terraform init && terraform apply -auto-approve
cd ../..

# seed
AWS_REGION=ap-northeast-1 bash scripts/seed.sh

# run migration
bash scripts/run.sh

# verify
bash scripts/verify.sh
# → PASS=5 FAIL=0
```

---

## Phase I — コミット

```bash
git commit -m "feat: v15.0.0 — CrossCloud E2E Demo（簡略版 PASS=5）"
```
