# v15.2.0 Plan — GCP BigQuery Rune（`!Gcp` エフェクト）

Date: 2026-06-14

---

## Phase A: Cargo バージョン更新

### A-1: `fav/Cargo.toml`

```toml
version = "15.2.0"
```

---

## Phase B: テスト追加（v152000_tests）

### B-1: `fav/src/driver.rs` — `v152000_tests` モジュール追加

```rust
#[cfg(test)]
mod v152000_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn version_is_15_2_0() {
        let cargo = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo.contains("version = \"15.2.0\""), "Cargo.toml version should be 15.2.0");
    }

    #[test]
    fn bigquery_query_raw_primitive_exists() {
        let vm = fs::read_to_string("src/backend/vm.rs").unwrap();
        assert!(vm.contains("BigQuery.query_raw"), "vm.rs must contain BigQuery.query_raw primitive");
    }

    #[test]
    fn gcp_effect_in_ast() {
        let ast = fs::read_to_string("src/ast.rs").unwrap();
        assert!(ast.contains("Gcp"), "ast.rs must contain Effect::Gcp variant");
    }

    #[test]
    fn bigquery_rune_exists() {
        assert!(
            Path::new("../runes/bigquery/bigquery.fav").exists(),
            "runes/bigquery/bigquery.fav must exist"
        );
    }

    #[test]
    fn bigquery_e2e_demo_structure() {
        assert!(Path::new("../infra/e2e-demo/bigquery/src/demo.fav").exists());
        assert!(Path::new("../infra/e2e-demo/bigquery/terraform/gcp/main.tf").exists());
        assert!(Path::new("../infra/e2e-demo/bigquery/scripts/run.sh").exists());
        assert!(Path::new("../infra/e2e-demo/bigquery/README.md").exists());
    }
}
```

---

## Phase C: `Effect::Gcp` 型システム追加

### C-1: `fav/src/ast.rs`

`Effect` enum に `Gcp` を追加（`Snowflake` の直後）:

```rust
pub enum Effect {
    // ... 既存 ...
    Snowflake,
    Gcp,       // v15.2.0
    // ...
}
```

`Display` / `from_str` 実装にも `"Gcp"` を追加。

### C-2: `fav/src/middle/checker.rs`

#### `BUILTIN_EFFECTS` / `BUILTIN_NAMESPACES`

```rust
// BUILTIN_EFFECTS に追加
"Gcp",

// BUILTIN_NAMESPACES に追加（BigQuery は ns チェックの対象）
"BigQuery",
```

#### `builtin_ret_ty` — BigQuery ブランチ追加

Snowflake の実装を参考に、以下を追加:

```rust
("BigQuery", "query_raw") => {
    self.require_gcp_effect(span);
    Some(Type::Result(
        Box::new(Type::Str),
        Box::new(Type::Str),
    ))
}
("BigQuery", "execute_raw") => {
    self.require_gcp_effect(span);
    Some(Type::Result(
        Box::new(Type::Int),
        Box::new(Type::Str),
    ))
}
("BigQuery", "infer_table_raw") => {
    self.require_gcp_effect(span);
    Some(Type::Result(
        Box::new(Type::Str),
        Box::new(Type::Str),
    ))
}
```

#### `require_gcp_effect` ヘルパー追加

```rust
fn require_gcp_effect(&mut self, span: Span) {
    if !self.current_effects.contains(&Effect::Gcp) {
        self.errors.push(CheckError {
            code: "E0318",
            message: "BigQuery.* requires !Gcp effect declaration".to_string(),
            span,
        });
    }
}
```

### C-3: `fav/src/lineage.rs`

```rust
pub enum EffectKind {
    // ... 既存 ...
    SnowflakeRead,
    SnowflakeWrite,
    GcpRead,    // v15.2.0
    GcpWrite,   // v15.2.0
}

// collect_gcp_call_kinds 追加（collect_snowflake_call_kinds と同パターン）
fn collect_gcp_call_kinds(expr: &IRExpr, kinds: &mut HashSet<EffectKind>) {
    match expr {
        IRExpr::Call { ns: "BigQuery", method: "query_raw" | "infer_table_raw", .. } => {
            kinds.insert(EffectKind::GcpRead);
        }
        IRExpr::Call { ns: "BigQuery", method: "execute_raw", .. } => {
            kinds.insert(EffectKind::GcpWrite);
        }
        _ => {}
    }
}
```

---

