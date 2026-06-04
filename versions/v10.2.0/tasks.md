# Favnir v10.2.0 Tasks

Date: 2026-06-04
Theme: VM Primitive 追加 — Snowflake SQL API v2

---

## Phase A: JWT ヘルパー実装（`vm.rs`）

- [x] A-1: `SnowflakeClaims` 構造体を追加（`iss` / `sub` / `iat` / `exp`）
- [x] A-2: `snowflake_generate_jwt(account, user, private_key_pem, public_key_fp)` 関数を追加
  - `account.to_uppercase()` / `user.to_uppercase()`
  - `iss`: `"{ACCOUNT}.{USER}.SHA256:{FP}"`、`sub`: `"{ACCOUNT}.{USER}"`
  - `Header::new(Algorithm::RS256)` + `EncodingKey::from_rsa_pem`
- [x] A-3: `snowflake_api_post(account, jwt, body)` 関数を追加
  - `POST https://{account}.snowflakecomputing.com/api/v2/statements`
  - ヘッダー: `Authorization: Bearer {jwt}` / `X-Snowflake-Authorization-Token-Type: KEYPAIR_JWT`
  - ureq エラーを `Err(String)` に変換
- [x] A-4: `snowflake_read_env(key)` ヘルパーを追加（未設定時に `Err("{key} is not set")`）
- [x] A-5: `cargo build` 通過確認

---

## Phase B: `call_builtin` に Snowflake primitive を追加

- [x] B-1: `"Snowflake.execute_raw"` アームを追加
  - 引数から `sql: String` を取り出す
  - 環境変数 4 件（ACCOUNT / USER / PRIVATE_KEY / PUBLIC_KEY_FP）を読み取る
  - `snowflake_generate_jwt` で JWT 生成
  - リクエストボディ構築（WAREHOUSE / ROLE / DATABASE / SCHEMA はオプション）
  - `snowflake_api_post` 呼び出し → `Ok("ok")` または `Err(msg)`
- [x] B-2: `"Snowflake.query_raw"` アームを追加
  - B-1 と同じ前半処理（環境変数・JWT・ボディ構築）
  - レスポンス変換: `resultSetMetaData.rowType` + `data` → JSON オブジェクト配列文字列
- [x] B-3: `cargo build` 通過確認

---

## Phase C: テスト追加

- [x] C-1: `v10200_tests` モジュールを追加（3 件）— `vm.rs` 末尾
  - [x] C-1a: `snowflake_execute_raw_missing_env_returns_err`
    - 環境変数なしで execute_raw を呼ぶ → `Err("SNOWFLAKE_ACCOUNT is not set")`
  - [x] C-1b: `snowflake_query_raw_missing_env_returns_err`
    - 環境変数なしで query_raw を呼ぶ → `Err("SNOWFLAKE_ACCOUNT is not set")`
  - [x] C-1c: `snowflake_jwt_well_formed`
    - テスト用 RSA キーペアを埋め込み（test-only、本番鍵とは無関係）
    - `snowflake_generate_jwt` を呼んでトークンが 3 パート形式か確認
    - `jsonwebtoken::decode` でペイロードを取り出し `iss` / `sub` / `iat` / `exp` を検証
- [x] C-2: `cargo test v10200` — 3 件通過
  - `compiler.rs` の Rust builtin NS リスト 2 箇所に `"Snowflake"` を追加

---

## Phase D: 完了処理

- [x] D-1: `cargo test bootstrap_full_self_hosting` — Windows でスタック制限のため実行不可（pre-existing issue、CI は `#[ignore]` で管理）
- [x] D-2: `fav check --legacy-check self/compiler.fav` — エラーなし（CI と同様のフラグを使用）
- [x] D-3: `fav check self/checker.fav` — エラーなし
- [x] D-4: `cargo test` — 全件通過（1264 件）
- [x] D-5: `fav/Cargo.toml` version → `"10.2.0"`
- [x] D-6: `fav/self/cli.fav` の `run_version` → `"10.2.0"`
- [x] D-7: 本ファイル完了チェック
- [ ] D-8: `memory/MEMORY.md` に v10.2.0 完了を記録
- [ ] D-9: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `vm.rs` に `Snowflake.execute_raw` / `Snowflake.query_raw` が追加されている | ✅ |
| 環境変数未設定時に `Err("SNOWFLAKE_ACCOUNT is not set")` が返る | ✅ |
| JWT の `iss` が `ACCOUNT.USER.SHA256:FP` 形式になっている | ✅ |
| `cargo test v10200` — 3 件通過 | ✅ |
| `cargo test bootstrap_full_self_hosting` 維持 | ⚠️ Windows stack 制限（pre-existing、CI は #[ignore]） |
| `cargo test` 全件通過 | ✅ 1264 件 |

---

## スコープ外（後続バージョンへ）

- `Effect::Snowflake` 追加（8 ファイル更新）→ v10.3.0
- `checker.fav` に Snowflake 型シグネチャ追加 → v10.4.0
- `compiler.fav` の builtin NS に `"Snowflake"` 追加 → v10.5.0
- `runes/snowflake/` 実装 → v10.6.0
- フィンガープリント自動計算（秘密鍵から動的導出）→ 検討中
