# v14.4.0 Tasks — AWS Rune 正式パッケージング (runes/aws/)

Date: 2026-06-12
Branch: master

---

## Phase A — `fav/src/backend/vm.rs`: 新規 VM プリミティブ追加

- [ ] A-1: `Ctx.aws_get_field_raw` ハンドラ追加（`Ctx.azure_get_field_raw` の直後）
  - 引数 2 個: `ctx: AwsCtx（文字列）`, `field: String`
  - `"ok({...})"` または生 JSON をパースして指定フィールドを返す
  - 返り値: `Value::String(field_value)`
  - パターン: `plan.md` Phase A-1 参照

- [ ] A-2: `AWS.secrets_get_raw` ハンドラ追加（DynamoDB primitives ブロックの後）
  - 引数 2 個: `region: String`, `secret_name: String`
  - ureq + SigV4 で Secrets Manager `GetSecretValue` API を呼ぶ
  - `sqs_send_message_raw` のパターンを参照（service = "secretsmanager"）
  - 返り値: `Value::String("ok(secret_string)")` or `Value::String("err(message)")`

- [ ] A-3: `cargo build` でコンパイルエラーなし確認

---

## Phase B — `fav/src/middle/checker.rs`: builtin_ret_ty 追加

- [ ] B-1: `builtin_ret_ty` に `Ctx.aws_get_field_raw` 追加
  ```rust
  ("Ctx", "aws_get_field_raw") => Some(Type::String),
  ```
  （`("Ctx", "azure_get_field_raw")` の直後）

- [ ] B-2: `("AWS", "secrets_get_raw")` を `("AWS", ...)` ブロックに追加
  - `require_aws_effect(span)` を呼び出す
  - `Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))` を返す

- [ ] B-3: `ns_env_def` の `"Ctx"` ブロックに `"aws_get_field_raw"` を追加
  （E0007 防止のため namespace に登録）

- [ ] B-4: `cargo build` でコンパイルエラーなし確認

---

## Phase C — `runes/aws/secrets.fav`: 新規作成

- [ ] C-1: `C:\Users\yoshi\favnir\runes\aws\secrets.fav` を新規作成
  ```fav
  // runes/aws/secrets.fav — AWS Secrets Manager wrapper (v14.4.0)

  import rune "ctx"

  public fn secrets_get(ctx: AwsCtx, secret_name: String) -> Result<String, String> !AWS {
      let region = Ctx.aws_get_field_raw(ctx, "region")
      AWS.secrets_get_raw(region, secret_name)
  }
  ```

- [ ] C-2: `target/debug/fav.exe check runes/aws/secrets.fav` でエラーなし確認
  （`fav/` ディレクトリで実行。fav.toml の `[runes] path` が参照するため）

---

## Phase D — `runes/aws/s3.fav`: ctx-aware ラッパー追加

- [ ] D-1: `C:\Users\yoshi\favnir\runes\aws\s3.fav` の先頭に `import rune "ctx"` 追加

- [ ] D-2: `s3.fav` の末尾に ctx-aware ラッパーを追加
  - `public fn s3_put(ctx: AwsCtx, key, body) -> Result<Unit, String> !AWS`
  - `public fn s3_get(ctx: AwsCtx, key) -> Result<String, String> !AWS`
  - `public fn s3_delete(ctx: AwsCtx, key) -> Result<Unit, String> !AWS`
  - `public fn s3_list(ctx: AwsCtx, prefix) -> Result<List<String>, String> !AWS`
  - 各関数の内部: `Ctx.aws_get_field_raw(ctx, "s3_bucket")` でバケット名を取得

  **注意**: 既存の `get_object`, `put_object` 等はそのまま保持（後方互換）

- [ ] D-3: `target/debug/fav.exe check runes/aws/s3.fav` でエラーなし確認

---

## Phase E — `runes/aws/aws.fav` + `rune.toml` 更新

- [ ] E-1: `aws.fav` に `use secrets.*` を追加
  ```fav
  use s3.*
  use sqs.*
  use dynamodb.*
  use secrets.*
  ```

- [ ] E-2: `rune.toml` を更新
  - `version = "14.4.0"`
  - `description` に "Secrets Manager" を追加

- [ ] E-3: 既存の `aws_rune_test_file_passes` が引き続きパスすることを確認
  ```
  cargo test aws_rune_test_file_passes
  ```

---

## Phase F — `fav/src/driver.rs`: v144000_tests + バージョンバンプ

- [ ] F-1: `v144000_tests` モジュールを追加（`v143000_tests` の直後推奨）
  - [ ] `version_is_14_4_0` — `CARGO_PKG_VERSION == "14.4.0"` 確認
  - [ ] `secrets_get_raw_registered` — `AWS.secrets_get_raw` で E0007 が出ない確認
  - [ ] `aws_ctx_field_raw_registered` — `Ctx.aws_get_field_raw` で E0007 が出ない確認
  - [ ] `aws_rune_s3_ctx_functions_present` — `s3.fav` に `s3_put`/`s3_get` が存在する確認

  テスト本文は `plan.md` の Phase F を参照。

- [ ] F-2: `v143000_tests` の `version_is_14_3_0` を `>=` 比較に修正

- [ ] F-3: `fav/Cargo.toml` バージョンを `"14.4.0"` にバンプ

- [ ] F-4: `cargo test v144000` で 4 件全パス確認

---

## Phase G — 全テスト + コミット

- [ ] G-1: `cargo test v144000` 全 4 件パス
- [ ] G-2: `cargo test` 全件パス（リグレッションなし）
- [ ] G-3: `git commit -m "feat: v14.4.0 — AWS Rune 正式パッケージング (Ctx.aws_get_field_raw / AWS.secrets_get_raw)"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Ctx.aws_get_field_raw` が E0007 を出さない | [ ] |
| `AWS.secrets_get_raw` が E0007 を出さない | [ ] |
| `runes/aws/secrets.fav` が `fav check` をパス | [ ] |
| `runes/aws/s3.fav` に `s3_put` / `s3_get` が存在する | [ ] |
| `cargo test aws_rune_test_file_passes` がパス（リグレッションなし） | [ ] |
| `cargo test v144000` 全 4 件パス | [ ] |
| `cargo test` 全件パス | [ ] |
| `CARGO_PKG_VERSION == "14.4.0"` | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.4.0/spec.md` | 仕様・ユーザー体験 |
| `versions/v14.4.0/plan.md` | 実装詳細・コードスニペット |
| `versions/v14.3.0/tasks.md` | 先行バージョンのパターン参照 |
| `versions/roadmap-v14.1-v15.0.md` | v14.4.0 の位置づけ・依存関係 |
