# Plan: v85.1.0 — `SapTomlConfig` + `inject_sap_config()`

## Step 1: 前提確認

- `cargo test` を実行し、3,931 tests, 0 failures を確認する
- `fav/Cargo.toml` の `version = "85.0.0"` であることを確認する
- `fav/src/driver.rs` に `mod v85000_tests` が存在することを確認する（v85.0.0 完了済みの証拠）
- `fav/src/toml.rs` に `inject_snowflake_config` などの既存 inject 関数パターンを確認する

## Step 2: `fav/src/toml.rs` に `SapTomlConfig` を追加

`SnowflakeTomlConfig` / `PostgresTomlConfig` の定義の近くに以下を追加する。

```rust
// ── v85.1.0: SapTomlConfig ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct SapTomlConfig {
    pub base_url: Option<String>,
    pub client:   Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth:     Option<String>,
}
```

## Step 3: `FavTomlProject` に `sap` フィールドを追加

`FavTomlProject` 構造体の `snowflake: Option<SnowflakeTomlConfig>` の近くに追加する。

```rust
pub sap: Option<SapTomlConfig>,
```

## Step 4: `fav/src/driver.rs` に `inject_sap_config()` を追加

`inject_snowflake_config` の直後に追加する。

```rust
// ── v85.1.0: inject_sap_config ──────────────────────────────────────────────

pub fn inject_sap_config(cfg: &fav_core::toml::SapTomlConfig) {
    if let Some(v) = &cfg.base_url {
        std::env::set_var("SAP_BASE_URL", expand_env_vars(v));
    }
    if let Some(v) = &cfg.client {
        std::env::set_var("SAP_CLIENT", expand_env_vars(v));
    }
    if let Some(v) = &cfg.username {
        std::env::set_var("SAP_USER", expand_env_vars(v));
    }
    if let Some(v) = &cfg.password {
        std::env::set_var("SAP_PASS", expand_env_vars(v));
    }
    if let Some(v) = &cfg.auth {
        std::env::set_var("SAP_AUTH", expand_env_vars(v));
    }
}
```

## Step 5: `cmd_run` / `cmd_check` で `inject_sap_config` を呼ぶ

既存の `inject_snowflake_config` 呼び出し箇所のパターンに合わせて追加する。
`unwrap_or_default()` 形式であれば同形式、`if let` 形式であれば同形式を採用すること。

`cmd_run` と `cmd_check` の **両方** に追加すること（片方のみでは不完全）。

```rust
// inject_snowflake_config 呼び出しの直後に追加（パターンは既存に合わせる）
inject_sap_config(&project.sap.unwrap_or_default());
```

## Step 6: `fav/src/driver.rs` に `mod v85100_tests` を追加

`mod v85000_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85100_tests {
    use fav_core::toml::SapTomlConfig;

    #[test]
    fn sap_toml_config_parses_base_url() {
        let toml_str = "[sap]\nbase_url = \"https://example.com\"\n";
        #[derive(serde::Deserialize)]
        struct Wrapper { sap: Option<SapTomlConfig> }
        let parsed: Wrapper = toml::from_str(toml_str).unwrap();
        let cfg = parsed.sap.unwrap();
        assert_eq!(cfg.base_url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn inject_sap_config_sets_env_vars() {
        let cfg = SapTomlConfig {
            base_url: Some("https://test.example.com".to_string()),
            client:   Some("200".to_string()),
            username: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            auth:     Some("basic".to_string()),
        };
        super::inject_sap_config(&cfg);
        assert_eq!(std::env::var("SAP_BASE_URL").unwrap(), "https://test.example.com");
        assert_eq!(std::env::var("SAP_CLIENT").unwrap(), "200");
    }
}
```

## Step 7: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3933 tests, 0 failures
```

## Step 8: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.1.0 エントリを追加する。

## Step 9: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
