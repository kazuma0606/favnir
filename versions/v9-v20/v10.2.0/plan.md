# Favnir v10.2.0 Plan

Date: 2026-06-04
Theme: VM Primitive 追加 — Snowflake SQL API v2

---

## Phase A: JWT ヘルパー実装

### A-1: Serde クレームの構造体を追加

`vm.rs` の native-only ブロック内（`// ── Snowflake helpers (v10.2.0)` セクションを新設）:

```rust
#[derive(serde::Serialize)]
struct SnowflakeClaims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
}
```

### A-2: `snowflake_generate_jwt` 関数を追加

```rust
#[cfg(not(target_arch = "wasm32"))]
fn snowflake_generate_jwt(
    account: &str,
    user: &str,
    private_key_pem: &str,
    public_key_fp: &str,
) -> Result<String, String> {
    let account_up = account.to_uppercase();
    let user_up    = user.to_uppercase();

    let now = chrono::Utc::now().timestamp();
    let claims = SnowflakeClaims {
        iss: format!("{}.{}.SHA256:{}", account_up, user_up, public_key_fp),
        sub: format!("{}.{}", account_up, user_up),
        iat: now,
        exp: now + 3600,
    };

    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(|e| format!("Snowflake JWT: invalid private key: {}", e))?;

    jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| format!("Snowflake JWT: encode failed: {}", e))
}
```

**依存**: `jsonwebtoken`（既存）、`chrono`（既存）。

### A-3: `snowflake_api_post` 関数を追加

```rust
#[cfg(not(target_arch = "wasm32"))]
fn snowflake_api_post(
    account: &str,
    jwt: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let url = format!(
        "https://{}.snowflakecomputing.com/api/v2/statements",
        account
    );
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", jwt))
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .set("X-Snowflake-Authorization-Token-Type", "KEYPAIR_JWT")
        .send_string(&body.to_string())
        .map_err(|e| match e {
            ureq::Error::Status(_, r) => r.into_string().unwrap_or_default(),
            ureq::Error::Transport(t) => t.to_string(),
        })?;

    let text = resp.into_string()
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text)
        .map_err(|e| format!("Snowflake API: invalid JSON response: {}", e))
}
```

### A-4: `snowflake_read_env` ヘルパーを追加

環境変数を読み取り、未設定時に `Err` を返す共通関数:

```rust
fn snowflake_read_env(key: &str) -> Result<String, String> {
    std::env::var(key)
        .map_err(|_| format!("{} is not set", key))
}
```

---

## Phase B: `call_builtin` に Snowflake primitive を追加

### B-1: `Snowflake.execute_raw` を追加

`call_builtin` の LLM セクションの下に追加:

```rust
// ── Snowflake.execute_raw / Snowflake.query_raw (v10.2.0) ───────────
"Snowflake.execute_raw" => {
    let sql = match args.into_iter().next() {
        Some(VMValue::Str(s)) => s,
        _ => return err_vm(VMValue::Str("Snowflake.execute_raw requires a sql argument".to_string())),
    };
    // 環境変数読み取り
    let account    = match snowflake_read_env("SNOWFLAKE_ACCOUNT")     { Ok(v) => v, Err(e) => return err_vm(VMValue::Str(e)) };
    let user       = match snowflake_read_env("SNOWFLAKE_USER")        { Ok(v) => v, Err(e) => return err_vm(VMValue::Str(e)) };
    let privkey    = match snowflake_read_env("SNOWFLAKE_PRIVATE_KEY") { Ok(v) => v, Err(e) => return err_vm(VMValue::Str(e)) };
    let pubkey_fp  = match snowflake_read_env("SNOWFLAKE_PUBLIC_KEY_FP") { Ok(v) => v, Err(e) => return err_vm(VMValue::Str(e)) };
    // JWT 生成
    let jwt = match snowflake_generate_jwt(&account, &user, &privkey, &pubkey_fp) {
        Ok(t) => t,
        Err(e) => return err_vm(VMValue::Str(e)),
    };
    // リクエストボディ構築
    let mut body = serde_json::json!({ "statement": sql, "timeout": 60 });
    if let Ok(wh) = std::env::var("SNOWFLAKE_WAREHOUSE") { body["warehouse"] = serde_json::Value::String(wh); }
    if let Ok(rl) = std::env::var("SNOWFLAKE_ROLE")      { body["role"]      = serde_json::Value::String(rl); }
    if let Ok(db) = std::env::var("SNOWFLAKE_DATABASE")  { body["database"]  = serde_json::Value::String(db); }
    if let Ok(sc) = std::env::var("SNOWFLAKE_SCHEMA")    { body["schema"]    = serde_json::Value::String(sc); }
    // API 呼び出し
    match snowflake_api_post(&account, &jwt, &body) {
        Ok(_) => ok_vm(VMValue::Str("ok".to_string())),
        Err(e) => err_vm(VMValue::Str(e)),
    }
}
```