## Phase D: `BigQuery.*` VM Primitive

### D-1: `fav/src/backend/vm.rs` — GCP 認証ヘルパー

BigQuery API 呼び出しの前に OAuth2 アクセストークンを取得する共通ヘルパーを追加:

```rust
/// GCP Service Account JSON から OAuth2 Bearer token を取得する。
/// GOOGLE_APPLICATION_CREDENTIALS 環境変数が指すファイルを読む。
fn gcp_get_access_token() -> Result<String, String> {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use std::time::{SystemTime, UNIX_EPOCH};

    let cred_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .map_err(|_| "GOOGLE_APPLICATION_CREDENTIALS not set".to_string())?;
    let cred_json = std::fs::read_to_string(&cred_path)
        .map_err(|e| format!("Failed to read credentials file: {e}"))?;
    let cred: serde_json::Value = serde_json::from_str(&cred_json)
        .map_err(|e| format!("Invalid credentials JSON: {e}"))?;

    let client_email = cred["client_email"].as_str()
        .ok_or("Missing client_email in credentials")?;
    let private_key = cred["private_key"].as_str()
        .ok_or("Missing private_key in credentials")?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let claims = serde_json::json!({
        "iss": client_email,
        "scope": "https://www.googleapis.com/auth/bigquery",
        "aud": "https://oauth2.googleapis.com/token",
        "iat": now,
        "exp": now + 3600,
    });

    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(|e| format!("Invalid private key: {e}"))?;
    let jwt = encode(&header, &claims, &key)
        .map_err(|e| format!("JWT encode failed: {e}"))?;

    // Google token endpoint に POST
    let resp = ureq::post("https://oauth2.googleapis.com/token")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&format!(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={}",
            jwt
        ))
        .map_err(|e| format!("Token request failed: {e}"))?;
    let body: serde_json::Value = resp.into_json()
        .map_err(|e| format!("Token response parse failed: {e}"))?;
    body["access_token"].as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No access_token in response: {}", body))
}
```

### D-2: `BigQuery.query_raw` primitive

```rust
// ── BigQuery.query_raw (v15.2.0) ─────────────────────────────────────────
// Args: (project_id: String, dataset: String, sql: String, params_json: String)
// Returns: Result<String, String>
// 同期クエリ → rows を JSON 文字列で返す
"BigQuery.query_raw" => {
    let mut it = args.into_iter();
    let project_id = vm_string(it.next().ok_or("BigQuery.query_raw: missing project_id")?,
        "BigQuery.query_raw")?;
    let _dataset = vm_string(it.next().ok_or("BigQuery.query_raw: missing dataset")?,
        "BigQuery.query_raw")?;
    let sql = vm_string(it.next().ok_or("BigQuery.query_raw: missing sql")?,
        "BigQuery.query_raw")?;
    let _params = vm_string(it.next().ok_or("BigQuery.query_raw: missing params_json")?,
        "BigQuery.query_raw")?;

    let token = match gcp_get_access_token() {
        Ok(t) => t,
        Err(e) => return Ok(VMValue::Variant("err".into(),
            Some(Box::new(VMValue::Str(e))))),
    };

    let url = format!(
        "https://bigquery.googleapis.com/bigquery/v2/projects/{}/queries",
        project_id
    );
    let body = serde_json::json!({
        "query": sql,
        "useLegacySql": false,
        "maxResults": 10000,
        "timeoutMs": 30000,
    });

    let resp = match ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
    {
        Ok(r) => r,
        Err(e) => return Ok(VMValue::Variant("err".into(),
            Some(Box::new(VMValue::Str(format!("BigQuery.query_raw: {e}")))))),
    };

    let json: serde_json::Value = match resp.into_json() {
        Ok(j) => j,
        Err(e) => return Ok(VMValue::Variant("err".into(),
            Some(Box::new(VMValue::Str(format!("BigQuery.query_raw: parse: {e}")))))),
    };

    // rows を JSON 文字列に変換
    let rows = json.get("rows").cloned().unwrap_or(serde_json::Value::Array(vec![]));
    let schema = json.get("schema").cloned().unwrap_or(serde_json::Value::Null);
    let result = serde_json::json!({"schema": schema, "rows": rows}).to_string();

    Ok(VMValue::Variant("ok".into(), Some(Box::new(VMValue::Str(result)))))
}
```

### D-3: `BigQuery.execute_raw` primitive

