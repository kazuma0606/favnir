# Plan — v57.2.0 — シークレット管理統合（Vault / AWS Secrets Manager）

## 実装順序

```
Cargo.toml → toml.rs（SecretsConfig 追加）
→ driver.rs（v57200_tests 追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（CHANGELOG + current.md + roadmap 更新）
→ tasks.md COMPLETE 更新
```

依存関係:
- `toml.rs` の `SecretsConfig` 追加は `driver.rs` の `v57200_tests` より先に行う
  （`use crate::toml::SecretsConfig` がコンパイル時に解決されるため）
- `Cargo.toml` / `toml.rs` は互いに独立（並行可）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.1.0"  →  version = "57.2.0"
```

---

## Step 2: `fav/src/toml.rs` — `SecretsConfig` 構造体追加

### 2-1: 構造体定義を `RbacConfig` の直後（`FavToml` の直前）に挿入

```rust
/// Secrets management configuration from `[secrets]` in fav.toml (v57.2.0).
#[derive(Debug, Clone, Default)]
pub struct SecretsConfig {
    /// Provider name: "aws-secrets-manager" or "vault".
    pub provider: String,
    /// AWS region (or Vault address) for the provider.
    pub region: String,
    /// ENV_VAR_NAME → provider secret path mapping.
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

### 2-2: `FavToml` 構造体に `secrets` フィールドを追加

`rbac: Option<RbacConfig>` フィールドの直後に追加:
```rust
/// Optional secrets management configuration (v57.2.0).
pub secrets: Option<SecretsConfig>,
```

### 2-3: `parse_fav_toml` のセクション解析に追加

`[security.rbac.bindings]` の解析ブロックの直後に挿入:

```rust
} else if trimmed == "[secrets]" {
    section = "secrets";
} else if trimmed == "[secrets.bindings]" {
    section = "secrets.bindings";
```

各セクションのキー処理:
- `section == "secrets"`:
  - `key == "provider"` → `secrets_cfg.provider = value`
  - `key == "region"` → `secrets_cfg.region = value`
- `section == "secrets.bindings"`:
  - `key = "secret/path"` 形式 → `secrets_cfg.bindings.insert(key, value)`

アキュムレータを `parse_fav_toml` の先頭で宣言:
```rust
let mut secrets_cfg = SecretsConfig::default();
```

### 2-4: `FavToml` 返却部に `secrets` フィールドを追加

`parse_fav_toml` の `FavToml { ... }` 返却部に追加:
```rust
secrets: if secrets_cfg.provider.is_empty() { None } else { Some(secrets_cfg) },
```

### 2-5: `FavToml` 直接初期化 5 箇所に `secrets: None` を追加

| ファイル | 箇所数 | 内容 |
|---|---|---|
| `driver.rs` | 1 | `FavToml::load` の `unwrap_or(FavToml { ... })` |
| `resolver.rs` | 3 | `let toml = FavToml {` × 3 |
| `checker.rs` | 2 | `let toml = FavToml {` × 2 |
| **合計** | **6** | |

各箇所で `rbac: None,` の直後に `secrets: None,` を追加。

---

## Step 3: `fav/src/driver.rs` — `v57200_tests` 追加

`v57100_tests` の直前（`// -- v57100_tests` コメント行の直前）に挿入:

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

## Step 4: `fav/src/driver.rs` — バージョンチェックテスト更新

### v56300_tests::cargo_toml_version_is_56_3_0

```
"57.1.0" → "57.2.0"（contains 文字列 + failure メッセージの両方）
```

### v56900_tests::cargo_toml_version_is_56_9_0（rolling）

```
"57.1.0" → "57.2.0"（contains 文字列 + failure メッセージの両方）
```

### v57000_tests::cargo_toml_version_is_57_0_0（rolling）

```
"57.1.0" → "57.2.0"（contains 文字列 + failure メッセージの両方）
```

> `v57100_tests` には `cargo_toml_version_is_57_1_0` がないため更新不要。

---

## Step 5: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3257 tests passed, 0 failed を確認。`v57200_tests` の 2 件が全通過することを確認。

---

## Step 6: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 7: ポスト処理

1. `CHANGELOG.md` に v57.2.0 エントリを追加（先頭）
2. `versions/current.md` を v57.2.0 / 3257 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.2.0 実績を COMPLETE に更新
   - `3255 + 2 = 3257 tests passed, 0 failed（日付）` を追記
4. `versions/roadmap/roadmap-v55.1-v60.0.md` のテスト数推移テーブルに v57.2.0 行を追加

---

## Step 8: `versions/v55-v60/v57.2.0/tasks.md` を COMPLETE に更新

全チェックボックス（T0 含む）を `[x]` にする。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `FavToml` の直接初期化 5 箇所に `secrets: None` を追加し忘れる | コンパイル時エラー（E0063）で即座に検出される |
| `[secrets.bindings]` のパース: 値が文字列（`"path"`）で配列ではない | `rbac.bindings` と混同しないよう注意。`"` を strip して文字列として取得 |
| `list_keys` のソート漏れ | `cmd_secrets_list` テストで順序を検証しているため自動検出される |
| `v57100_tests` への挿入位置ミス（後ろに入る） | awk での挿入は `// -- v57100_tests` コメント行の直前を対象にする |
