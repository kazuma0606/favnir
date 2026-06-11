# v14.2.0 Tasks — AzureCtx / AwsCtx + fav.toml [azure]

Date: 2026-06-12
Branch: master（または feat/v14-crosscloud-ctx）

---

## Phase A — `fav/src/toml.rs`: AzureTomlConfig 追加

- [ ] A-1: `AzureTomlConfig` 構造体を追加（`SnowflakeTomlConfig` の直後、toml.rs:96 付近）
  ```rust
  #[derive(Debug, Clone)]
  pub struct AzureTomlConfig {
      pub postgres_url:    Option<String>,
      pub storage_account: Option<String>,
      pub storage_key:     Option<String>,
      pub container:       Option<String>,
  }
  ```

- [ ] A-2: `FavToml` 構造体に `azure: Option<AzureTomlConfig>` フィールド追加
  （`context: Option<ContextConfig>` の直後に追加）

- [ ] A-3: `parse_fav_toml` に `[azure]` パースブロック追加
  - `expand_env_vars` で `${VAR}` 展開
  - `FavToml { ... azure: azure_cfg, ... }` でフィールドを設定

- [ ] A-4: `cargo build` でコンパイルエラーなし確認

---

## Phase B — `fav/src/backend/vm.rs`: VM プリミティブ追加

- [ ] B-1: `"Ctx.build_aws_raw"` ハンドラ追加（`"Ctx.build_raw"` の直後）
  - 引数 3 個: `region: String`, `s3_bucket: String`, `db_url: String`
  - 返り値: `Value::String("ok({...json...})")`

- [ ] B-2: `"Ctx.build_azure_raw"` ハンドラ追加
  - 引数 4 個: `postgres_url`, `storage_account`, `storage_key`, `container`
  - 返り値: `Value::String("ok({...json...})")`

- [ ] B-3: `"Ctx.azure_get_field_raw"` ハンドラ追加
  - 引数 2 個: `ctx: AzureCtx（文字列）`, `field: String`
  - JSON パースして指定フィールドの値を返す
  - 返り値: `Value::String(field_value)`

- [ ] B-4: `cargo build` でコンパイルエラーなし確認

---

## Phase C — `fav/src/middle/checker.rs`: builtin_ret_ty 追加

- [ ] C-1: `builtin_ret_ty` の `("Ctx", ...)` ブロックに追加
  ```rust
  ("Ctx", "build_aws_raw")       => "Result<AwsCtx, String>",
  ("Ctx", "build_azure_raw")     => "Result<AzureCtx, String>",
  ("Ctx", "azure_get_field_raw") => "String",
  ```

- [ ] C-2: `ns_env_def` の `"Ctx"` ブロックに 3 関数を追加
  （E0007 防止のため namespace に登録）

- [ ] C-3: `cargo build` でコンパイルエラーなし確認

---

## Phase D — `runes/ctx/crosscloud.fav`: 型と builder 追加

- [ ] D-1: `fav/runes/ctx/crosscloud.fav` を新規作成
  ```fav
  type AwsCtx(String)
  type AzureCtx(String)

  public fn build_aws(region, s3_bucket, db_url) -> Result<AwsCtx, String>
  public fn build_azure(postgres_url, storage_account, storage_key, container) -> Result<AzureCtx, String>
  public fn azure_postgres_url(ctx: AzureCtx) -> String
  public fn azure_storage_account(ctx: AzureCtx) -> String
  public fn azure_storage_key(ctx: AzureCtx) -> String
  public fn azure_container(ctx: AzureCtx) -> String
  ```
  詳細は `plan.md` の Phase D を参照。

- [ ] D-2: `target/debug/fav.exe check fav/runes/ctx/crosscloud.fav` でエラーなし確認

---

## Phase E — `fav/src/driver.rs`: inject_azure_config + バージョンバンプ

- [ ] E-1: `inject_azure_config(cfg: &AzureTomlConfig)` 関数追加
  - `inject_snowflake_config`（driver.rs:327）の直後に追加
  - `AZURE_POSTGRES_URL` / `AZURE_STORAGE_ACCOUNT` / `AZURE_STORAGE_KEY` / `AZURE_CONTAINER` を `std::env::set_var` で設定

- [ ] E-2: `load_run_config` / `load_check_config` から `inject_azure_config` を呼び出す
  - `inject_snowflake_config` 呼び出しと同じブロック内に追加

- [ ] E-3: `fav/Cargo.toml` バージョンを `"14.2.0"` にバンプ

- [ ] E-4: `cargo build` でコンパイルエラーなし確認

---

## Phase F — `fav/src/driver.rs`: v142000_tests 追加

- [ ] F-1: `v142000_tests` モジュールを追加（v141000_tests の直後推奨）
  - [ ] `version_is_14_2_0` — `CARGO_PKG_VERSION == "14.2.0"` 確認
  - [ ] `fav_toml_azure_section_parsed` — `parse_fav_toml` で `[azure]` をパースできる確認
  - [ ] `aws_ctx_build_raw_registered` — `Ctx.build_aws_raw` で E0007 が出ない確認
  - [ ] `azure_ctx_build_raw_registered` — `Ctx.build_azure_raw` で E0007 が出ない確認

  テスト本文は `plan.md` の Phase F を参照。

- [ ] F-2: `cargo test v142000` で 4 件全パス確認

---

## Phase G — 全テスト + コミット

- [ ] G-1: `cargo test v142000` 全 4 件パス
- [ ] G-2: `cargo test` 全件パス（リグレッションなし）
- [ ] G-3: `git commit -m "feat: v14.2.0 — AzureCtx / AwsCtx + fav.toml [azure]"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `fav.toml` に `[azure]` を書いてエラーなし | [ ] |
| `Ctx.build_aws_raw` が E0007 を出さない | [ ] |
| `Ctx.build_azure_raw` が E0007 を出さない | [ ] |
| `runes/ctx/crosscloud.fav` が `fav check` をパス | [ ] |
| `cargo test v142000` 全 4 件パス | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `CARGO_PKG_VERSION == "14.2.0"` | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.2.0/spec.md` | 仕様・ユーザー体験 |
| `versions/v14.2.0/plan.md` | 実装詳細・コードスニペット |
| `versions/v14.1.0/tasks.md` | 先行バージョンのパターン参照 |
| `versions/roadmap-v14.1-v15.0.md` | v14.2.0 の位置づけ・依存関係 |
