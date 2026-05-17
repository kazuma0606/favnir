# Favnir v4.5.0 実装計画 — Auth Rune

作成日: 2026-05-17

---

## Phase 0: バージョン更新 + 依存クレート追加

- `fav/Cargo.toml` の version を `"4.5.0"` に変更
- 以下の依存を追加:
  ```toml
  jsonwebtoken = "9"
  hmac = "0.12"
  sha2 = "0.10"
  ```
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.5.0` に更新

`jsonwebtoken` は HS256 / RS256 の JWT encode/decode/verify を担当する。
`hmac` + `sha2` は API キー生成の HMAC-SHA256 計算に使用する。

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: `Crypto.jwt_verify_raw`

```rust
"Crypto.jwt_verify_raw" => {
    // args: (token: String, secret: String, alg: String)
    // alg: "HS256" | "RS256"
    // 成功: ok_vm(VMValue::Record(claims_map))
    // 失敗: Ok(err_vm(VMValue::Str(e.to_string())))
}
```

実装方針:
- `alg == "HS256"`: `DecodingKey::from_secret(secret.as_bytes())` + `Validation::new(Algorithm::HS256)`
- `alg == "RS256"`: `DecodingKey::from_rsa_pem(secret.as_bytes())` + `Validation::new(Algorithm::RS256)`
- `decode::<serde_json::Value>(token, &key, &validation)` で検証
- `TokenData.claims` の各フィールドを `VMValue::Str` に変換して `HashMap` を構築
  - `serde_json::Value::Number(n)` → `n.to_string()`
  - `serde_json::Value::Bool(b)` → `b.to_string()`
  - `serde_json::Value::String(s)` → `s`
  - その他 → `serde_json::to_string(&v)` でフォールバック

### 1-B: `Crypto.jwt_decode_raw`

```rust
"Crypto.jwt_decode_raw" => {
    // args: (token: String)
    // 署名検証なし・有効期限チェックなし
    // ペイロード部分（base64url デコード）のみ返す
}
```

実装方針:
- `Validation::new(Algorithm::HS256)` に `.insecure_disable_signature_validation()` を設定
- または手動で `.` で分割して中央部分を base64url デコード
- `jsonwebtoken` の `dangerous_insecure_decode` を使用するのが最も簡単

### 1-C: `Crypto.jwt_sign_raw`

```rust
"Crypto.jwt_sign_raw" => {
    // args: (claims_json: String, secret: String, alg: String)
    // claims_json: '{"sub":"user1","exp":9999999999}' 形式
    // 成功: Ok(ok_vm(VMValue::Str(token)))
    // 失敗: Ok(err_vm(VMValue::Str(e)))
}
```

実装方針:
- `serde_json::from_str::<serde_json::Value>(&claims_json)` でパース
- `EncodingKey::from_secret(secret.as_bytes())` + `Header::new(Algorithm::HS256)`
- `encode(&header, &claims, &key)` でトークン生成

### 1-D: `Crypto.hmac_sha256_raw`

```rust
"Crypto.hmac_sha256_raw" => {
    // args: (key: String, data: String)
    // 返値: Ok(VMValue::Str(hex_string)) — 失敗しない
}
```

実装方針:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

let mut mac = HmacSha256::new_from_slice(key.as_bytes())
    .map_err(|e| format!("hmac_sha256: invalid key: {}", e))?;
mac.update(data.as_bytes());
let result = mac.finalize();
let hex = hex_encode(result.into_bytes());
Ok(VMValue::Str(hex))
```

hex エンコードは `format!("{:02x}", b)` でバイト列を変換。既存の hex ユーティリティを流用。

### 1-E: `Crypto.sha256_raw`

```rust
"Crypto.sha256_raw" => {
    // args: (data: String)
    // 返値: Ok(VMValue::Str(hex_string)) — 失敗しない
}
```

実装方針:
```rust
use sha2::{Sha256, Digest};
let mut hasher = Sha256::new();
hasher.update(data.as_bytes());
let result = hasher.finalize();
let hex = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
Ok(VMValue::Str(hex))
```

### 1-F: `Crypto.random_hex_raw`

