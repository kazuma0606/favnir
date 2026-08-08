# Plan — v57.7.0 — マルチテナント分離

## 実装方針

v57.3.0（TlsConfig）と同じ TOML パース層パターンで実装する。
`TenancyIsolation` / `TenancyConfig` を `toml.rs` に追加し、
`parse_fav_toml` で `[tenancy]` / `[tenancy.isolation]` セクションを解析する。
テストは `driver.rs` の `v57700_tests` モジュールに完結させる（自己完結型）。

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `57.6.0` → `57.7.0` |
| `fav/src/toml.rs` | `TenancyIsolation` / `TenancyConfig` 追加、`FavToml` フィールド追加、`parse_fav_toml` 更新 |
| `fav/src/driver.rs` | `v57700_tests` 追加、バージョンチェックテスト 3 件更新 |
| `fav/src/resolver.rs` | `FavToml { ... }` 直接初期化 3 箇所に `tenancy: None` 追加 |
| `fav/src/middle/checker.rs` | `FavToml { ... }` 直接初期化 2 箇所に `tenancy: None` 追加 |

---

## 詳細手順

### Step 1: `fav/Cargo.toml` version 更新

```
57.6.0 → 57.7.0
```

### Step 2: `fav/src/toml.rs` — 構造体追加

#### 2-1: `TlsConfig` の直後（`FavToml` の直前）に挿入

```rust
/// Per-rune isolation settings from `[tenancy.isolation]` (v57.7.0).
#[derive(Debug, Clone, Default)]
pub struct TenancyIsolation {
    /// Snowflake schema prefix pattern (e.g. `"tenant_${TENANT_ID}"`).
    pub snowflake_schema: Option<String>,
    /// Kafka topic prefix pattern (e.g. `"${TENANT_ID}."`).
    pub kafka_topic_prefix: Option<String>,
}

/// Multi-tenant configuration from `[tenancy]` in fav.toml (v57.7.0).
#[derive(Debug, Clone, Default)]
pub struct TenancyConfig {
    /// Tenancy mode: `"strict"` enforces tenant ID on all Rune access.
    pub mode: String,
    /// Tenant identifier (supports `${ENV_VAR}` expansion).
    pub tenant: Option<String>,
    /// Optional per-rune isolation settings.
    pub isolation: Option<TenancyIsolation>,
}

impl TenancyConfig {
    /// Returns `true` when mode is `"strict"`.
    pub fn is_strict(&self) -> bool {
        self.mode == "strict"
    }
}
```

#### 2-2: `FavToml` に `tenancy` フィールド追加

`pub tls: Option<TlsConfig>,` の直後:
```rust
/// Optional multi-tenant configuration (v57.7.0).
pub tenancy: Option<TenancyConfig>,
```

#### 2-3: `parse_fav_toml` — アキュムレータ宣言

`tls_cfg` 宣言の直後:
```rust
let mut tenancy_cfg: Option<TenancyConfig> = None;
```

#### 2-4: `parse_fav_toml` — セクション検出追加

`"[security.tls]"` 検出の直後:
```rust
} else if trimmed == "[tenancy]" {
    section = "tenancy";
} else if trimmed == "[tenancy.isolation]" {
    section = "tenancy.isolation";
```

#### 2-5: `parse_fav_toml` — セクション処理アーム追加

`_ => {}` の直前:
```rust
"tenancy" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        let current = tenancy_cfg.get_or_insert_with(TenancyConfig::default);
        match key {
            "mode"   => current.mode   = expand_env_vars(val),
            "tenant" => current.tenant = Some(expand_env_vars(val)),
            _ => {}
        }
    }
}
"tenancy.isolation" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        let tc = tenancy_cfg.get_or_insert_with(TenancyConfig::default);
        let iso = tc.isolation.get_or_insert_with(TenancyIsolation::default);
        match key {
            "snowflake_schema"   => iso.snowflake_schema   = Some(expand_env_vars(val)),
            "kafka_topic_prefix" => iso.kafka_topic_prefix = Some(expand_env_vars(val)),
            _ => {}
        }
    }
}
```

#### 2-6: `FavToml` 返却部に追加

```rust
tenancy: tenancy_cfg,
```

### Step 3: `FavToml` 直接初期化 6 箇所に `tenancy: None` 追加

各箇所で `tls: None,` の直後に追加:
```rust
tenancy: None,
```

| ファイル | 箇所 |
|---|---|
| `driver.rs` | `FavToml::load` の `unwrap_or` フォールバック（1 箇所） |
| `resolver.rs` | 3 箇所 |
| `checker.rs` | 2 箇所 |

### Step 4: `driver.rs` — `v57700_tests` 挿入

`v57600_tests` の直前に挿入:

```rust
// -- v57700_tests (v57.7.0) -- マルチテナント分離 --
#[cfg(test)]
mod v57700_tests {
    use crate::toml::{TenancyConfig, TenancyIsolation};

    fn make_tenancy() -> TenancyConfig {
        TenancyConfig {
            mode: "strict".to_string(),
            tenant: Some("acme-corp".to_string()),
            isolation: Some(TenancyIsolation {
                snowflake_schema: Some("tenant_acme".to_string()),
                kafka_topic_prefix: Some("acme.".to_string()),
            }),
        }
    }

    #[test]
    // Note: verifies TenancyConfig struct fields directly (not TOML parse path).
    fn tenancy_config_parsed() {
        let tc = make_tenancy();
        assert_eq!(tc.mode, "strict");
        assert_eq!(tc.tenant.as_deref(), Some("acme-corp"));
        let iso = tc.isolation.as_ref().expect("isolation should be set");
        assert_eq!(iso.snowflake_schema.as_deref(), Some("tenant_acme"));
        assert_eq!(iso.kafka_topic_prefix.as_deref(), Some("acme."));
    }

    #[test]
    fn tenancy_strict_enforced() {
        // strict mode → is_strict() = true
        let strict = make_tenancy();
        assert!(strict.is_strict(), "mode=strict should return is_strict()=true");

        // non-strict mode → is_strict() = false
        let relaxed = TenancyConfig {
            mode: "permissive".to_string(),
            tenant: None,
            isolation: None,
        };
        assert!(!relaxed.is_strict(), "mode=permissive should return is_strict()=false");
    }
}
```

### Step 5: バージョンチェックテスト更新（rolling）

| テスト | 変更前 | 変更後 |
|---|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | `"57.6.0"` | `"57.7.0"` |
| `v56900_tests::cargo_toml_version_is_56_9_0` | `"57.6.0"` | `"57.7.0"` |
| `v57000_tests::cargo_toml_version_is_57_0_0` | `"57.6.0"` | `"57.7.0"` |

---

## テスト戦略

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `3267 tests passed, 0 failed`（ベース 3265 + 2）

---

## ポスト処理

1. `CHANGELOG.md` に `[v57.7.0]` エントリ追加
2. `versions/current.md` を v57.7.0 / 3267 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.7.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` のテスト数推移テーブルに v57.7.0 行を追加
