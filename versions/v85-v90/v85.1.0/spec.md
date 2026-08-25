# Spec: v85.1.0 — `SapTomlConfig` + `inject_sap_config()`

## Background

v85.0.0「Favnir 4.0 宣言」をもって Quality-First Era が完成した。
本バージョンから **SAP Integration Era**（v85.1〜v90.0）を開始する。

最初のステップとして、`fav.toml [sap]` セクションを解析し、
SAP 接続情報を環境変数に注入する Rust 基盤を構築する。
Snowflake（v10.7.0）・Postgres（v11.5.0）と同じパターンで実装する。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.1.0 セクション）

## Goals

- `fav/src/toml.rs` に `SapTomlConfig` 構造体を追加する
- `FavTomlProject` に `sap: Option<SapTomlConfig>` フィールドを追加する
- `fav/src/driver.rs` に `inject_sap_config(cfg: &SapTomlConfig)` を追加する
- `cmd_run` / `cmd_check` の設定読み込み箇所で `inject_sap_config` を呼ぶ
- Rust テスト 2 件を追加して **3,933 tests** を達成する

## API / Type Definitions

```rust
// fav/src/toml.rs（既存 FavTomlProject の後に追加）

#[derive(Debug, Deserialize, Default)]
pub struct SapTomlConfig {
    pub base_url: Option<String>,  // "https://my-s4hana.example.com"
    pub client:   Option<String>,  // SAP クライアント番号（例: "100"）
    pub username: Option<String>,  // "${SAP_USER}" 形式の env var 展開対応
    pub password: Option<String>,  // "${SAP_PASS}" 形式
    pub auth:     Option<String>,  // "basic" | "oauth2"（デフォルト "basic"）
}

// FavTomlProject に追加するフィールド
pub struct FavTomlProject {
    // ... 既存フィールド ...
    pub sap: Option<SapTomlConfig>,
}
```

```rust
// fav/src/driver.rs（inject_snowflake_config の近くに追加）

/// fav.toml [sap] の設定を環境変数に注入する。
/// ${VAR} 形式は expand_env_vars() で展開される。
pub fn inject_sap_config(cfg: &SapTomlConfig) {
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

### `fav.toml` での記述例

```toml
[sap]
base_url = "https://my-s4hana.example.com"
client   = "100"
username = "${SAP_USER}"
password = "${SAP_PASS}"
auth     = "basic"
```

## Success Criteria

- `cargo test` が **3,933 tests**, 0 failures
- `sap_toml_config_parses_base_url`:
  - TOML 文字列に `[sap]\nbase_url = "https://example.com"` を含む場合、
    `SapTomlConfig.base_url` が `Some("https://example.com")` であること
- `inject_sap_config_sets_env_vars`:
  - `SapTomlConfig { base_url: Some("https://test.example.com"), ... }` を渡すと
    `std::env::var("SAP_BASE_URL")` が `"https://test.example.com"` を返すこと

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/toml.rs` | 追記 | `SapTomlConfig` 構造体、`FavTomlProject.sap` フィールド |
| `fav/src/driver.rs` | 追記 | `inject_sap_config()` 関数、`mod v85100_tests`（テスト 2 件）、`cmd_run` / `cmd_check` での呼び出し |

## Error Codes

新規エラーコードなし。

## 注記

- `expand_env_vars()` は既存関数（Snowflake / Postgres で使用済み）を再利用する
- `inject_sap_config` の呼び出しタイミングは `inject_snowflake_config` と同箇所（`FavTomlProject` を読み込んだ直後）
- WASM ビルド非対象（`cfg(not(target_arch = "wasm32"))` は不要 — env var は WASM 未使用）
- `inject_sap_config_sets_env_vars` は `std::env::set_var` を使用するため、並行テストとの env var 競合に注意。既存の Snowflake テストと同じ対処方針（テスト固有の値を使う、必要に応じて事後 `std::env::remove_var` でクリーンアップ）を採用すること
- `cmd_run` の設定読み込み箇所で `inject_sap_config` が呼ばれていること（コードレビューまたは smoke test で確認）
- Success Criteria のテストでは `SAP_BASE_URL` と `SAP_CLIENT` の 2 変数を確認する（全 5 変数の確認は任意）