```rust
"Crypto.random_hex_raw" => {
    // args: (n: Int) — 生成バイト数
    // 返値: Ok(VMValue::Str(hex_string)) — 2n 文字
    // rand::rngs::OsRng を使用（暗号学的安全）
}
```

実装方針:
```rust
use rand::RngCore;
let n = /* vm_int(args[0]) */;
let mut bytes = vec![0u8; n as usize];
rand::rngs::OsRng.fill_bytes(&mut bytes);
let hex = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
Ok(VMValue::Str(hex))
```

`rand` クレートは既存依存なので追加不要。`OsRng` は `rand::rngs::OsRng` として利用可能。

---

## Phase 2: checker.rs への変更（`fav/src/middle/checker.rs`）

### 2-A: `Effect::Auth` の追加

`Effect` enum に `Auth` バリアントを追加する:
```rust
pub enum Effect { Io, Db, Network, Random, Checkpoint, Rpc, Aws, Auth }
```

`Effect` の文字列変換（`effect_name` / `from_str` 等）に `Auth` を追加する。
`"!Auth"` パース対応も行う（既存の `match` に `"Auth"` を追加）。

### 2-B: `require_auth_effect` の追加

```rust
fn require_auth_effect(&mut self, span: &Span) {
    if !self.current_effects.contains(&Effect::Auth) {
        self.type_error("E0108", "Auth.* call requires `!Auth` effect on enclosing fn/trf", span);
    }
}
```

エラーコードは `E0108`（`E0107` が `!Db` 未宣言エラーなので連番）。

### 2-C: `check_builtin_apply` への `Crypto.*` アーム追加

既存 `("Gen", _)` フォールバックの**前**に配置:

```rust
// Crypto.* (v4.5.0)
("Crypto", "jwt_verify_raw") => {
    self.require_auth_effect(span);
    Some(Type::Result(
        Box::new(Type::Map(Box::new(Type::String), Box::new(Type::String))),
        Box::new(Type::String),
    ))
}
("Crypto", "jwt_decode_raw") => {
    self.require_auth_effect(span);
    Some(Type::Result(
        Box::new(Type::Map(Box::new(Type::String), Box::new(Type::String))),
        Box::new(Type::String),
    ))
}
("Crypto", "jwt_sign_raw") => {
    self.require_auth_effect(span);
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("Crypto", "hmac_sha256_raw") => {
    self.require_auth_effect(span);
    Some(Type::String)
}
("Crypto", "sha256_raw") => {
    self.require_auth_effect(span);
    Some(Type::String)
}
("Crypto", "random_hex_raw") => {
    self.require_auth_effect(span);
    Some(Type::String)
}
("Crypto", _) => {
    self.require_auth_effect(span);
    Some(Type::Unknown)
}
```

### 2-D: `compiler.rs` への `"Crypto"` 登録

`compiler.rs` の global loop で builtin namespace を列挙している箇所（`"Gen"`, `"Validate"` 等と同列）に `"Crypto"` を追加する。

---

## Phase 3: `fav.toml` 拡張（`fav/src/toml.rs`）

### 3-A: `AuthConfig` 構造体の追加

```rust
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub mode: String,  // "jwt" | "cognito" | "none"
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { mode: "jwt".into() }
    }
}
```

`FavToml` に `pub auth: Option<AuthConfig>` を追加。

### 3-B: TOML パース処理の追加

`parse_toml` 関数内の `[auth]` セクションをパースする:

```rust
"[auth]" => current_section = "auth",
// ...
"auth" if key == "mode" => {
    auth_config.mode = value.to_string();
}
```

### 3-C: `Auth.get_mode_raw` VM primitive の追加

```rust
"Auth.get_mode_raw" => {
    // fav.toml の [auth] mode 値を返す
    // FavToml がグローバルに参照できない場合は thread_local を使う
    let mode = AUTH_MODE.with(|m| m.borrow().clone());
    Ok(VMValue::Str(mode))
}
```

`AUTH_MODE` thread_local は `set_auth_mode(mode: &str)` で `cmd_run` 時に設定する。
checker.rs は `Auth.get_mode_raw` を `!Auth` なし + `String` 戻り値として登録する。

---

## Phase 4: rune ファイル作成（`runes/auth/`）

### 4-A: `runes/auth/jwt.fav`

