# Favnir v10.7.0 Plan

Date: 2026-06-04
Theme: fav.toml Snowflake 設定対応

---

## Phase A: toml.rs 更新

### A-1: `SnowflakeTomlConfig` 構造体を追加（`AwsTomlConfig` の直後）

```rust
// ── Snowflake config (v10.7.0) ────────────────────────────────────────────────

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

### A-2: `FavToml` に `snowflake` フィールドを追加

`deploy: Option<DeployConfig>` の後に追加:
```rust
/// Optional Snowflake configuration (v10.7.0).
pub snowflake: Option<SnowflakeTomlConfig>,
```

### A-3: `parse_fav_toml` に `[snowflake]` セクション解析を追加

変数宣言部:
```rust
let mut snowflake_cfg: Option<SnowflakeTomlConfig> = None;
```

セクション検出（`if trimmed == "[deploy]"` の後):
```rust
if trimmed == "[snowflake]" {
    section = "snowflake";
    continue;
}
```

セクション処理（`"deploy" =>` の後):
```rust
"snowflake" => {
    let mut current = snowflake_cfg.take().unwrap_or(SnowflakeTomlConfig {
        account:   None,
        user:      None,
        warehouse: None,
        role:      None,
        database:  None,
        schema:    None,
    });
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "account"   => current.account   = Some(val.to_string()),
            "user"      => current.user      = Some(val.to_string()),
            "warehouse" => current.warehouse = Some(val.to_string()),
            "role"      => current.role      = Some(val.to_string()),
            "database"  => current.database  = Some(val.to_string()),
            "schema"    => current.schema    = Some(val.to_string()),
            _ => {}
        }
    }
    snowflake_cfg = Some(current);
}
```

`FavToml { ... }` 末尾に追加:
```rust
snowflake: snowflake_cfg,
```

### A-4: `expand_env_vars` 公開関数を追加（`parse_kv` の後）

```rust
/// Expand `${VAR_NAME}` references in a string using environment variables.
/// Unset variables are replaced with an empty string.
pub fn expand_env_vars(s: &str) -> String {
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

## Phase B: driver.rs 更新

### B-1: `inject_snowflake_config` 関数を追加

`default_fav_toml` 関数の前あたりに追加:

```rust
fn inject_snowflake_config(cfg: &crate::toml::SnowflakeTomlConfig) {
    use crate::toml::expand_env_vars;
    let pairs: &[(&str, Option<&str>)] = &[
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
                // SAFETY: called before VM starts; single-threaded at this point
                unsafe { std::env::set_var(key, expand_env_vars(v)); }
            }
        }
    }
}
```

### B-2: `run_with_favnir_pipeline_project` で `inject_snowflake_config` を呼ぶ

`toml.snowflake` が `Some` の場合に注入:

```rust
if let Some(sf_cfg) = &toml.snowflake {
    inject_snowflake_config(sf_cfg);
}
```

### B-3: `cmd_run` legacy path（Rust pipeline, project mode）でも同様に注入

`FavToml::load` でロードした後、実行前に `inject_snowflake_config` を呼ぶ。

### B-4: `default_fav_toml` にコメントアウト `[snowflake]` 例を追加

```rust
fn default_fav_toml(name: &str) -> String {
    format!(
        "[project]\nname    = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\nsrc     = \"src\"\n\n\
         # [snowflake]\n\
         # account   = \"${{SNOWFLAKE_ACCOUNT}}\"\n\
         # user      = \"${{SNOWFLAKE_USER}}\"\n\
         # warehouse = \"COMPUTE_WH\"\n\
         # database  = \"MY_DB\"\n\
         # schema    = \"PUBLIC\"\n"
    )
}
```

---

## Phase C: テスト追加

### C-1: `toml.rs` tests — `toml_snowflake_section_parsed`

```rust
#[test]
fn toml_snowflake_section_parsed() {
    let t = parse(
        "[rune]\nname = \"app\"\nversion = \"1.0.0\"\n\
         [snowflake]\naccount = \"myaccount\"\nuser = \"myuser\"\nwarehouse = \"WH\"\ndatabase = \"DB\"\n",
    );
    let sf = t.snowflake.expect("snowflake config");
    assert_eq!(sf.account.as_deref(),   Some("myaccount"));
    assert_eq!(sf.user.as_deref(),      Some("myuser"));
    assert_eq!(sf.warehouse.as_deref(), Some("WH"));
    assert_eq!(sf.database.as_deref(),  Some("DB"));
}
```

### C-2: `toml.rs` tests — `toml_snowflake_env_var_expanded`

```rust
#[test]
fn toml_snowflake_env_var_expanded() {
    unsafe { std::env::set_var("TEST_SF_ACCT_10700", "myaccount"); }
    let expanded = expand_env_vars("${TEST_SF_ACCT_10700}.snowflakecomputing.com");
    assert_eq!(expanded, "myaccount.snowflakecomputing.com");
    unsafe { std::env::remove_var("TEST_SF_ACCT_10700"); }
}
```

### C-3: `driver.rs` — `v10700_tests::toml_snowflake_inject_sets_env_vars`

```rust
#[cfg(test)]
mod v10700_tests {
    #[test]
    fn toml_snowflake_inject_sets_env_vars() {
        unsafe { std::env::remove_var("SNOWFLAKE_WAREHOUSE"); }
        let cfg = crate::toml::SnowflakeTomlConfig {
            account:   None,
            user:      None,
            warehouse: Some("TEST_WH_10700".to_string()),
            role:      None,
            database:  None,
            schema:    None,
        };
        super::inject_snowflake_config(&cfg);
        assert_eq!(
            std::env::var("SNOWFLAKE_WAREHOUSE").ok(),
            Some("TEST_WH_10700".to_string()),
        );
        // cleanup
        unsafe { std::env::remove_var("SNOWFLAKE_WAREHOUSE"); }
    }

    #[test]
    fn toml_snowflake_inject_does_not_overwrite_existing_env() {
        unsafe { std::env::set_var("SNOWFLAKE_ROLE", "EXISTING_ROLE"); }
        let cfg = crate::toml::SnowflakeTomlConfig {
            account:   None,
            user:      None,
            warehouse: None,
            role:      Some("NEW_ROLE".to_string()),
            database:  None,
            schema:    None,
        };
        super::inject_snowflake_config(&cfg);
        assert_eq!(
            std::env::var("SNOWFLAKE_ROLE").ok(),
            Some("EXISTING_ROLE".to_string()),
            "should not overwrite existing SNOWFLAKE_ROLE",
        );
        unsafe { std::env::remove_var("SNOWFLAKE_ROLE"); }
    }
}
```

---

## Phase D: バージョン更新

### D-1: `fav/Cargo.toml` version → `"10.7.0"`

### D-2: `fav/self/cli.fav` の `run_version` → `"10.7.0"`

---

## Phase E: self-check + cargo test

```bash
# E-1: compiler.fav self-check
fav check --legacy-check self/compiler.fav

# E-2: 新規テスト確認
cargo test v10700
cargo test toml_snowflake

# E-3: bootstrap 維持確認
cargo test bootstrap

# E-4: 全件確認（目標: 1277 件 = 1272 + 5 新規）
cargo test
```