```rust
// ── BigQuery.execute_raw (v15.2.0) ───────────────────────────────────────
// Args: (project_id: String, dataset: String, sql: String, params_json: String)
// Returns: Result<Int, String>
// DML ジョブ（INSERT/UPDATE/DELETE）→ 影響行数を返す
"BigQuery.execute_raw" => {
    // Jobs API でクエリジョブを作成 → 完了まで polling → numDmlAffectedRows を返す
    // (実装は BigQuery.query_raw と同様の認証フロー)
    // POST /bigquery/v2/projects/{project_id}/jobs
    // Body: {"configuration": {"query": {"query": sql, "useLegacySql": false}}}
    // → jobId 取得 → GET /jobs/{jobId} で status.state == "DONE" を確認
    // → statistics.query.numDmlAffectedRows を返す
}
```

### D-4: `BigQuery.infer_table_raw` primitive

```rust
// ── BigQuery.infer_table_raw (v15.2.0) ───────────────────────────────────
// Args: (project_id: String, dataset: String, table: String)
// Returns: Result<String, String>
// INFORMATION_SCHEMA.COLUMNS クエリ → スキーマ JSON を返す
"BigQuery.infer_table_raw" => {
    // SELECT column_name, data_type, is_nullable
    // FROM `{project_id}.{dataset}.INFORMATION_SCHEMA.COLUMNS`
    // WHERE table_name = '{table}'
    // → BigQuery.query_raw と同じ認証・呼び出しフロー
}
```

---

## Phase E: checker.fav 更新

### E-1: `fav/runes/stdlib/compiler.fav`（またはインライン）

`bigquery_fn` 関数スキーム追加。Snowflake の `snowflake_fn` を参考:

```fav
fn bigquery_fn(method: String) -> Result<String, String> {
  // query_raw  → "Result<String, String>"
  // execute_raw → "Result<Int, String>"
  // infer_table_raw → "Result<String, String>"
}
```

`ns_to_effect` に追加:
```fav
// "BigQuery" → "Gcp"
```

---

## Phase F: BigQuery Rune

### F-1: `runes/bigquery/bigquery.fav`

Snowflake Rune（`runes/snowflake/snowflake.fav`）を参考に作成:

```fav
// runes/bigquery/bigquery.fav — BigQuery Rune (v15.2.0)

import rune "env"

fn query<T>(project_id: String, dataset: String, sql: String) -> Result<List<T>, String> !Gcp {
  chain raw_json <- BigQuery.query_raw(project_id, dataset, sql, "[]")
  chain rows    <- Json.decode_list<T>(raw_json)
  Result.ok(rows)
}

fn execute(project_id: String, dataset: String, sql: String) -> Result<Int, String> !Gcp {
  BigQuery.execute_raw(project_id, dataset, sql, "[]")
}
```

---

## Phase G: `fav.toml [gcp]` セクション

### G-1: `fav/src/driver.rs` — `GcpConfig` 構造体 + `inject_gcp_config`

```rust
#[derive(Debug, serde::Deserialize)]
pub struct GcpConfig {
    pub project_id: Option<String>,
    pub credentials_file: Option<String>,
    pub dataset: Option<String>,
}

fn inject_gcp_config(cfg: &GcpConfig) {
    if let Some(ref p) = cfg.project_id {
        std::env::set_var("GCP_PROJECT_ID", p);
    }
    if let Some(ref c) = cfg.credentials_file {
        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", c);
    }
}
```

### G-2: `FavToml` に `gcp: Option<GcpConfig>` 追加

```rust
pub struct FavToml {
    // ... 既存 ...
    pub gcp: Option<GcpConfig>,
}
```

`cmd_run` の toml 読み込みブロックに `inject_gcp_config` 呼び出しを追加。

---

## Phase H: `fav infer --from bigquery`

### H-1: `fav/src/driver.rs` — `cmd_infer` の `--from bigquery` ブランチ

Snowflake の `Snowflake.infer_table_raw` パターンを参考に実装:

```rust
"bigquery" => {
    let project_id = std::env::var("GCP_PROJECT_ID")
        .unwrap_or_default();
    let dataset = std::env::var("BQ_DATASET").unwrap_or_default();
    let src = format!(
        r#"public fn main(ctx: AppCtx) -> Result<Unit, String> !Gcp {{
             chain schema_json <- BigQuery.infer_table_raw("{}", "{}", "{}")
             bind _ <- ctx.io.println(schema_json)
             Result.ok(())
        }}"#,
        project_id, dataset, table
    );
    // fav run --legacy で実行 → stdout を Favnir 型定義に変換
}
```

