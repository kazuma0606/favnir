# v14.2.0 Plan — 技術実装設計

Date: 2026-06-12

---

## 実装順序（Phase A → E）

```
A: fav/src/toml.rs — AzureTomlConfig 追加
    ↓
B: fav/src/backend/vm.rs — Ctx.build_aws_raw / Ctx.build_azure_raw プリミティブ追加
    ↓
C: fav/src/middle/checker.rs — builtin_ret_ty に追加
    ↓
D: runes/ctx/crosscloud.fav — AwsCtx / AzureCtx 型 + builder
    ↓
E: fav/src/driver.rs — inject_azure_config + Cargo.toml バージョンバンプ
    ↓
F: fav/src/driver.rs — v142000_tests 追加
```

---

## Phase A: `fav/src/toml.rs`

### A-1: `AzureTomlConfig` 構造体追加

既存の `SnowflakeTomlConfig` パターンを踏襲する。

```rust
// ── Azure config (v14.2.0) ────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct AzureTomlConfig {
    pub postgres_url:    Option<String>,
    pub storage_account: Option<String>,
    pub storage_key:     Option<String>,
    pub container:       Option<String>,
}
```

### A-2: `FavToml` に `azure` フィールド追加

`context: Option<ContextConfig>` の直後に追加:
```rust
/// Optional Azure configuration (v14.2.0).
pub azure: Option<AzureTomlConfig>,
```

### A-3: `parse_fav_toml` に `[azure]` パース追加

`parse_fav_toml` 関数内、`snowflake` パースブロックを参考に追加。
環境変数展開は既存の `expand_env_vars` を使う（`${AZURE_POSTGRES_URL}` など）。

```rust
// [azure] section
let azure_cfg = table.get("azure").and_then(|v| v.as_table()).map(|t| {
    AzureTomlConfig {
        postgres_url:    get_str(t, "postgres_url").map(|s| expand_env_vars(&s)),
        storage_account: get_str(t, "storage_account").map(|s| expand_env_vars(&s)),
        storage_key:     get_str(t, "storage_key").map(|s| expand_env_vars(&s)),
        container:       get_str(t, "container").map(|s| expand_env_vars(&s)),
    }
});
```

`FavToml { ... azure: azure_cfg, ... }` でフィールドを設定。

---

## Phase B: `fav/src/backend/vm.rs`

### B-1: `Ctx.build_aws_raw` プリミティブ追加

既存 `"Ctx.build_raw"` ハンドラ（line ~13109）の直後に追加。

```rust
"Ctx.build_aws_raw" => {
    // Ctx.build_aws_raw(region: String, s3_bucket: String, db_url: String) -> Result<AwsCtx, String>
    if args.len() != 3 {
        return Err("Ctx.build_aws_raw requires 3 arguments".to_string());
    }
    let region    = as_string(&args[0], "region")?;
    let s3_bucket = as_string(&args[1], "s3_bucket")?;
    let db_url    = as_string(&args[2], "db_url")?;
    let json = format!(
        r#"{{"region":"{}","s3_bucket":"{}","db_url":"{}"}}"#,
        region, s3_bucket, db_url
    );
    Ok(Value::String(format!("ok({})", json)))
}

"Ctx.build_azure_raw" => {
    // Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container) -> Result<AzureCtx, String>
    if args.len() != 4 {
        return Err("Ctx.build_azure_raw requires 4 arguments".to_string());
    }
    let postgres_url    = as_string(&args[0], "postgres_url")?;
    let storage_account = as_string(&args[1], "storage_account")?;
    let storage_key     = as_string(&args[2], "storage_key")?;
    let container       = as_string(&args[3], "container")?;
    let json = format!(
        r#"{{"postgres_url":"{}","storage_account":"{}","storage_key":"{}","container":"{}"}}"#,
        postgres_url, storage_account, storage_key, container
    );
    Ok(Value::String(format!("ok({})", json)))
}

"Ctx.azure_get_field_raw" => {
    // Ctx.azure_get_field_raw(ctx: AzureCtx, field: String) -> String
    if args.len() != 2 {
        return Err("Ctx.azure_get_field_raw requires 2 arguments".to_string());
    }
    let ctx_str = as_string(&args[0], "ctx")?;
    let field   = as_string(&args[1], "field")?;
    // ctx_str is either raw JSON or "ok({...})"
    let json_str = ctx_str.trim_start_matches("ok(").trim_end_matches(')');
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let val = parsed.get(&field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Value::String(val))
}
```

