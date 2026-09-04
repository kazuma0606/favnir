# Spec: v96.2.0 — `fav.toml [sap.environments]` マルチ環境設定

## Background

v96.1.0 で `SapEnvironment` 型と `Ctx.sap_env()` スタブを追加したが、
実際の環境切替には接続先情報（base_url / client / username / password）が必要。
v96.2.0 では `fav.toml` に `[sap.environments.<NAME>]` セクションを追加し、
PRD / QAS / DEV の各環境接続設定を記述できるようにする。

これにより `Ctx.sap_env("PRD")` の本実装（v96.2.0 以降）の基盤が整う。

## Goals

1. `toml.rs` に `SapEnvEntry`（単一環境設定）と `SapEnvironmentsConfig`（環境マップ）構造体を追加する
2. `SapTomlConfig` に `environments: HashMap<String, SapEnvEntry>` フィールドを追加する
3. `[sap.environments.PRD]` 形式のセクションをパースする処理を `toml.rs` に追加する
4. `driver.rs` に `mod v96200_tests`（2 テスト）を追加する

## TOML 構文例

```toml
[sap]
base_url = "${SAP_BASE_URL}"
client   = "100"
username = "${SAP_USER}"
password = "${SAP_PASS}"

[sap.environments.PRD]
base_url = "${SAP_PRD_URL}"
client   = "100"
username = "${SAP_PRD_USER}"
password = "${SAP_PRD_PASS}"

[sap.environments.QAS]
base_url = "${SAP_QAS_URL}"
client   = "200"
username = "${SAP_QAS_USER}"
password = "${SAP_QAS_PASS}"
```

## 型定義（Rust）

```rust
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

`SapTomlConfig` に追加するフィールド:
```rust
pub environments: SapEnvironmentsConfig,
```

> **注意**: `SapTomlConfig` には既存フィールド（`base_url / client / username / password / auth`）がある（v85.1.0 実装済み）。`environments` はその末尾に追加する。既存フィールドは変更しない。

## パース方針

`[sap.environments.PRD]` のようなセクションヘッダーは以下の形式で検出する:

```
trimmed.starts_with("[sap.environments.") && trimmed.ends_with(']')
```

環境名を抽出して `SapEnvironmentsConfig` マップに挿入する。
既存の `[sap]` セクションのパースは変更しない。

## Success Criteria

- `toml.rs` に `SapEnvEntry` 構造体と `SapEnvironmentsConfig` 型エイリアスが定義される
- `SapTomlConfig` に `environments` フィールドが追加される
- `cargo test` で 4,192 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `SapEnvEntry` 構造体・`SapEnvironmentsConfig` 型・`SapTomlConfig.environments` フィールド・パース処理を追加 |
| `fav/src/driver.rs` | `mod v96200_tests`（2 テスト）を追加 |