### B-2: `Snowflake.query_raw` を追加

`Snowflake.execute_raw` の直下に追加。
API 呼び出しまでは同じ。レスポンス処理のみ異なる:

```
// レスポンス変換: rowType + data → JSON オブジェクト配列
let cols: Vec<String> = response["resultSetMetaData"]["rowType"]
    .as_array()
    .unwrap_or(&vec![])
    .iter()
    .map(|c| c["name"].as_str().unwrap_or("").to_string())
    .collect();

let rows: Vec<serde_json::Value> = response["data"]
    .as_array()
    .unwrap_or(&vec![])
    .iter()
    .map(|row| {
        let mut obj = serde_json::Map::new();
        for (i, col) in cols.iter().enumerate() {
            obj.insert(col.clone(), row[i].clone());
        }
        serde_json::Value::Object(obj)
    })
    .collect();

let json_str = serde_json::to_string(&rows).unwrap_or_default();
ok_vm(VMValue::Str(json_str))
```

### B-3: `cargo build` 通過確認

---

## Phase C: テスト追加

### C-1: `v10200_tests` モジュールを追加

`vm_stdlib_tests.rs`（または `driver.rs` のテストセクション）に追加:

**テスト 1: `snowflake_execute_raw_missing_env_returns_err`**
- 環境変数なしで `Snowflake.execute_raw("SELECT 1")` を実行
- `Err("SNOWFLAKE_ACCOUNT is not set")` が返ることを確認

**テスト 2: `snowflake_query_raw_missing_env_returns_err`**
- 環境変数なしで `Snowflake.query_raw("SELECT 1")` を実行
- `Err("SNOWFLAKE_ACCOUNT is not set")` が返ることを確認

**テスト 3: `snowflake_jwt_well_formed`**
- ダミーの RSA キーペアを生成し、`snowflake_generate_jwt` を呼ぶ
- 返却されたトークンが `xxx.yyy.zzz` 形式（3 パート）であることを確認
- `jsonwebtoken::decode` でペイロードを取り出し、`iss` / `sub` / `iat` / `exp` が存在することを確認
  （署名検証は `Algorithm::RS256` + `Validation::new(Algorithm::RS256)` で行う）

### C-2: `cargo test v10200` — 3 件通過

---

## Phase D: 完了処理

### D-1: `cargo test bootstrap` — 通過確認

### D-2: `cargo test` — 全件通過確認

### D-3: `fav/Cargo.toml` version → `"10.2.0"`

### D-4: `fav/self/cli.fav` の `run_version` → `"10.2.0"`

### D-5: `memory/MEMORY.md` に v10.2.0 完了を記録

### D-6: commit

---

## 実装順序

```
A-1 → A-2 → A-3 → A-4   (ヘルパー関数)
B-1 → B-2 → B-3          (call_builtin への追加)
C-1 → C-2                 (テスト)
D-1 → D-2 → D-3 → D-4 → D-5 → D-6
```

A と B は vm.rs の `// ── Snowflake helpers` セクションを新設して追加する。
既存の Llm セクション（9197 行付近）の下に配置する。

---

## 注意事項

- `EncodingKey::from_rsa_pem` は `BEGIN PRIVATE KEY`（PKCS#8 unencrypted）を受け付ける
  （`BEGIN RSA PRIVATE KEY`（PKCS#1）は `from_rsa_pem` ではなく `from_rsa_der` が必要）
- 環境変数 `SNOWFLAKE_PRIVATE_KEY` には `\n` を含む PEM 文字列をそのまま渡す
  （シェルから渡す場合は `export SNOWFLAKE_PRIVATE_KEY=$(cat snowflake_rsa_key.p8)` を使う）
- `ureq` の `Status` エラーは 4xx/5xx レスポンスのため、ボディをそのまま Err に渡す
