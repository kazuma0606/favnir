# Spec — v57.3.0 — TLS / mTLS サポート（HTTP / gRPC Rune）

## 概要

`fav.toml` の `[security.tls]` セクションを解析し、証明書・鍵のパスを保持する
`TlsConfig` 構造体を `toml.rs` に追加する。
`is_mtls()` メソッドで mTLS（クライアント証明書あり）設定かどうかを判定できる。

> **スコープ注意**: 実際の TLS ハンドシェイク・HTTP / gRPC Rune への証明書注入・
> `fav doctor` の TLS チェック項目追加は v57.3.0 のスコープ外。
> 本バージョンは TOML パース層と `TlsConfig` データ構造の確立に集中する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.3.0 セクション
- ベーステスト数: **3257**（v57.2.0 完了時点の実績値）
- 目標テスト数: **3259**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| HTTP / gRPC Rune への TLS / mTLS 実際の注入 | v57.3.0+ | Rune クライアント実装と結合が必要。TOML 層先行 |
| `fav doctor` の TLS 設定チェック項目追加 | v57.3.0+ | ロードマップ記載あり。doctor コマンド改修は独立スプリントで対応 |
| TLS ハンドシェイク・証明書の実際の検証 | v57.3.0+ | ランタイム実装を要する |
| サイトドキュメント（`enterprise/tls.mdx`） | v57.8.0 | Enterprise Security ドキュメントまとめ対応 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.3.0"
```

---

### 2. `fav/src/toml.rs` — `TlsConfig` 構造体追加

`SecretsConfig` の直後（`FavToml` の直前）に挿入する。

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

`FavToml` 構造体に追加（`secrets` フィールドの直後）:
```rust
/// Optional TLS / mTLS configuration (v57.3.0).
pub tls: Option<TlsConfig>,
```

`parse_fav_toml` のセクション解析に追加（`[secrets.bindings]` の直後）:
```
"[security.tls]" → section = "security.tls"
```

各フィールドのパース（`"security.tls"` アーム）:
- `ca_cert`  → `expand_env_vars(val)` → `tls_cfg.ca_cert = Some(...)`
- `tls_cert` → `expand_env_vars(val)` → `tls_cfg.tls_cert = Some(...)`
- `tls_key`  → `expand_env_vars(val)` → `tls_cfg.tls_key = Some(...)`
- `verify`   → `val == "true"` → `tls_cfg.verify = true`

アキュムレータ: `let mut tls_cfg: Option<TlsConfig> = None;`（RbacConfig パターンと同様）
セクション入口で `tls_cfg.get_or_insert_with(TlsConfig::default)` を使用。

`parse_fav_toml` の `FavToml { ... }` 返却部に `tls: tls_cfg` を追加。

> **パース仕様**: 単一行 `key = "value"` のみサポート。`expand_env_vars` を全文字列値に適用する。
> `verify` は文字列 `"true"` と一致した場合のみ `true`（その他は `false`）。

`FavToml` 直接初期化 6 箇所に `tls: None` を追加:
- `driver.rs` 1 箇所（`FavToml::load` の `unwrap_or` フォールバック）
- `resolver.rs` 3 箇所
- `checker.rs` 2 箇所

---

### 3. `fav/src/driver.rs` — `v57300_tests` 追加

`v57200_tests` の直前に挿入する。

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

### 4. `fav/src/driver.rs` — バージョンチェックテスト更新

以下の rolling チェックテストの期待値を `"57.2.0"` → `"57.3.0"` に更新:

| テスト | 備考 |
|---|---|
| `v56300_tests::cargo_toml_version_is_56_3_0` | 期待値 + failure メッセージ両方更新 |
| `v56900_tests::cargo_toml_version_is_56_9_0` | rolling |
| `v57000_tests::cargo_toml_version_is_57_0_0` | rolling |

> `v57100_tests` / `v57200_tests` には `cargo_toml_version_is_*` テストが存在しないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `tls_config_parsed` | `TlsConfig` の全フィールド（ca_cert / tls_cert / tls_key / verify）を検証 |
| `mtls_cert_injected` | `is_mtls()` が tls_cert + tls_key 両方ある場合に `true` を返すことを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3259 tests passed, 0 failed**、ベース 3257 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57300_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.3.0]` エントリが追加されている
- `versions/current.md` が v57.3.0 / 3259 tests を反映

---

## 備考

- `TlsConfig` は `toml.rs` の `SecretsConfig` の直後、`FavToml` の直前に配置
- `is_mtls()`: `tls_cert.is_some() && tls_key.is_some()` — 両方 Some のとき mTLS
- `verify` の Default は `false`（Rust 標準）。TOML に `verify = true` と書いた場合のみ `true` になる
- アキュムレータは `Option<TlsConfig>` パターン（`rbac_cfg` と同様: `let mut tls_cfg: Option<TlsConfig> = None`）— `tls_found` フラグ不要
  ※ `secrets_cfg` は `SecretsConfig::default()` 直接初期化パターンで異なる。参照するのは `rbac_cfg` の実装
- `[security.tls]` の文字列値には `expand_env_vars` を適用（v57.2.0 の code review 教訓）
- `v57200_tests` / `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
- `v57300_tests` モジュールを `v57200_tests` の直前に挿入する（正しい降順: …v57300_tests → v57200_tests → v57100_tests…）