出力例:
```fav
type Row = {
  user_id: Int,
  name: String,
  email: String,
  created_at: String,
}
```

---

## Phase I: E2E デモ

### I-1: `infra/e2e-demo/bigquery/src/demo.fav`

4 ステージパイプライン（Snowflake E2E と同パターン）:

```fav
// demo.fav — BigQuery E2E Demo (v15.2.0)
// 4 ステージ: LoadCsv |> TransformRows |> BigQueryInsert |> QuerySummary
//
// 環境変数:
//   GCP_PROJECT_ID          — GCP プロジェクト ID
//   GOOGLE_APPLICATION_CREDENTIALS — サービスアカウント JSON パス
//   BQ_DATASET              — BigQuery データセット名

import rune "env"
import rune "csv"
import rune "json"

type UserRow = {
  user_id: Int,
  full_name: String,
  email: String,
}

fn load_csv(ctx: AppCtx) -> Result<List<UserRow>, String> !Io {
  chain raw <- Fs.read_file_raw("/tmp/seed.csv")
  chain rows <- Csv.decode_list<UserRow>(raw)
  Result.ok(rows)
}

fn transform_rows(rows: List<UserRow>) -> Result<List<UserRow>, String> {
  bind cleaned <- List.map(rows, fn(r: UserRow) -> UserRow {
    { user_id: r.user_id, full_name: String.trim(r.full_name), email: String.lowercase(r.email) }
  })
  Result.ok(cleaned)
}

fn insert_to_bigquery(ctx: AppCtx, rows: List<UserRow>) -> Result<Int, String> !Gcp {
  bind project <- Option.unwrap_or(ctx.io.getenv_raw("GCP_PROJECT_ID"), "")
  bind dataset <- Option.unwrap_or(ctx.io.getenv_raw("BQ_DATASET"), "")
  bind values_sql <- List.map(rows, fn(r: UserRow) -> String {
    String.concat("(", String.concat(Int.to_string(r.user_id),
      String.concat(", '", String.concat(r.full_name,
        String.concat("', '", String.concat(r.email, "')")))))
  })
  bind values_str <- String.join(values_sql, ", ")
  bind sql <- String.concat("INSERT INTO users (user_id, full_name, email) VALUES ", values_str)
  BigQuery.execute_raw(project, dataset, sql, "[]")
}

fn query_summary(ctx: AppCtx) -> Result<String, String> !Gcp {
  bind project <- Option.unwrap_or(ctx.io.getenv_raw("GCP_PROJECT_ID"), "")
  bind dataset <- Option.unwrap_or(ctx.io.getenv_raw("BQ_DATASET"), "")
  BigQuery.query_raw(project, dataset, "SELECT COUNT(*) as cnt FROM users", "[]")
}

public fn main(ctx: AppCtx) -> Result<Unit, String> {
  seq [
    fn LoadCsv()       -> Result<List<UserRow>, String> { load_csv(ctx) }
    fn TransformRows() -> Result<List<UserRow>, String> { transform_rows(LoadCsv()) }
    fn BigQueryInsert()-> Result<Int, String>            { insert_to_bigquery(ctx, TransformRows()) }
    fn QuerySummary()  -> Result<String, String>         { query_summary(ctx) }
  ]
  bind _log <- ctx.io.println(String.concat("Summary: ", QuerySummary()))
  Result.ok(())
}
```

### I-2: `infra/e2e-demo/bigquery/terraform/gcp/main.tf`

```hcl
terraform {
  required_providers {
    google = { source = "hashicorp/google", version = "~> 5.0" }
  }
}

provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

resource "google_bigquery_dataset" "demo" {
  dataset_id = "favnir_demo"
  location   = var.gcp_region
}

resource "google_bigquery_table" "users" {
  dataset_id = google_bigquery_dataset.demo.dataset_id
  table_id   = "users"
  deletion_protection = false

  schema = jsonencode([
    { name = "user_id",   type = "INT64",  mode = "REQUIRED" },
    { name = "full_name", type = "STRING", mode = "NULLABLE" },
    { name = "email",     type = "STRING", mode = "NULLABLE" },
  ])
}

variable "gcp_project_id" { type = string }
variable "gcp_region"     { type = string; default = "asia-northeast1" }

output "dataset_id" { value = google_bigquery_dataset.demo.dataset_id }
output "table_id"   { value = google_bigquery_table.users.table_id }
```