```favnir
// runes/auth/jwt.fav — JWT 検証・デコード・署名 (v4.5.0)

// verify_jwt: HS256 署名を検証してクレームを返す
// fav.toml [auth] mode = "cognito" のとき署名検証をスキップ
public fn verify_jwt(token: String, secret: String) -> Result<Map<String, String>, String> !Auth {
    let mode = Auth.get_mode_raw()
    if mode == "cognito" {
        Crypto.jwt_decode_raw(token)
    } else {
        if mode == "none" {
            Crypto.jwt_decode_raw(token)
        } else {
            Crypto.jwt_verify_raw(token, secret, "HS256")
        }
    }
}

// verify_jwt_rs256: RS256（RSA 公開鍵）で署名を検証
public fn verify_jwt_rs256(token: String, public_key_pem: String) -> Result<Map<String, String>, String> !Auth {
    Crypto.jwt_verify_raw(token, public_key_pem, "RS256")
}

// decode_claims: 署名検証なしでクレームをデコード（Cognito / ALB 用）
public fn decode_claims(token: String) -> Result<Map<String, String>, String> !Auth {
    Crypto.jwt_decode_raw(token)
}

// from_cognito_header: ALB の X-Amzn-Oidc-Data ヘッダー値をデコード
// ALB が署名検証済みであることを前提とする（本番環境向け）
public fn from_cognito_header(header_value: String) -> Result<Map<String, String>, String> !Auth {
    Crypto.jwt_decode_raw(header_value)
}

// sign_hs256: HS256 で JWT を署名する（テスト・内部サービス用）
public fn sign_hs256(claims_json: String, secret: String) -> Result<String, String> !Auth {
    Crypto.jwt_sign_raw(claims_json, secret, "HS256")
}
```

### 4-B: `runes/auth/rbac.fav`

```favnir
// runes/auth/rbac.fav — ロールベースアクセス制御 (v4.5.0)

// require_role: claims の "role" フィールドが required_role と一致するか確認
public fn require_role(claims: Map<String, String>, required_role: String) -> Result<Unit, String> {
    let actual = Option.unwrap_or(Map.get(claims, "role"), "")
    if actual == required_role {
        Result.ok(())
    } else {
        Result.err(String.concat("role required: ", String.concat(required_role, String.concat(", got: ", actual))))
    }
}

// require_any_role: roles リストのいずれかが claims の "role" と一致するか確認
public fn require_any_role(claims: Map<String, String>, roles: List<String>) -> Result<Unit, String> {
    let actual = Option.unwrap_or(Map.get(claims, "role"), "")
    if List.contains(roles, actual) {
        Result.ok(())
    } else {
        Result.err(String.concat("none of the required roles matched, got: ", actual))
    }
}

// has_permission: claims の "permissions" フィールド（カンマ区切り）に permission が含まれるか
public fn has_permission(claims: Map<String, String>, permission: String) -> Bool {
    let perms = Option.unwrap_or(Map.get(claims, "permissions"), "")
    let parts = String.split(perms, ",")
    List.contains(parts, permission)
}

// sub: claims から "sub" を取り出す
public fn sub(claims: Map<String, String>) -> Result<String, String> {
    match Map.get(claims, "sub") {
        Some(v) => Result.ok(v)
        None    => Result.err("claims missing 'sub' field")
    }
}

// role: claims から "role" を取り出す
public fn role(claims: Map<String, String>) -> Result<String, String> {
    match Map.get(claims, "role") {
        Some(v) => Result.ok(v)
        None    => Result.err("claims missing 'role' field")
    }
}
```

### 4-C: `runes/auth/apikey.fav`

```favnir
// runes/auth/apikey.fav — API キー生成・検証 (v4.5.0)

// generate_api_key: prefix + 32バイト乱数で API キーを生成
// フォーマット: {prefix}_{64文字hex}
public fn generate_api_key(prefix: String) -> String !Auth {
    let rand_hex = Crypto.random_hex_raw(32)
    String.concat(prefix, String.concat("_", rand_hex))
}

// hmac_tag: API キーの HMAC-SHA256 タグを計算（DB 保存用）
// キー自体は保存せず、このタグと照合する
public fn hmac_tag(key: String, secret: String) -> String !Auth {
    Crypto.hmac_sha256_raw(secret, key)
}

// verify_by_tag: 提示キーのタグを計算し stored_tag と比較
public fn verify_by_tag(key: String, secret: String, stored_tag: String) -> Bool !Auth {
    let computed = Crypto.hmac_sha256_raw(secret, key)
    computed == stored_tag
}
```

