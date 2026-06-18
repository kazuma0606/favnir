# Favnir v10.7.0 Spec

Date: 2026-06-04
Theme: fav.toml Snowflake 設定対応

---

## 概要

`fav.toml` に `[snowflake]` セクションを追加し、Snowflake 接続設定を
プロジェクト設定ファイルで管理できるようにする。
`${ENV_VAR}` 形式の環境変数参照もサポートする。

---

## 実装方針

### 変更点

1. **`toml.rs`**: `SnowflakeTomlConfig` 構造体 + `[snowflake]` セクション解析 + `expand_env_vars` ヘルパー
2. **`driver.rs`**: `inject_snowflake_config` — toml の設定を env var として注入（未設定時のみ）
3. **`driver.rs`**: `run_with_favnir_pipeline_project` / `cmd_run` legacy path で `inject_snowflake_config` を呼ぶ
4. **`driver.rs`**: `default_fav_toml` にコメントアウトされた `[snowflake]` 例を追加
5. テスト: `toml_snowflake_section_parsed` / `toml_snowflake_env_var_expanded` / `toml_snowflake_inject_sets_env_vars`

### 設計原則

- env var が既に設定されている場合は上書きしない（dotenv と同じ動作）
- `private_key` / `public_key_fp` はセキュリティ上 fav.toml に書かせない（env var 専用）
- `account` / `user` / `warehouse` / `role` / `database` / `schema` のみ fav.toml で管理

---

## `SnowflakeTomlConfig` 構造体

```rust
#[derive(Debug, Clone)]
pub struct SnowflakeTomlConfig {
    pub account:   Option<String>,
    pub user:      Option<String>,
    pub warehouse: Option<String>,
    pub role:      Option<String>,
    pub database:  Option<String>,
    pub schema:    Option<String>,
}
```

`FavToml` に `pub snowflake: Option<SnowflakeTomlConfig>` を追加。

---

## `expand_env_vars` ヘルパー

`${VAR_NAME}` を `std::env::var("VAR_NAME")` で展開する。
変数が未設定の場合は空文字列に置換。

```rust
pub fn expand_env_vars(s: &str) -> String {
    // "${VAR}" パターンを正規表現なしで手動展開
    let mut result = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find('}') {
            let var_name = &after[..end];
            result.push_str(&std::env::var(var_name).unwrap_or_default());
            rest = &after[end + 1..];
        } else {
            result.push_str("${");
            rest = after;
        }
    }
    result.push_str(rest);
    result
}
```

---

## `inject_snowflake_config`（driver.rs）

fav.toml の Snowflake 設定を env var として注入する。
env var が既に設定されている場合はスキップ。

```rust
fn inject_snowflake_config(cfg: &crate::toml::SnowflakeTomlConfig) {
    use crate::toml::expand_env_vars;
    let pairs = [
        ("SNOWFLAKE_ACCOUNT",   cfg.account.as_deref()),
        ("SNOWFLAKE_USER",      cfg.user.as_deref()),
        ("SNOWFLAKE_WAREHOUSE", cfg.warehouse.as_deref()),
        ("SNOWFLAKE_ROLE",      cfg.role.as_deref()),
        ("SNOWFLAKE_DATABASE",  cfg.database.as_deref()),
        ("SNOWFLAKE_SCHEMA",    cfg.schema.as_deref()),
    ];
    for (key, val) in pairs {
        if let Some(v) = val {
            if std::env::var(key).is_err() {
                unsafe { std::env::set_var(key, expand_env_vars(v)); }
            }
        }
    }
}
```

---

## `default_fav_toml` 更新（fav new テンプレート）

```toml
[project]
name    = "{name}"
version = "0.1.0"
edition = "2026"
src     = "src"

# [snowflake]
# account   = "${SNOWFLAKE_ACCOUNT}"
# user      = "${SNOWFLAKE_USER}"
# warehouse = "COMPUTE_WH"
# database  = "MY_DB"
# schema    = "PUBLIC"
```

---

## テスト設計

### `toml_snowflake_section_parsed`（toml.rs tests）

```rust
let t = parse("[rune]\nname = \"app\"\nversion = \"1.0.0\"\n[snowflake]\naccount = \"myaccount\"\nuser = \"myuser\"\nwarehouse = \"WH\"\n");
let sf = t.snowflake.unwrap();
assert_eq!(sf.account.as_deref(), Some("myaccount"));
assert_eq!(sf.warehouse.as_deref(), Some("WH"));
```

### `toml_snowflake_env_var_expanded`（toml.rs tests）

```rust
unsafe { std::env::set_var("TEST_SF_ACCOUNT", "testaccount"); }
let expanded = expand_env_vars("${TEST_SF_ACCOUNT}.snowflakecomputing.com");
assert_eq!(expanded, "testaccount.snowflakecomputing.com");
```

### `toml_snowflake_inject_sets_env_vars`（driver.rs tests）

```rust
unsafe { std::env::remove_var("SNOWFLAKE_WAREHOUSE"); }
let cfg = SnowflakeTomlConfig { warehouse: Some("TEST_WH".to_string()), .. };
inject_snowflake_config(&cfg);
assert_eq!(std::env::var("SNOWFLAKE_WAREHOUSE").ok(), Some("TEST_WH".to_string()));
```

---

## バージョン更新

- `fav/Cargo.toml`: `version = "10.7.0"`
- `fav/self/cli.fav`: `run_version` → `"10.7.0"`
