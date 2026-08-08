# Spec — v57.7.0 — マルチテナント分離

## 概要

`fav.toml` の `[tenancy]` / `[tenancy.isolation]` セクションを解析し、
テナント識別子・分離設定を保持する `TenancyConfig` / `TenancyIsolation` 構造体を
`toml.rs` に追加する。`is_strict()` メソッドで `strict` モード設定かどうかを判定できる。

> **スコープ注意**: 実際の Rune エンドポイントへのテナント識別子自動挿入・
> `strict` モード時の E0425 エラー発行（checker 統合）・
> `fav run --tenant` CLI フラグは v57.7.0 のスコープ外。
> 本バージョンは TOML パース層と `TenancyConfig` / `TenancyIsolation` データ構造の確立に集中する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.7.0 セクション
- ベーステスト数: **3265**（v57.6.0 完了時点の実績値）
- 目標テスト数: **3267**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| Rune エンドポイントへのテナント識別子自動挿入 | 未定（別途スプリント設計時に確定） | Rune クライアント実装と結合が必要 |
| `strict` モード時の E0425 エラー発行（checker 統合） | 未定（別途スプリント設計時に確定） | checker 改修は独立スプリントで対応 |
| `fav run --tenant` CLI フラグ | 未定（別途スプリント設計時に確定） | CLI 層の改修は独立スプリントで対応 |
| サイトドキュメント（`enterprise/tenancy.mdx`） | v57.8.0 | Enterprise Security ドキュメントまとめ対応 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.7.0"
```

---

### 2. `fav/src/toml.rs` — `TenancyIsolation` + `TenancyConfig` 構造体追加

`TlsConfig` の直後（`FavToml` の直前）に挿入する。

#### 2-1: `TenancyIsolation` 構造体

```rust
/// Per-rune isolation settings from `[tenancy.isolation]` (v57.7.0).
#[derive(Debug, Clone, Default)]
pub struct TenancyIsolation {
    /// Snowflake schema prefix pattern (e.g. `"tenant_${TENANT_ID}"`).
    pub snowflake_schema: Option<String>,
    /// Kafka topic prefix pattern (e.g. `"${TENANT_ID}."`).
    pub kafka_topic_prefix: Option<String>,
}
```

#### 2-2: `TenancyConfig` 構造体

```rust
/// Multi-tenant configuration from `[tenancy]` in fav.toml (v57.7.0).
#[derive(Debug, Clone, Default)]
pub struct TenancyConfig {
    /// Tenancy mode: `"strict"` enforces tenant ID on all Rune access.
    /// Default (when `[tenancy]` section is absent or `mode` is unset): `""` (empty string = permissive-equivalent).
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

#### 2-3: `FavToml` 構造体に `tenancy` フィールドを追加

`pub tls: Option<TlsConfig>,` の直後に追加:
```rust
/// Optional multi-tenant configuration (v57.7.0).
pub tenancy: Option<TenancyConfig>,
```

#### 2-4: `parse_fav_toml` のセクション解析に追加

`[security.tls]` 検出の直後に挿入:
```rust
} else if trimmed == "[tenancy]" {
    section = "tenancy";
} else if trimmed == "[tenancy.isolation]" {
    section = "tenancy.isolation";
```

`"tenancy"` アームの処理（`_ => {}` の直前に追加）:
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

アキュムレータを `parse_fav_toml` の先頭で宣言（`tls_cfg` パターンと同様）:
```rust
let mut tenancy_cfg: Option<TenancyConfig> = None;
```

#### 2-5: `FavToml` 返却部に `tenancy` フィールドを追加

```rust
tenancy: tenancy_cfg,
```

#### 2-6: `FavToml` 直接初期化 6 箇所に `tenancy: None` を追加

| ファイル | 箇所数 |
|---|---|
| `driver.rs` | 1（`FavToml::load` の `unwrap_or` フォールバック） |
| `resolver.rs` | 3 |
| `checker.rs` | 2 |
| **合計** | **6** |

各箇所で `tls: None,` の直後に `tenancy: None,` を追加。

---

### 3. `fav/src/driver.rs` — `v57700_tests` 追加

`v57600_tests` の直前に挿入する。

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

---

### 4. `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.6.0" → "57.7.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.6.0" → "57.7.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.6.0" → "57.7.0"（rolling）
```

> `v57100_tests` 〜 `v57600_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `tenancy_config_parsed` | `TenancyConfig` 全フィールド（mode / tenant / isolation.snowflake_schema / isolation.kafka_topic_prefix）を検証 |
| `tenancy_strict_enforced` | `is_strict()` が `"strict"` で `true`、`"permissive"` で `false` を返すことを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3267 tests passed, 0 failed**、ベース 3265 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57700_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.7.0]` エントリが追加されている
- `versions/current.md` が v57.7.0 / 3267 tests を反映

---

## 備考

- `TenancyIsolation` は `TenancyConfig` の直前（`TlsConfig` の直後）に配置
- `mode` フィールドは `String`（`Default` → `""`）。`[tenancy]` セクションが存在しない場合は `tenancy_cfg = None` のまま → `is_strict()` は呼ばれない。`mode` が指定されない場合（セクションはあるが `mode =` なし）は空文字列となり `is_strict() = false`（permissive 相当）
- `tenancy_config_parsed` テストは構造体を直接初期化して全フィールドを検証する（TOML パース経路テストは後続バージョンで追加予定）
- アキュムレータは `Option<TenancyConfig>` パターン（`rbac_cfg` / `tls_cfg` と同様）
- `[tenancy.isolation]` 検出時、`tenancy_cfg` がまだ None なら `get_or_insert_with` で初期化してから `isolation` を設定する
- 文字列値には必ず `expand_env_vars` を適用
- `driver.rs`（1）/ `resolver.rs`（3）/ `checker.rs`（2）の `FavToml { ... }` 直接初期化（計 6 箇所）に `tenancy: None` が必要
- `v57600_tests` 〜 `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
