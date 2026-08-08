# Spec — v57.2.0 — シークレット管理統合（Vault / AWS Secrets Manager）

## 概要

`fav.toml` の `[secrets]` セクションを解析し、AWS Secrets Manager / HashiCorp Vault への
バインディングを設定する `SecretsConfig` 構造体を `toml.rs` に追加する。
`list_keys` メソッドで登録されたシークレットキー名を列挙できる。

> **スコープ注意**: 実際の AWS API 呼び出し・環境変数注入・`fav secrets rotate` は v57.2.0 のスコープ外。
> 本バージョンは TOML パース層と `SecretsConfig` データ構造の確立に集中する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.2.0 セクション
- ベーステスト数: **3255**（v57.1.0 完了時点の実績値）
- 目標テスト数: **3257**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| 実際の AWS Secrets Manager API 呼び出し | v57.2.0+ | ネットワーク依存・認証設定を要する。TOML 層の先行整備が先決 |
| HashiCorp Vault 呼び出し | v57.2.0+ | 同上 |
| `fav run --inject-secrets` フラグ | v57.2.0+ | CLI 統合は SecretsConfig 安定化後 |
| `fav secrets list` CLI コマンド実装 | v57.2.0+ | ロードマップに記載あり。本バージョンは `list_keys()` メソッドのみ実装し、CLI ディスパッチは後続で対応 |
| `fav secrets rotate` コマンド | v57.2.0+ | ロードマップ記載だが TOML 基盤整備が先 |
| サイトドキュメント（`enterprise/secrets.mdx`） | v57.8.0 | Enterprise Security ドキュメントまとめ対応 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.2.0"
```

---

### 2. `fav/src/toml.rs` — `SecretsConfig` 構造体追加

`RbacConfig` の直後（`FavToml` の直前）に挿入する。

```rust
/// Secrets management configuration from `[secrets]` in fav.toml (v57.2.0).
#[derive(Debug, Clone, Default)]
pub struct SecretsConfig {
    /// Provider name: "aws-secrets-manager" or "vault".
    pub provider: String,
    /// AWS region (or Vault address) for the provider.
    pub region: String,
    /// ENV_VAR_NAME → provider secret path mapping.
    /// e.g. "SNOWFLAKE_PASSWORD" → "prod/snowflake/password"
    pub bindings: std::collections::HashMap<String, String>,
}

impl SecretsConfig {
    /// Returns a sorted list of registered secret key names (env var names).
    pub fn list_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.bindings.keys().map(|k| k.as_str()).collect();
        keys.sort();
        keys
    }
}
```

`FavToml` 構造体に追加:
```rust
/// Optional secrets management configuration (v57.2.0).
pub secrets: Option<SecretsConfig>,
```

`parse_fav_toml` のセクション解析に追加（`[security.rbac.bindings]` の直後）:
- `"[secrets]"` セクション: `provider = "..."` / `region = "..."` をパース
- `"[secrets.bindings]"` セクション: `ENV_KEY = "secret/path"` 形式をパース

`parse_fav_toml` の `FavToml { ... }` 返却部に `secrets: secrets_cfg` を追加。

`FavToml` 直接初期化 6 箇所に `secrets: None` を追加:
- `driver.rs` 1 箇所（`FavToml::load` の `unwrap_or` フォールバック）
- `resolver.rs` 3 箇所
- `checker.rs` 2 箇所

> **パース仕様**: 単一行 `key = "value"` のみサポート（複数行形式は v57.2.0 スコープ外）。
> **None 判定**: `[secrets]` セクションが存在しても `provider` が空文字列の場合は `None` として扱う（`region` のみ・`bindings` のみ指定の場合も同様）。

---

### 3. `fav/src/driver.rs` — `v57200_tests` 追加

`v57100_tests` の直前に挿入する。

```rust
// -- v57200_tests (v57.2.0) -- シークレット管理統合 --
#[cfg(test)]
mod v57200_tests {
    use crate::toml::SecretsConfig;

    fn make_secrets() -> SecretsConfig {
        let mut bindings = std::collections::HashMap::new();
        bindings.insert(
            "SNOWFLAKE_PASSWORD".to_string(),
            "prod/snowflake/password".to_string(),
        );
        bindings.insert(
            "KAFKA_API_KEY".to_string(),
            "prod/kafka/api-key".to_string(),
        );
        SecretsConfig {
            provider: "aws-secrets-manager".to_string(),
            region: "ap-northeast-1".to_string(),
            bindings,
        }
    }

    #[test]
    fn secrets_provider_config_parsed() {
        let cfg = make_secrets();
        assert_eq!(cfg.provider, "aws-secrets-manager");
        assert_eq!(cfg.region, "ap-northeast-1");
        assert_eq!(cfg.bindings.len(), 2);
        assert_eq!(
            cfg.bindings.get("SNOWFLAKE_PASSWORD").map(|s| s.as_str()),
            Some("prod/snowflake/password"),
        );
    }

    #[test]
    fn cmd_secrets_list() {
        let cfg = make_secrets();
        let keys = cfg.list_keys(); // sorted
        assert_eq!(keys, vec!["KAFKA_API_KEY", "SNOWFLAKE_PASSWORD"]);
    }
}
```

---

### 4. `fav/src/driver.rs` — バージョンチェックテスト更新

以下の rolling チェックテストの期待値を `"57.1.0"` → `"57.2.0"` に更新:

| テスト | 備考 |
|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | 期待値 + failure メッセージ両方更新 |
| `v56900_tests::cargo_toml_version_is_56_9_0` | rolling。期待値 + failure メッセージ両方更新 |
| `v57000_tests::cargo_toml_version_is_57_0_0` | rolling。期待値 + failure メッセージ両方更新 |

> `v57100_tests` には `cargo_toml_version_is_57_1_0` テストが存在しないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `secrets_provider_config_parsed` | `SecretsConfig` の `provider` / `region` / `bindings.len()` / 特定キーの値を検証 |
| `cmd_secrets_list` | `list_keys()` がソート済みキー名スライスを返すことを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3257 tests passed, 0 failed**、ベース 3255 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57200_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.2.0]` エントリが追加されている
- `versions/current.md` が v57.2.0 / 3257 tests を反映

---

## 備考

- `SecretsConfig` は `toml.rs` の `RbacConfig` の直後に配置（struct 群の末尾、`FavToml` の直前）
- `list_keys` は常にソート済みで返す（テストの安定性のため）
- `[secrets.bindings]` のパース: `KEY = "path"` の文字列値を読む（配列ではない点が `rbac.bindings` と異なる）
- `v57200_tests` は `use crate::toml::SecretsConfig` を使用（`use super::*` 不要）
- `v57100_tests` の直前に挿入。v57200_tests が v57100_tests の後ろに入るのは NG
  正しい順序: v57200_tests → v57100_tests → v57000_tests（新しいものほど上）
