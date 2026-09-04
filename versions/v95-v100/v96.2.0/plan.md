# Plan: v96.2.0 — `fav.toml [sap.environments]` マルチ環境設定

## 実装ステップ

### Step 1: `fav/src/toml.rs` — `SapEnvEntry` 構造体と `SapEnvironmentsConfig` 型を追加

既存 `SapTomlConfig` 定義（行 433〜442 付近）の直前に追加する。

```rust
// ── SAP environments config (v96.2.0) ────────────────────────────────────────

/// `[sap.environments.<NAME>]` 単一環境エントリ（v96.2.0）
#[derive(Debug, Clone, Default)]
pub struct SapEnvEntry {
    pub base_url: Option<String>,
    pub client:   Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// `[sap.environments]` セクション全体（v96.2.0）
/// key: 環境名（"PRD" / "QAS" / "DEV" 等）、value: 接続設定
pub type SapEnvironmentsConfig = std::collections::HashMap<String, SapEnvEntry>;
```

### Step 2: `fav/src/toml.rs` — `SapTomlConfig` に `environments` フィールドを追加

```rust
#[derive(Debug, Clone, Default)]
pub struct SapTomlConfig {
    pub base_url:     Option<String>,
    pub client:       Option<String>,
    pub username:     Option<String>,
    pub password:     Option<String>,
    pub auth:         Option<String>,
    pub environments: SapEnvironmentsConfig,  // v96.2.0 追加
}
```

### Step 3: `fav/src/toml.rs` — `[sap.environments.<NAME>]` セクションのパース処理を追加

パーサーループ内の `[sap]` セクション検出（行 785 付近）の直前に追加する。
セクションヘッダーのマッチ条件:

```rust
if trimmed.starts_with("[sap.environments.") && trimmed.ends_with(']') {
    let env_name = &trimmed["[sap.environments.".len()..trimmed.len() - 1];
    section = "sap_env";
    current_sap_env = env_name.to_string();
    continue;
}
```

`"sap_env"` セクション内の KV パース:

```rust
"sap_env" => {
    let entry = sap_cfg
        .get_or_insert_with(SapTomlConfig::default)
        .environments
        .entry(current_sap_env.clone())
        .or_default();
    if let Some((key, val)) = parse_kv(trimmed) {
        match key {
            "base_url" => entry.base_url = Some(val.to_string()),
            "client"   => entry.client   = Some(val.to_string()),
            "username" => entry.username = Some(val.to_string()),
            "password" => entry.password = Some(val.to_string()),
            _ => {}
        }
    }
}
```

必要な変数宣言（パーサー関数内のローカル変数として追加）:

```rust
let mut current_sap_env = String::new();
```

### Step 4: `fav/src/driver.rs` — `mod v96200_tests` を追加

`mod v96100_tests` の直後に追加する。

```rust
#[cfg(test)]
mod v96200_tests {
    #[test]
    fn sap_env_entry_struct_defined() {
        let content = include_str!("toml.rs");
        assert!(
            content.contains("SapEnvEntry"),
            "toml.rs should define SapEnvEntry struct"
        );
    }

    #[test]
    fn sap_toml_config_has_environments_field() {
        let content = include_str!("toml.rs");
        assert!(
            content.contains("SapEnvironmentsConfig"),
            "toml.rs should define SapEnvironmentsConfig type"
        );
    }
}
```

## 依存関係

```
Step 1 (SapEnvEntry / SapEnvironmentsConfig 型) →
Step 2 (SapTomlConfig.environments フィールド) →
Step 3 (パース処理) →
Step 4 (テスト)
```
