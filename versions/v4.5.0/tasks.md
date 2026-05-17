# Favnir v4.5.0 タスクリスト — Auth Rune

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新 + 依存追加

- [x] `fav/Cargo.toml` の version を `"4.5.0"` に変更
- [x] `fav/Cargo.toml` に `jsonwebtoken = "9"` を追加
- [x] `fav/Cargo.toml` に `hmac = "0.12"` を追加
- [x] `fav/Cargo.toml` に `sha2 = "0.10"` を追加
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.5.0` に更新

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: `Crypto.jwt_verify_raw`

- [x] `vm_call_builtin` に `"Crypto.jwt_verify_raw"` アームを追加
- [x] `args[0]` から token (String)、`args[1]` から secret (String)、`args[2]` から alg (String) を取り出す
- [x] `alg == "HS256"` のとき `DecodingKey::from_secret` + `Validation::new(Algorithm::HS256)` で検証
- [x] `alg == "RS256"` のとき `DecodingKey::from_rsa_pem` + `Validation::new(Algorithm::RS256)` で検証
- [x] `jsonwebtoken::decode::<serde_json::Value>` で検証後にクレームを `HashMap<String, VMValue>` に変換
- [x] 成功: `Ok(ok_vm(VMValue::Record(claims_map)))`、失敗: `Ok(err_vm(VMValue::Str(e.to_string())))`

### 1-B: `Crypto.jwt_decode_raw`

- [x] `vm_call_builtin` に `"Crypto.jwt_decode_raw"` アームを追加
- [x] jsonwebtoken v9 の `Validation::new(HS256)` + `insecure_disable_signature_validation()` + `validate_exp = false` + dummy key で署名検証なしデコード
- [x] クレーム変換は `json_value_to_vm_claims_map` 共通関数に切り出し

### 1-C: `Crypto.jwt_sign_raw`

- [x] `vm_call_builtin` に `"Crypto.jwt_sign_raw"` アームを追加
- [x] `EncodingKey::from_secret` + `Header::new(Algorithm::HS256)` でトークン生成

### 1-D: `Crypto.hmac_sha256_raw`

- [x] `Hmac::<Sha256>::new_from_slice` → `mac.update` → `mac.finalize` → hex エンコード（64 文字）

### 1-E: `Crypto.sha256_raw`

- [x] `Sha256::new()` → `hasher.update` → `hasher.finalize` → hex エンコード（64 文字）

### 1-F: `Crypto.random_hex_raw`

- [x] `rand::rngs::OsRng.fill_bytes` で n バイト生成 → hex エンコード（2n 文字）

### 1-G: `Auth.get_mode_raw`

- [x] `AUTH_MODE` thread_local + `set_auth_mode(mode: &str)` 関数を追加
- [x] `vm_call_builtin` に `"Auth.get_mode_raw"` アームを追加

---

## Phase 2: checker.rs への変更（`fav/src/middle/checker.rs`）

- [x] `"Auth"` を `BUILTIN_EFFECTS` に追加（`Effect::Unknown("Auth")` として扱う、新バリアント追加なし）
- [x] `require_auth_effect(&mut self, span: &Span)` メソッドを追加（エラーコード `E0311`）
- [x] `check_builtin_apply` に `("Crypto", *)` アームを追加（各関数 + フォールバック）
- [x] `check_builtin_apply` に `("Auth", "get_mode_raw")` アームを追加
- [x] `register_builtins` に `"Crypto"` / `"Auth"` namespace を追加
- [x] `check_test_def` のエフェクトリストに `Effect::Unknown("Auth")` / `Effect::Unknown("DuckDb")` を追加

---

## Phase 3: `fav.toml` 拡張（`fav/src/toml.rs`）

- [x] `AuthConfig { mode: String }` 構造体を追加
- [x] `FavToml` に `pub auth: Option<AuthConfig>` フィールドを追加
- [x] `parse_fav_toml` に `[auth]` セクションのパース処理を追加
- [x] `driver.rs` の `cmd_run` で `toml.auth` から `set_auth_mode` を呼ぶ
- [x] `FavToml` のリテラル初期化箇所（checker.rs ×2、resolver.rs ×2、driver.rs ×1）に `auth: None` を追加

---

## Phase 4: rune ファイル作成

### 4-A: `runes/auth/jwt.fav`

- [x] `verify_jwt` — `Auth.get_mode_raw()` でモード判定、jwt/cognito/none で分岐
- [x] `verify_jwt_rs256` — RS256 検証
- [x] `decode_claims` — 署名なしデコード
- [x] `from_cognito_header` — ALB ヘッダーデコード
- [x] `sign_hs256` — HS256 署名

### 4-B: `runes/auth/rbac.fav`

- [x] `require_role` — `Map.get(claims, "role") == required_role` で判定（`let` 不使用・インライン）
- [x] `require_any_role` — `List.any(roles, |r| r == ...)` で判定（`List.contains` 不使用）
- [x] `has_permission` — `List.any(String.split(...), |p| p == permission)`
- [x] `sub` / `role` — クレーム抽出

### 4-C: `runes/auth/apikey.fav`

- [x] `generate_api_key` — `String.concat(prefix, String.concat("_", Crypto.random_hex_raw(32)))`
- [x] `hmac_tag` — `Crypto.hmac_sha256_raw(secret, key)`
- [x] `verify_by_tag` — タグ再計算して一致確認

### 4-D: `runes/auth/oauth2.fav`

- [x] `authorization_url` — 純粋関数、クエリパラメータ組み立て
- [x] `exchange_code` / `refresh_token` — スタブ実装（`Result.err("not implemented")`）

### 4-E: `runes/auth/auth.fav`

- [x] barrel file として全サブモジュールを `use` で集約

### 4-F: `runes/auth/auth.test.fav`

- [x] 14 件のテストを実装（`let` 不使用、`List.of` → `String.split`、`List.contains` → `List.any`）
  - jwt sign/verify roundtrip
  - jwt verify wrong secret returns err
  - jwt decode no verify returns claims
  - hmac sha256 returns 64 char hex
  - sha256 returns 64 char hex
  - random hex 16 returns 32 chars
  - require_role match returns ok
  - require_role mismatch returns err
  - require_any_role match returns ok
  - require_any_role all mismatch returns err
  - has_permission found returns true
  - generate_api_key has prefix
  - hmac_tag and verify_by_tag roundtrip
  - authorization_url contains client_id

---

## Phase 5: テスト追加

### 5-A: `fav/src/backend/vm_stdlib_tests.rs`（6 件）

- [x] `crypto_jwt_sign_and_verify_roundtrip`
- [x] `crypto_jwt_verify_invalid_signature_returns_err`
- [x] `crypto_jwt_decode_no_verify`
- [x] `crypto_hmac_sha256_known_value`（`let` 不使用・インライン）
- [x] `crypto_sha256_known_value`
- [x] `crypto_random_hex_length`

### 5-B: `fav/src/driver.rs` 統合テスト（5 件）

- [x] `auth_rune_test_file_passes`
- [x] `jwt_verify_in_favnir_source`
- [x] `require_role_in_favnir_source`（`let` 不使用・インライン）
- [x] `api_key_generate_verify_in_favnir_source`（`let` 不使用・インライン）
- [x] `oauth2_authorization_url_in_favnir_source`（`let` 不使用・インライン）

---

## Phase 6: examples 追加

- [x] `examples/auth_demo/fav.toml` を作成
- [x] `examples/auth_demo/src/main.fav` を作成（`let` 不使用・インライン）

---

## 完了条件

- [x] `cargo build` が通る
- [x] 既存 837 件が全て pass
- [x] 新規テスト 11 件（Rust）+ 14 件（Favnir）= 25 件以上が pass
- [x] `Crypto.jwt_verify_raw(token, secret, "HS256")` が正しいクレームを返す
- [x] 改ざんトークン・誤った secret が Err を返す
- [x] `Crypto.jwt_decode_raw` が署名なしでクレームを返す
- [x] `Crypto.hmac_sha256_raw` / `Crypto.sha256_raw` が 64 文字 hex を返す
- [x] `Crypto.random_hex_raw(16)` が 32 文字を返す
- [x] `auth.require_role` がロール不一致で Err を返す
- [x] `auth.generate_api_key` が `{prefix}_` で始まるキーを返す
- [x] `auth.authorization_url` が有効なクエリパラメータを含む URL を返す
- [x] `!Auth` 未宣言の関数から `Crypto.*` を呼ぶとコンパイルエラー（E0311）
- [x] `examples/auth_demo/` が `fav run` で動く

---

## 実装メモ（次バージョンへの引き継ぎ）

- **`let` は Favnir のキーワードではない** — `Ident("let")` としてレキシング。ブロック内では `let x = expr` 構文は使えない。値は必ずインラインにするか `bind`（Task<T> のみ）を使う。
- **`List.contains` は存在しない** — `List.any(list, |x| x == val)` を使う。
- **`List.of` は存在しない** — `String.split("a,b,c", ",")` で `List<String>` を生成する。
- **`!Auth` エフェクトは `Effect::Unknown("Auth")` として実装** — 新バリアント追加不要（exhaustive match 変更ゼロ）。
- **jsonwebtoken v9 の decode-no-verify** — `dangerous_insecure_decode` は存在しない。`Validation::new(HS256)` + `insecure_disable_signature_validation()` + `validate_exp = false` + `DecodingKey::from_secret(b"")` を使う。
- **テストブロックのエフェクト** — `check_test_def` で `[Io, File, Unknown("Auth"), Unknown("DuckDb")]` を設定済み。テストブロック内で `Crypto.*` を直接呼び出せる。

**最終テスト結果: 848 passed; 0 failed**
