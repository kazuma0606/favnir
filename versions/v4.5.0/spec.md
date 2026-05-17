# Favnir v4.5.0 仕様書 — Auth Rune（JWT / RBAC / API Key / OAuth2）

作成日: 2026-05-17

## 概要

認証・認可を Favnir の型システムに統合する。
`!Auth` エフェクトにより、認証コードが明示的にマークされる。
ローカル開発では Rust の crypto primitive で完全検証し、
AWS 本番では ALB + Cognito のプリベリファイドヘッダーを信頼する設計。

---

## 1. 設計方針

### 1.1 `!Auth` エフェクト

すべての `Crypto.*` 呼び出しは `!Auth` エフェクトを要求する。
エフェクトを宣言していない関数から `auth.*` を呼ぶとコンパイルエラーになる。

```favnir
// !Auth を宣言しない関数から呼ぶとエラー
public fn get_user(token: String) -> Result<User, String> {
    bind claims <- auth.verify_jwt(token, Env.get_or("JWT_SECRET", ""))  // E0xxx: !Auth required
    ...
}

// 正しい宣言
public fn get_user(token: String) -> Result<User, String> !Auth {
    bind claims <- auth.verify_jwt(token, Env.get_or("JWT_SECRET", ""))
    ...
}
```

### 1.2 ローカル vs AWS 本番

| 環境 | 動作 |
|------|------|
| ローカル開発 | `Crypto.jwt_verify_raw` で HMAC/RSA 署名を Rust で完全検証 |
| AWS 本番（ALB + Cognito） | `Crypto.jwt_decode_raw` でクレーム取得のみ（ALB が署名検証済み） |

`fav.toml` の `[auth] mode` 設定で切り替える（rune 層が mode を参照）:

```toml
[auth]
mode = "jwt"      # jwt | cognito | none
```

- `jwt`: `auth.verify_jwt` が `Crypto.jwt_verify_raw` で署名検証する
- `cognito`: `auth.verify_jwt` が `Crypto.jwt_decode_raw` のみ実行（ALB 信頼）
- `none`: `auth.verify_jwt` は常に成功（開発・テスト用）

### 1.3 既存型との関係

| 概念 | 型 | 備考 |
|------|----|------|
| JWT クレーム | `Map<String, String>` | 汎用 Map で表現（専用型なし） |
| エラー | `String` | シンプルなメッセージ文字列 |
| エフェクト | `!Auth` | 新規追加（他エフェクトと独立） |

---

## 2. Cargo 依存追加

```toml
[dependencies]
jsonwebtoken = "9"  # JWT encode/decode/verify (HS256 / RS256)
hmac = "0.12"       # HMAC-SHA256 汎用実装
sha2 = "0.10"       # SHA-256 ハッシュ
```

`jsonwebtoken` が HS256 / RS256 を担当し、`hmac` + `sha2` は API キー生成等の汎用 crypto に使用する。

---

## 3. VM プリミティブ（`Crypto.*`）

すべて `!Auth` を要求する。戻り値は `Result<_, String>` または純粋な `String`。

### 3-A: JWT 検証・デコード

```
Crypto.jwt_verify_raw(token, secret, alg) -> Result<Map<String,String>, String>
```
- `alg`: `"HS256"` または `"RS256"`
- `"HS256"`: `secret` は共有シークレット文字列
- `"RS256"`: `secret` は RSA 公開鍵の PEM 文字列
- 署名・有効期限（`exp`）を検証し、成功したらクレーム Map を返す
- クレーム値はすべて `String` に変換（Int → `"1234"`、Bool → `"true"`）

```
Crypto.jwt_decode_raw(token) -> Result<Map<String,String>, String>
```
- 署名検証なしでペイロードをデコード
- ALB + Cognito の pre-verified ヘッダー用（本番環境向け）

```
Crypto.jwt_sign_raw(claims_json, secret, alg) -> Result<String, String>
```
- `claims_json`: `'{"sub":"user1","role":"admin","exp":9999999999}'` 形式の JSON 文字列
- テスト用トークン生成に使用（本番では秘密鍵管理が必要）

### 3-B: HMAC / ハッシュ

```
Crypto.hmac_sha256_raw(key, data) -> String
```
- `key` / `data` は UTF-8 文字列
- 返値: 小文字 hex エンコードされた 64 文字 HMAC-SHA256 ダイジェスト

```
Crypto.sha256_raw(data) -> String
```
- 返値: 小文字 hex エンコードされた 64 文字 SHA-256 ダイジェスト

### 3-C: ランダム生成

```
Crypto.random_hex_raw(n) -> String
```
- `n` バイトの暗号学的安全な乱数を生成し、小文字 hex 文字列（`2n` 文字）で返す
- API キーのランダム部分生成に使用
- `rand::rngs::OsRng` を使用（シードに依存しない）