### B-2: `"Ctx"` namespace の match arm に追加

既存の `"Ctx"` ブランチ（line ~5963）の `call_external_builtin` に
`"Ctx.build_aws_raw"` / `"Ctx.build_azure_raw"` / `"Ctx.azure_get_field_raw"` を追加。

---

## Phase C: `fav/src/middle/checker.rs`

### C-1: `builtin_ret_ty` に追加

```rust
("Ctx", "build_aws_raw")        => "Result<AwsCtx, String>",
("Ctx", "build_azure_raw")      => "Result<AzureCtx, String>",
("Ctx", "azure_get_field_raw")  => "String",
```

※ `AwsCtx` / `AzureCtx` は Favnir の型チェッカーでは `Unknown` として扱われる（名目型ラッパーのため）。
  `builtin_ret_ty` の戻り値文字列はチェッカーが `Unknown` にフォールバックするので問題なし。

### C-2: NS env def に `"Ctx"` ブロックの拡張

既存の `"Ctx"` ブロックに `"build_aws_raw"` / `"build_azure_raw"` / `"azure_get_field_raw"` を追加。

---

## Phase D: `runes/ctx/crosscloud.fav`（新規作成）

```fav
// runes/ctx/crosscloud.fav — AwsCtx / AzureCtx builder (v14.2.0)

type AwsCtx(String)
type AzureCtx(String)

// CrossCloudCtx は呼び出し側で構成する（統合型は v15.0.0 で検討）:
// type CrossCloudCtx = { aws: AwsCtx azure: AzureCtx }

public fn build_aws(
    region: String,
    s3_bucket: String,
    db_url: String
) -> Result<AwsCtx, String> {
    Ctx.build_aws_raw(region, s3_bucket, db_url)
}

public fn build_azure(
    postgres_url: String,
    storage_account: String,
    storage_key: String,
    container: String
) -> Result<AzureCtx, String> {
    Ctx.build_azure_raw(postgres_url, storage_account, storage_key, container)
}

public fn azure_postgres_url(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "postgres_url")
}

public fn azure_storage_account(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "storage_account")
}

public fn azure_storage_key(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "storage_key")
}

public fn azure_container(ctx: AzureCtx) -> String {
    Ctx.azure_get_field_raw(ctx, "container")
}
```

---

## Phase E: `fav/src/driver.rs`

### E-1: `inject_azure_config` 関数追加

`inject_snowflake_config` の直後に追加:

```rust
fn inject_azure_config(cfg: &crate::toml::AzureTomlConfig) {
    if let Some(url) = &cfg.postgres_url {
        std::env::set_var("AZURE_POSTGRES_URL", url);
    }
    if let Some(account) = &cfg.storage_account {
        std::env::set_var("AZURE_STORAGE_ACCOUNT", account);
    }
    if let Some(key) = &cfg.storage_key {
        std::env::set_var("AZURE_STORAGE_KEY", key);
    }
    if let Some(container) = &cfg.container {
        std::env::set_var("AZURE_CONTAINER", container);
    }
}
```

### E-2: `load_run_config` から呼び出す

既存の `inject_snowflake_config` 呼び出しブロックと同じパターンで:

```rust
if let Some(azure_cfg) = toml.azure.as_ref() {
    inject_azure_config(azure_cfg);
}
```

`load_run_config` / `load_check_config` の両方に追加（snowflake と同じ場所）。

### E-3: `Cargo.toml` バージョンバンプ

`fav/Cargo.toml`:
```toml
version = "14.2.0"
```

---

## Phase F: `fav/src/driver.rs` — `v142000_tests`