### 4-D: `runes/auth/oauth2.fav`

```favnir
// runes/auth/oauth2.fav — OAuth2 認可フロー (v4.5.0)

// authorization_url: OAuth2 認可エンドポイント URL を組み立てる（純粋関数）
public fn authorization_url(
    endpoint: String,
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String
) -> String {
    String.concat(endpoint,
        String.concat("?client_id=", String.concat(client_id,
        String.concat("&redirect_uri=", String.concat(redirect_uri,
        String.concat("&scope=", String.concat(scope,
        String.concat("&state=", String.concat(state,
        "&response_type=code")))))))))
}

// exchange_code: 認可コードをトークンに交換（HTTP POST to token_endpoint）
// 返値: Map には "access_token" / "token_type" / "expires_in" 等が含まれる
public fn exchange_code(
    token_endpoint: String,
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String
) -> Result<Map<String, String>, String> !Auth !Network {
    let body = String.concat("grant_type=authorization_code",
               String.concat("&client_id=", String.concat(client_id,
               String.concat("&client_secret=", String.concat(client_secret,
               String.concat("&code=", String.concat(code,
               String.concat("&redirect_uri=", redirect_uri))))))))
    match Http.post_raw(token_endpoint, body) {
        Ok(resp) => match resp.status < 300 {
            true  => Result.ok(Http.parse_json_map_raw(resp.body))
            false => Result.err(String.concat("token exchange failed: ", resp.body))
        }
        Err(e) => Result.err(e.message)
    }
}

// refresh_token: リフレッシュトークンで新しいアクセストークンを取得
public fn refresh_token(
    token_endpoint: String,
    client_id: String,
    client_secret: String,
    refresh_token_val: String
) -> Result<Map<String, String>, String> !Auth !Network {
    let body = String.concat("grant_type=refresh_token",
               String.concat("&client_id=", String.concat(client_id,
               String.concat("&client_secret=", String.concat(client_secret,
               String.concat("&refresh_token=", refresh_token_val))))))
    match Http.post_raw(token_endpoint, body) {
        Ok(resp) => match resp.status < 300 {
            true  => Result.ok(Http.parse_json_map_raw(resp.body))
            false => Result.err(String.concat("refresh failed: ", resp.body))
        }
        Err(e) => Result.err(e.message)
    }
}
```

> **注意**: `oauth2.fav` は `Http.post_raw` と `Http.parse_json_map_raw` を直接呼ぶ。
> `Http.parse_json_map_raw` は vm.rs に新規追加する必要がある（JSON 文字列 → `Map<String,String>`）。
> これが複雑な場合は v4.5.0 では `exchange_code` / `refresh_token` をスタブ（`!Network` を呼ぶ旨コメント）として残し、実装は v4.6.0 に回す。

### 4-E: `runes/auth/auth.fav`（barrel file）

```favnir
// runes/auth/auth.fav — Auth Rune public API (v4.5.0)
use jwt.{ verify_jwt, verify_jwt_rs256, decode_claims, from_cognito_header, sign_hs256 }
use rbac.{ require_role, require_any_role, has_permission, sub, role }
use apikey.{ generate_api_key, hmac_tag, verify_by_tag }
use oauth2.{ authorization_url, exchange_code, refresh_token }
```

### 4-F: `runes/auth/auth.test.fav`

テスト用の定数（HS256 テストトークン生成 → 検証）を使用した 12 件以上のテスト。
JWT 生成には `Gen.jwt_sign_raw` を使うため、テスト内で `Crypto.jwt_sign_raw` を直接呼ぶ。

