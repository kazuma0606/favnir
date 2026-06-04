# Favnir v10.2.0 Spec

Date: 2026-06-04
Theme: VM Primitive 追加 — Snowflake SQL API v2

---

## 概要

Rust VM（`vm.rs`）に Snowflake SQL API v2 用の primitive を追加する。
このフェーズでは Rune・エフェクト型は追加しない。
`vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` を実装し、
後続の v10.3.0（Effect::Snowflake）・v10.6.0（Rune）の土台を作る。

---

## 前提（v10.1.0 完了時点）

- Snowflake インフラ構築済み（`infra/snowflake/`）
  - FAVNIR_WH / FAVNIR DB / PUBLIC schema / FAVNIR_APP ロール作成済み
  - SSM に接続情報を格納済み
- RSA キーペア生成済み（`infra/snowflake/snowflake_rsa_key.p8`）
- Snowflake ユーザーに公開鍵登録済み（`ALTER USER ... SET RSA_PUBLIC_KEY=...`）
- `jsonwebtoken = "9"` / `sha2 = "0.10"` / `base64 = "0.22"` が `Cargo.toml` に存在

---

## Snowflake SQL API v2 の仕様

### 認証方式

JWT Bearer 認証（RSA 2048-bit キーペア）。

**JWT クレーム**:
```
{
  "iss": "{ACCOUNT_UPPER}.{USER_UPPER}.SHA256:{FINGERPRINT}",
  "sub": "{ACCOUNT_UPPER}.{USER_UPPER}",
  "iat": <unix timestamp>,
  "exp": <unix timestamp + 3600>
}
```

- `ACCOUNT_UPPER`: アカウント識別子の大文字化（例: `RTQJKBW-IX11747`）
- `USER_UPPER`: ユーザー名の大文字化（例: `YOSHIMURAHISANORI`）
- `FINGERPRINT`: RSA 公開鍵の SHA256 フィンガープリント（base64, 例: `esJnLXZIP/...`）
- アルゴリズム: RS256（`EncodingKey::from_rsa_pem` が既存の `jsonwebtoken` で動作）

**注**: フィンガープリントは `SNOWFLAKE_PUBLIC_KEY_FP` 環境変数で提供する。
事前計算値: `SHA256:esJnLXZIP/bOd4Bbbyqc8F274i5z25zpNfTCPI4yg+Y=`（SHA256: プレフィックスを除いた値）

### エンドポイント

```
POST https://{account}.snowflakecomputing.com/api/v2/statements
Authorization: Bearer {jwt}
Content-Type: application/json
Accept: application/json
X-Snowflake-Authorization-Token-Type: KEYPAIR_JWT

{
  "statement": "<sql>",
  "timeout":   60,
  "warehouse": "<warehouse>",
  "role":      "<role>",
  "database":  "<database>",
  "schema":    "<schema>"
}
```

---

## 環境変数

| 変数名 | 必須 | 例 | 説明 |
|---|---|---|---|
| `SNOWFLAKE_ACCOUNT` | ✓ | `rtqjkbw-ix11747` | アカウント識別子 |
| `SNOWFLAKE_USER` | ✓ | `YOSHIMURAHISANORI` | ユーザー名 |
| `SNOWFLAKE_PRIVATE_KEY` | ✓ | `-----BEGIN PRIVATE KEY-----\n...` | RSA 秘密鍵 PEM 文字列 |
| `SNOWFLAKE_PUBLIC_KEY_FP` | ✓ | `esJnLXZIP/...` | 公開鍵 SHA256 フィンガープリント（SHA256: なし） |
| `SNOWFLAKE_WAREHOUSE` | — | `FAVNIR_WH` | デフォルト warehouse（省略可） |
| `SNOWFLAKE_ROLE` | — | `FAVNIR_APP` | デフォルト role（省略可） |
| `SNOWFLAKE_DATABASE` | — | `FAVNIR` | デフォルト database（省略可） |
| `SNOWFLAKE_SCHEMA` | — | `PUBLIC` | デフォルト schema（省略可） |

---

## VM Primitive

### Snowflake.execute_raw

```
Snowflake.execute_raw(sql: String) -> Result<String, String>
```

DDL / DML（CREATE TABLE, INSERT, UPDATE, DELETE 等）を実行する。
成功時は `Ok("ok")` を返す。

### Snowflake.query_raw

```
Snowflake.query_raw(sql: String) -> Result<String, String>
```

SELECT 文を実行し、結果を JSON 文字列として返す。

**返却 JSON 形式**:
```json
[
  {"ORDER_ID": 1, "CUSTOMER": "Alice", "AMOUNT": 100.0},
  {"ORDER_ID": 2, "CUSTOMER": "Bob",   "AMOUNT": 200.0}
]
```

Snowflake API v2 レスポンスの `resultSetMetaData.rowType`（カラム名）と
`data`（行データ）をマージしてオブジェクト配列に変換する。

---

## 実装方針

### ヘルパー関数

```rust
// Snowflake JWT を生成する
fn snowflake_generate_jwt(
    account: &str,
    user: &str,
    private_key_pem: &str,
    public_key_fp: &str,
) -> Result<String, String>

// Snowflake SQL API v2 に POST リクエストを送信する
fn snowflake_api_post(
    account: &str,
    jwt: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String>
```

### call_builtin への追加位置

`!Llm` 追加時と同じパターンで、`call_builtin` の match アームに追加する:
```rust
// ── Snowflake.execute_raw / Snowflake.query_raw (v10.2.0) ───────────
"Snowflake.execute_raw" => { ... }
"Snowflake.query_raw"   => { ... }
```

---

## エラーハンドリング

環境変数未設定時は早期 `Err` を返す（`!Llm` と同じパターン）:

```rust
let account = match std::env::var("SNOWFLAKE_ACCOUNT") {
    Ok(v) => v,
    Err(_) => return err_vm(VMValue::Str("SNOWFLAKE_ACCOUNT is not set".to_string())),
};
```

---

## テスト

環境変数未設定時のエラーテスト（実接続不要）:
- `snowflake_execute_raw_missing_env_returns_err`
- `snowflake_query_raw_missing_env_returns_err`
- `snowflake_jwt_well_formed` — JWT 生成の形式確認（署名なし、ペイロード検証）

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` が追加されている | |
| 環境変数未設定時に `Err("SNOWFLAKE_ACCOUNT is not set")` が返る | |
| JWT クレームに `iss` / `sub` / `iat` / `exp` が正しく設定される | |
| `cargo test v10200` — 3 件通過 | |
| `cargo test bootstrap` 維持 | |
| `cargo test` 全件通過 | |

---

## スコープ外（後続バージョンへ）

- `Effect::Snowflake` 追加 → v10.3.0
- `checker.fav` 更新 → v10.4.0
- `compiler.fav` NS 登録 → v10.5.0
- Snowflake Rune → v10.6.0
- フィンガープリント自動計算（秘密鍵から動的導出） → 検討中