---

## 4. checker.rs への変更

### 4-A: `Effect::Auth` の追加

```rust
pub enum Effect {
    Io, Db, Network, Random, Checkpoint, Rpc, Aws,
    Auth,  // v4.5.0 新規追加
}
```

### 4-B: `require_auth_effect` の追加

`check_builtin_apply` で `("Crypto", _)` のすべてのアームに `self.require_auth_effect(span)` を付与する。

### 4-C: `Crypto.*` の戻り値型

| メソッド | 戻り値型 |
|---------|---------|
| `jwt_verify_raw` | `Result<Map<String,String>, String>` |
| `jwt_decode_raw` | `Result<Map<String,String>, String>` |
| `jwt_sign_raw` | `Result<String, String>` |
| `hmac_sha256_raw` | `String` |
| `sha256_raw` | `String` |
| `random_hex_raw` | `String` |
| `_`（その他） | `Unknown` |

### 4-D: `"Crypto"` namespace の登録

`compiler.rs` の global loop に `"Crypto"` を追加する（既存の `"Gen"`, `"Validate"` と同様）。

---

## 5. `fav.toml` 拡張（`toml.rs`）

```rust
#[derive(Debug, Clone, Default)]
pub struct AuthConfig {
    pub mode: String,  // "jwt" | "cognito" | "none"; デフォルト "jwt"
}

pub struct FavToml {
    // ... 既存フィールド
    pub auth: Option<AuthConfig>,
}
```

`Auth.mode_raw()` VM primitive でアクセス可能にする（optional）。
実際の切り替えは rune 層（`jwt.fav`）で `Auth.get_mode_raw()` を呼んで分岐する。

---

## 6. rune ファイル構成

```
runes/auth/
  auth.fav        ← public API（barrel file）
  jwt.fav         ← JWT 検証・デコード・署名
  rbac.fav        ← ロールベースアクセス制御
  apikey.fav      ← API キー生成・検証
  oauth2.fav      ← OAuth2 認可フロー
  auth.test.fav   ← テスト
```

---

## 7. rune API 仕様

### 7-A: `jwt.fav`

```favnir
// verify_jwt: HMAC-SHA256 署名を検証して claims を返す
// mode = "cognito" のとき署名検証をスキップ
public fn verify_jwt(token: String, secret: String) -> Result<Map<String, String>, String> !Auth

// verify_jwt_rs256: RSA 公開鍵で署名を検証
public fn verify_jwt_rs256(token: String, public_key_pem: String) -> Result<Map<String, String>, String> !Auth

// decode_claims: 署名検証なしでクレームをデコード（Cognito / ALB 用）
public fn decode_claims(token: String) -> Result<Map<String, String>, String> !Auth

// from_cognito_header: ALB の X-Amzn-Oidc-Data ヘッダー値をデコード
// ALB が署名検証済みであることを前提とする
public fn from_cognito_header(header_value: String) -> Result<Map<String, String>, String> !Auth

// sign_hs256: テスト用 JWT トークン生成（シークレットで署名）
public fn sign_hs256(claims_json: String, secret: String) -> Result<String, String> !Auth
```

### 7-B: `rbac.fav`

```favnir
// require_role: claims に指定ロールがなければ Err を返す
// claims の "role" フィールドを参照する
public fn require_role(claims: Map<String, String>, required_role: String) -> Result<Unit, String>

// require_any_role: いずれかのロールがあれば Ok
public fn require_any_role(claims: Map<String, String>, roles: List<String>) -> Result<Unit, String>

// has_permission: claims の "permissions" フィールド（カンマ区切り）に permission が含まれるか
public fn has_permission(claims: Map<String, String>, permission: String) -> Bool

// sub: claims から "sub"（subject）を取り出す
public fn sub(claims: Map<String, String>) -> Result<String, String>

// role: claims から "role" を取り出す
public fn role(claims: Map<String, String>) -> Result<String, String>
```

### 7-C: `apikey.fav`

```favnir
// generate_api_key: prefix + 乱数で API キーを生成する
// フォーマット: {prefix}_{32バイト乱数のhex}
public fn generate_api_key(prefix: String) -> String !Auth

// hmac_tag: HMAC-SHA256 で API キーのタグを生成（保存用）
// API キーをそのまま保存せず、このタグを DB に保存して verify で照合する
public fn hmac_tag(key: String, secret: String) -> String !Auth

// verify_by_tag: 提示されたキーから tag を計算し stored_tag と比較
// constant-time 比較ではないが実用上は十分（タイミング攻撃はレート制限で対処）
public fn verify_by_tag(key: String, secret: String, stored_tag: String) -> Bool !Auth
```