```rust
#[cfg(test)]
mod v142000_tests {
    use super::tests::*;

    #[test]
    fn version_is_14_2_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "14.2.0");
    }

    #[test]
    fn fav_toml_azure_section_parsed() {
        // fav.toml の [azure] セクションをパースできることを確認
        let toml_src = r#"
name = "test"
version = "0.1.0"
[azure]
postgres_url = "postgresql://user:pass@host/db"
storage_account = "mystorageaccount"
storage_key = "mykey=="
container = "favnir-proof"
"#;
        let toml = crate::toml::parse_fav_toml(toml_src);
        let azure = toml.azure.expect("azure section");
        assert_eq!(azure.postgres_url.as_deref(), Some("postgresql://user:pass@host/db"));
        assert_eq!(azure.container.as_deref(), Some("favnir-proof"));
    }

    #[test]
    fn aws_ctx_build_raw_registered() {
        // Ctx.build_aws_raw が E0007 を出さないことを確認
        let src = r#"
public fn main(ctx: AppCtx) -> Unit {
    bind result <- Ctx.build_aws_raw("ap-northeast-1", "my-bucket", "postgresql://localhost/db")
    ctx.io.println("ok")
}
"#;
        let diags = check_source_raw(src);
        let has_e0007 = diags.iter().any(|d| d.contains("E0007"));
        assert!(!has_e0007, "Ctx.build_aws_raw should not produce E0007, got: {:?}", diags);
    }

    #[test]
    fn azure_ctx_build_raw_registered() {
        // Ctx.build_azure_raw が E0007 を出さないことを確認
        let src = r#"
public fn main(ctx: AppCtx) -> Unit {
    bind result <- Ctx.build_azure_raw(
        "postgresql://user:pass@host/db",
        "mystorageaccount",
        "mykey==",
        "favnir-proof"
    )
    ctx.io.println("ok")
}
"#;
        let diags = check_source_raw(src);
        let has_e0007 = diags.iter().any(|d| d.contains("E0007"));
        assert!(!has_e0007, "Ctx.build_azure_raw should not produce E0007, got: {:?}", diags);
    }
}
```

---

## 参照先ファイル（実装時に確認すること）

| ファイル | 参照目的 |
|---|---|
| `fav/src/toml.rs:96-103` | `SnowflakeTomlConfig` — 構造体パターン |
| `fav/src/toml.rs:174-213` | `FavToml` — フィールド追加箇所 |
| `fav/src/toml.rs:parse_fav_toml` | `[snowflake]` パースブロックをコピーして改変 |
| `fav/src/backend/vm.rs:13109` | `Ctx.build_raw` — プリミティブパターン |
| `fav/src/middle/checker.rs` | `("Ctx", ...)` ブロック — builtin_ret_ty |
| `fav/src/driver.rs:327` | `inject_snowflake_config` — inject パターン |
| `fav/runes/ctx/ctx.fav` | 既存 Ctx rune の形式 |

---

## 実装上の注意点

1. **`Ctx.build_aws_raw` の返り値**: `"ok({json})"` 文字列。これが `AwsCtx` として使われる。
   `Ctx.azure_get_field_raw` は受け取った文字列を JSON パースしてフィールドを取り出す。

2. **型チェッカーの `AwsCtx` / `AzureCtx`**: checker.fav は名目型ラッパーを `Unknown` にフォールバックする。
   `builtin_ret_ty("Ctx", "build_aws_raw")` の戻り値が `"Result<AwsCtx, String>"` でも
   `infer_hm` の型推論では `Unknown` として処理される（既存動作）。E0007 が出なければ OK。

3. **`parse_fav_toml` のテスト**: `parse_fav_toml` が `pub(crate)` でない場合は `pub` に変更するか
   `tests` モジュールの `super::` パス経由でアクセスする（既存パターンを確認）。

4. **Windows 環境での `set_var`**: `inject_azure_config` で `std::env::set_var` を使う場合、
   並行テストでの競合に注意。既存の `inject_snowflake_config` と同じパターンで問題なし。

5. **`runes/ctx/` ディレクトリ**: `crosscloud.fav` を追加する際、`rune.toml` の更新は不要
   （ディレクトリ rune は内部ファイルを自動的に結合する）。