主なテスト:
1. `jwt sign and verify roundtrip`
2. `jwt verify wrong secret returns err`
3. `jwt decode no verify returns claims`
4. `require_role match returns ok`
5. `require_role mismatch returns err`
6. `require_any_role match returns ok`
7. `require_any_role all mismatch returns err`
8. `has_permission found returns true`
9. `has_permission missing returns false`
10. `generate_api_key has correct prefix`
11. `hmac_tag and verify_by_tag roundtrip`
12. `verify_by_tag wrong key returns false`
13. `authorization_url contains client_id`
14. `authorization_url contains response_type_code`

---

## Phase 5: テスト追加

### 5-A: `fav/src/backend/vm_stdlib_tests.rs`（6 件）

```rust
fn crypto_jwt_sign_and_verify_roundtrip() {
    // sign_raw で生成したトークンを verify_raw で検証
    // claims の "sub" が "user1" になることを確認
}

fn crypto_jwt_verify_invalid_signature_returns_err() {
    // secret が違うと verify_raw が Err を返す
}

fn crypto_jwt_decode_no_verify() {
    // decode_raw はどんな secret でも claims を返す
}

fn crypto_hmac_sha256_known_value() {
    // HMAC-SHA256("key", "data") が既知の hex 値と一致
}

fn crypto_sha256_known_value() {
    // SHA-256("hello") が既知の hex 値と一致
}

fn crypto_random_hex_length() {
    // random_hex_raw(16) が 32 文字を返す
}
```

### 5-B: `fav/src/driver.rs` 統合テスト（5 件）

```rust
fn auth_rune_test_file_passes()
fn jwt_verify_in_favnir_source()          // sign → verify が Ok
fn require_role_in_favnir_source()        // 一致 Ok / 不一致 Err
fn api_key_generate_verify_in_favnir_source()
fn oauth2_authorization_url_in_favnir_source()
```

---

## Phase 6: examples 追加（`examples/auth_demo/`）

```
examples/auth_demo/
  fav.toml
  src/
    main.fav
```

`main.fav` のデモ内容:
1. `Crypto.jwt_sign_raw` でテストトークンを生成
2. `auth.verify_jwt` で検証 → claims を表示
3. `auth.require_role` で認可チェック（成功・失敗の両方を表示）
4. `auth.generate_api_key` で API キーを生成 → `auth.hmac_tag` → `auth.verify_by_tag`
5. `auth.authorization_url` で OAuth2 認可 URL を組み立て・表示
6. `auth.sub` / `auth.role` でクレームフィールドを取り出す

---

## 実装順序と依存関係

```
Phase 0 (バージョン更新 + cargo 依存追加)
  ↓
Phase 1 (VM プリミティブ: 1-A〜1-F)  ← コア実装
  ↓                      ↓
Phase 2 (checker.rs)   Phase 3 (toml.rs)  ← 並列可
  ↓
Phase 4 (rune ファイル)
  ↓
Phase 5 (テスト)
  ↓
Phase 6 (examples)
```

Phase 1 の各サブタスクは独立しており、`1-A`（jwt_verify）が最優先。
`1-D`/`1-E`/`1-F`（hmac/sha2/random）は `Phase 4` の `apikey.fav` の前提。

---

## リスクと対策

| リスク | 影響 | 対策 |
|--------|------|------|
| `jsonwebtoken = "9"` のコンパイル時間 | ビルド遅延 | 初回のみ。Windows MSVC では `ring` クレートのビルドに時間がかかる可能性あり |
| RS256 の PEM パース失敗 | jwt_verify_raw の RS256 実装不全 | HS256 を優先実装し、RS256 は spec 記載のみで v4.5.0 は HS256 のみテストする |
| `oauth2.fav` の `Http.parse_json_map_raw` 未実装 | exchange_code / refresh_token が動かない | oauth2.fav の HTTP 連携部分は型チェックのみとし、実際の HTTP 呼び出しは v4.6.0 で完全実装 |
| `!Auth` エフェクトの追加による既存テスト破損 | テスト失敗 | `Crypto.*` の呼び出しは既存コードに存在しないため影響なし |
| `Effect::Auth` 追加による match exhaustiveness エラー | コンパイル失敗 | `Effect` に関する全 match 式に `Auth` アームを追加する |
| `Auth.get_mode_raw` を thread_local で管理する際のデフォルト値 | mode = "" でエラー | デフォルトは `"jwt"` に設定し、`cmd_run` で toml から読んで上書きする |