### 7-D: `oauth2.fav`

```favnir
// authorization_url: OAuth2 認可エンドポイント URL を組み立てる（純粋関数）
public fn authorization_url(
    endpoint: String,
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String
) -> String

// exchange_code: 認可コードをアクセストークンと交換（HTTP POST）
public fn exchange_code(
    token_endpoint: String,
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String
) -> Result<Map<String, String>, String> !Auth !Network

// refresh_token: リフレッシュトークンで新しいアクセストークンを取得
public fn refresh_token(
    token_endpoint: String,
    client_id: String,
    client_secret: String,
    refresh_token_val: String
) -> Result<Map<String, String>, String> !Auth !Network
```

---

## 8. 使用イメージ

### 8-A: HTTP ハンドラでの JWT 検証

```favnir
import rune "auth"
import rune "http"

type Claims = { sub: String role: String exp: String }

public fn handle_orders(token: String) -> Result<String, String> !Auth {
    bind claims <- auth.verify_jwt(token, Env.get_or("JWT_SECRET", "dev-secret"))
    bind _      <- auth.require_role(claims, "data_engineer")
    bind sub    <- auth.sub(claims)
    Result.ok(String.concat("Hello, ", sub))
}
```

### 8-B: API キー認証フロー

```favnir
import rune "auth"

public fn create_api_key(prefix: String, store_secret: String) -> Map<String, String> !Auth {
    let key = auth.generate_api_key(prefix)
    let tag = auth.hmac_tag(key, store_secret)
    // tag を DB に保存し、key をユーザーに返す
    Map.set(Map.set((), "key", key), "tag", tag)
}

public fn verify_request_api_key(key: String, store_secret: String, stored_tag: String) -> Bool !Auth {
    auth.verify_by_tag(key, store_secret, stored_tag)
}
```

### 8-C: OAuth2 フロー

```favnir
import rune "auth"

public fn start_oauth(client_id: String, redirect_uri: String) -> String {
    auth.authorization_url(
        "https://accounts.google.com/o/oauth2/v2/auth",
        client_id, redirect_uri,
        "openid email profile",
        "random-state-value"
    )
}

public fn handle_callback(code: String, client_id: String, client_secret: String) -> Result<Map<String, String>, String> !Auth !Network {
    auth.exchange_code(
        "https://oauth2.googleapis.com/token",
        client_id, client_secret,
        code, "https://myapp.example.com/callback"
    )
}
```

---

## 9. テスト方針

### 9-A: vm_stdlib_tests.rs（6 件）

| テスト名 | 内容 |
|---------|------|
| `crypto_jwt_sign_and_verify_roundtrip` | sign_raw → verify_raw が同じ claims を返す |
| `crypto_jwt_verify_invalid_signature_returns_err` | 改ざんトークンが Err を返す |
| `crypto_jwt_decode_no_verify` | decode_raw が署名なしでクレームを返す |
| `crypto_hmac_sha256_known_value` | 既知入力で期待される hex を返す |
| `crypto_sha256_known_value` | 既知入力で期待される hex を返す |
| `crypto_random_hex_length` | `random_hex_raw(16)` が 32 文字を返す |

### 9-B: driver.rs 統合テスト（5 件）

| テスト名 | 内容 |
|---------|------|
| `auth_rune_test_file_passes` | auth.test.fav 全件 pass |
| `jwt_verify_in_favnir_source` | Favnir ソースで sign → verify が Ok |
| `require_role_in_favnir_source` | role 一致で Ok、不一致で Err |
| `api_key_generate_verify_in_favnir_source` | generate → hmac_tag → verify_by_tag |
| `oauth2_authorization_url_in_favnir_source` | URL 組み立て結果が期待文字列を含む |

### 9-C: `runes/auth/auth.test.fav`（12 件以上）

- JWT 検証：正常・期限切れ・改ざん・デコードのみ
- RBAC：require_role 正常・エラー・require_any_role・has_permission
- API キー：generate_api_key フォーマット・hmac_tag 一致・verify_by_tag 正常/不正
- OAuth2：authorization_url の URL 組み立て

---

## 10. 完了条件

- `cargo build` が通る
- 既存 837 件が全て pass
- 新規テスト 23 件以上が pass（Rust 11 件 + Favnir 12 件以上）
- `auth.verify_jwt` が HS256 トークンを正しく検証できる
- 改ざんトークン・期限切れトークンが Err を返す
- `auth.require_role` がロール不一致で Err を返す
- `auth.generate_api_key` が `{prefix}_` で始まる 64+ 文字のキーを生成する
- `auth.authorization_url` が有効な OAuth2 URL を組み立てる
- `examples/auth_demo/` が `fav run` で動く