### I-3: `infra/e2e-demo/bigquery/scripts/seed.sh`

```bash
#!/bin/bash
# seed.sh — BigQuery テスト用 CSV を生成する
cat > /tmp/seed.csv <<'CSV'
user_id,full_name,email
1,  Alice Smith  ,alice@example.com
2,  Bob Jones  ,BOB@example.com
3,  Carol White  ,carol@example.com
CSV
echo "[seed] /tmp/seed.csv 生成完了（3 件）"
```

### I-4: `infra/e2e-demo/bigquery/scripts/run.sh`

```bash
#!/bin/bash
# run.sh — BigQuery E2E デモ実行
set -euo pipefail

GCP_PROJECT_ID="${1:-}"
BQ_DATASET="${BQ_DATASET:-favnir_demo}"

if [ -z "$GCP_PROJECT_ID" ]; then
  echo "Usage: $0 <gcp_project_id>"
  exit 1
fi

export GCP_PROJECT_ID
export BQ_DATASET
export GOOGLE_APPLICATION_CREDENTIALS="${GOOGLE_APPLICATION_CREDENTIALS:-}"

bash "$(dirname "$0")/seed.sh"
fav run --legacy "$(dirname "$0")/../src/demo.fav"
```

### I-5: `infra/e2e-demo/bigquery/scripts/verify.sh`

```bash
#!/bin/bash
# verify.sh — BigQuery に 3 件 INSERT されているか確認
set -euo pipefail
PASS=0; FAIL=0

check() {
  local LABEL="$1"; local EXPECTED="$2"; local ACTUAL="$3"
  if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "[PASS] ${LABEL}"; PASS=$((PASS+1))
  else
    echo "[FAIL] ${LABEL} — expected ${EXPECTED}, got ${ACTUAL}"; FAIL=$((FAIL+1))
  fi
}

COUNT=$(bq query --nouse_legacy_sql --format=csv \
  "SELECT COUNT(*) FROM \`${GCP_PROJECT_ID}.${BQ_DATASET}.users\`" \
  | tail -1 | tr -d ' \r\n')

check "INSERT 3件" "3" "$COUNT"

echo ""
echo "PASS=${PASS} FAIL=${FAIL}"
[ "$FAIL" -eq 0 ] || exit 1
```

---

## Phase J: コミット

コミットメッセージ例:
```
feat: v15.2.0 — GCP BigQuery Rune + !Gcp エフェクト

- vm.rs: BigQuery.query_raw / execute_raw / infer_table_raw primitive 追加
- ast.rs / checker.rs / lineage.rs: Effect::Gcp + E0318 + GcpRead/GcpWrite 追加
- checker.fav: bigquery_fn スキーム + ns_to_effect 更新
- runes/bigquery/bigquery.fav: query<T> / execute Rune
- driver.rs: GcpConfig + inject_gcp_config + fav infer --from bigquery
- infra/e2e-demo/bigquery/: 4 ステージ E2E デモ（Terraform GCP + scripts）
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version → 15.2.0 |
| `fav/Cargo.lock` | 自動更新 |
| `fav/src/ast.rs` | `Effect::Gcp` 追加 |
| `fav/src/backend/vm.rs` | `gcp_get_access_token` ヘルパー + BigQuery.* 3 primitive 追加 |
| `fav/src/middle/checker.rs` | `require_gcp_effect` + `builtin_ret_ty` BigQuery ブランチ + E0318 |
| `fav/src/lineage.rs` | `GcpRead` / `GcpWrite` + `collect_gcp_call_kinds` |
| `fav/src/driver.rs` | `GcpConfig` + `inject_gcp_config` + `v152000_tests` + `cmd_infer` 拡張 |
| `runes/bigquery/bigquery.fav`（新規）| query<T> / execute Rune |
| `infra/e2e-demo/bigquery/`（新規）| E2E デモ一式 |

---

## 実装順序

```
A（バージョン）→ B（テスト）
→ C（Effect::Gcp 型システム）
→ D（vm.rs BigQuery.* primitive）
→ E（checker.fav 更新）
→ F（BigQuery Rune）
→ G（fav.toml [gcp]）
→ H（fav infer --from bigquery）
→ I（E2E デモ）
→ J（コミット）
```

D の `gcp_get_access_token` がコア実装。
認証が通れば REST API 呼び出しは Snowflake 実装のほぼ同パターンで実装できる。
