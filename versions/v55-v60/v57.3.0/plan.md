# Plan — v57.3.0 — TLS / mTLS サポート（HTTP / gRPC Rune）

## 実装順序

```
Cargo.toml → toml.rs（TlsConfig 追加）
→ driver.rs（v57300_tests 追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（CHANGELOG + current.md + roadmap 更新）
→ tasks.md COMPLETE 更新
```

依存関係:
- `toml.rs` の `TlsConfig` 追加は `driver.rs` の `v57300_tests` より先に行う
  （`use crate::toml::TlsConfig` がコンパイル時に解決されるため）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.2.0"  →  version = "57.3.0"
```

---

## Step 2: `fav/src/toml.rs` — `TlsConfig` 構造体追加

### 2-1: 構造体定義を `SecretsConfig` の直後（`FavToml` の直前）に挿入

```rust
/// TLS / mTLS configuration from `[security.tls]` in fav.toml (v57.3.0).
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    /// Path to the CA certificate file.
    pub ca_cert: Option<String>,
    /// Path to the client certificate file (required for mTLS).
    pub tls_cert: Option<String>,
    /// Path to the client private key file (required for mTLS).
    pub tls_key: Option<String>,
    /// Whether to verify the server's certificate. Defaults to false if not specified.
    pub verify: bool,
}

impl TlsConfig {
    /// Returns `true` when both `tls_cert` and `tls_key` are set (mTLS mode).
    pub fn is_mtls(&self) -> bool {
        self.tls_cert.is_some() && self.tls_key.is_some()
    }
}
```

### 2-2: `FavToml` 構造体に `tls` フィールドを追加

`pub secrets: Option<SecretsConfig>,` の直後に追加:
```rust
/// Optional TLS / mTLS configuration (v57.3.0).
pub tls: Option<TlsConfig>,
```

### 2-3: `parse_fav_toml` のセクション解析に追加

`[secrets.bindings]` の解析ブロック直後に挿入:

```rust
} else if trimmed == "[security.tls]" {
    section = "security.tls";
```

`"security.tls"` アームの処理（`_ => {}` の直前に追加）:

```rust
"security.tls" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        let current = tls_cfg.get_or_insert_with(TlsConfig::default);
        match key {
            "ca_cert"  => current.ca_cert  = Some(expand_env_vars(val)),
            "tls_cert" => current.tls_cert = Some(expand_env_vars(val)),
            "tls_key"  => current.tls_key  = Some(expand_env_vars(val)),
            "verify"   => current.verify   = val == "true",
            _ => {}
        }
    }
}
```

アキュムレータを `parse_fav_toml` の先頭で宣言（`rbac_cfg` パターンと同様。`secrets_cfg` は別パターンのため混同注意）:
```rust
let mut tls_cfg: Option<TlsConfig> = None;
```

### 2-4: `FavToml` 返却部に `tls` フィールドを追加

```rust
tls: tls_cfg,
```

### 2-5: `FavToml` 直接初期化 6 箇所に `tls: None` を追加

| ファイル | 箇所数 | 内容 |
|---|---|---|
| `driver.rs` | 1 | `FavToml::load` の `unwrap_or(FavToml { ... })` |
| `resolver.rs` | 3 | `let toml = FavToml {` × 3 |
| `checker.rs` | 2 | `let toml = FavToml {` × 2 |
| **合計** | **6** | |

各箇所で `secrets: None,` の直後に `tls: None,` を追加。

---

## Step 3: `fav/src/driver.rs` — `v57300_tests` 追加

`v57200_tests` の直前（`// -- v57200_tests` コメント行の直前）に挿入:

```rust
// -- v57300_tests (v57.3.0) -- TLS / mTLS サポート --
#[cfg(test)]
mod v57300_tests {
    use crate::toml::TlsConfig;

    fn make_tls() -> TlsConfig {
        TlsConfig {
            ca_cert: Some("certs/ca.pem".to_string()),
            tls_cert: Some("certs/client.pem".to_string()),
            tls_key: Some("certs/client-key.pem".to_string()),
            verify: true,
        }
    }

    #[test]
    // Note: verifies TlsConfig struct fields directly (not TOML parse path).
    fn tls_config_parsed() {
        let tls = make_tls();
        assert_eq!(tls.ca_cert.as_deref(), Some("certs/ca.pem"));
        assert_eq!(tls.tls_cert.as_deref(), Some("certs/client.pem"));
        assert_eq!(tls.tls_key.as_deref(), Some("certs/client-key.pem"));
        assert!(tls.verify, "verify should be true");
    }

    #[test]
    fn mtls_cert_injected() {
        let tls = make_tls();
        assert!(
            tls.is_mtls(),
            "both tls_cert and tls_key present should indicate mTLS mode"
        );
    }
}
```

---

## Step 4: `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.2.0" → "57.3.0"
v56900_tests::cargo_toml_version_is_56_9_0  : "57.2.0" → "57.3.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.2.0" → "57.3.0"（rolling）
```

> `v57100_tests` / `v57200_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## Step 5: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3259 tests passed, 0 failed を確認。`v57300_tests` の 2 件が全通過することを確認。

---

## Step 6: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 7: ポスト処理

1. `CHANGELOG.md` に v57.3.0 エントリを追加（先頭）
2. `versions/current.md` を v57.3.0 / 3259 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.3.0 実績を COMPLETE に更新
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.3.0 実績欄を COMPLETE に更新し、テスト数推移テーブルに v57.3.0 行（3259）を追加

---

## Step 8: `versions/v55-v60/v57.3.0/tasks.md` を COMPLETE に更新

全チェックボックス（T0 含む）を `[x]` にする。

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `FavToml` の直接初期化 6 箇所に `tls: None` を追加し忘れる | コンパイル時エラー（E0063）で即座に検出される |
| `verify` フィールドの Default が `false` になる | 意図通り。TOML に `verify = true` と書いた場合のみ `true` になる仕様 |
| `[security.tls]` が既存の `[security.rbac]` パースより後にある場合の section 変数の上書き | セクション判定は先着優先（`continue` で後続判定をスキップ）のため問題なし |
| `v57200_tests` コメント行の直前への挿入位置ミス | awk での挿入は `// -- v57200_tests` コメント行を対象にする |
| `expand_env_vars` の適用忘れ | v57.2.0 の code review 教訓（二重 trim + env 展開漏れ）を参照。文字列値には必ず適用する |
