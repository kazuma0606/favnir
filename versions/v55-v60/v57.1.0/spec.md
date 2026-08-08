# Spec — v57.1.0 — RBAC（ロールベースアクセス制御）for Rune

## 概要

`fav.toml` の `[security.rbac]` セクションを解析し、Rune へのアクセスをロールで制限する。
`RbacConfig` 構造体と `is_allowed` メソッドを `toml.rs` に追加。
E0424 エラーコード（RBAC アクセス拒否）を `error_catalog.rs` に追加。

テスト数補正: ロードマップ推定ベース 3250 → 実績 3252、目標 3254。

### v57.1.0 のスコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| checker でのロールコンテキスト検証（E0424 発火） | v57.2.0 以降 | `RbacConfig` の基盤整備を先行させ、checker 統合は独立スプリントで実施 |
| `fav run --role <role>` CLI フラグ | v57.2.0 以降 | checker 統合完成後に CLI フラグを追加する順序が自然 |
| サイトドキュメント（`enterprise/rbac.mdx`） | v57.8.0 | ロードマップ v57.8.0 で全 Enterprise Security ドキュメントをまとめて追加 |

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.1.0 セクション
- ベーステスト数: **3252**（v57.0.0 完了時点の実績値）
  - ※ロードマップは 3250 と記載しているが v57.0.0 で 4 件追加のため実際は 3252
- 目標テスト数: **3254**（+2）、かつ `cargo test` failures=0

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.1.0"
```

---

### 2. `fav/src/toml.rs` — `RbacConfig` 構造体追加

`FavToml` に `rbac: Option<RbacConfig>` フィールドを追加し、
`[security.rbac]` / `[security.rbac.bindings]` セクションをパースする。

```rust
/// RBAC configuration from `[security.rbac]` in fav.toml (v57.1.0).
#[derive(Debug, Clone, Default)]
pub struct RbacConfig {
    /// Declared roles (e.g. ["reader", "writer", "admin"]).
    pub roles: Vec<String>,
    /// Rune name → allowed roles mapping.
    /// If a rune is not listed, access is unrestricted.
    pub bindings: std::collections::HashMap<String, Vec<String>>,
}

impl RbacConfig {
    /// Returns `true` if `role` is allowed to access `rune`.
    /// If `rune` has no binding, access is unrestricted (returns `true`).
    pub fn is_allowed(&self, rune: &str, role: &str) -> bool {
        match self.bindings.get(rune) {
            Some(allowed) => allowed.iter().any(|r| r == role),
            None => true,
        }
    }
}
```

`FavToml` 構造体に追加:
```rust
/// Optional RBAC configuration (v57.1.0).
pub rbac: Option<RbacConfig>,
```

`parse_fav_toml` の section 解析に追加:
- `"security.rbac"` セクション: `roles` キーを `Vec<String>` としてパース
- `"security.rbac.bindings"` セクション: `"rune_name" = ["role1", "role2"]` をパース

---

### 3. `fav/src/error_catalog.rs` — E0424 追加

```rust
ErrorEntry {
    code: "E0424",
    title: "RBAC access denied",
    summary: "Current role does not have permission to access this Rune.",
    description: "The `[security.rbac.bindings]` configuration restricts access to \
                  this Rune to specific roles. The current role is not in the allowed list.",
    example: "# fav.toml: [security.rbac.bindings]\n# \"snowflake\" = [\"writer\", \"admin\"]\nfav run pipeline.fav --role reader  // E0424: reader cannot access snowflake",
    fix: "Use an allowed role (`fav run --role writer`) or update the RBAC bindings \
          in `fav.toml` to include your role.",
},
```

---

### 4. `fav/src/driver.rs` — `v57100_tests` 追加

`v57000_tests` の直前に挿入する。

```rust
// -- v57100_tests (v57.1.0) -- RBAC for Rune --
#[cfg(test)]
mod v57100_tests {
    use crate::toml::RbacConfig;

    fn make_rbac() -> RbacConfig {
        let mut bindings = std::collections::HashMap::new();
        bindings.insert(
            "snowflake".to_string(),
            vec!["writer".to_string(), "admin".to_string()],
        );
        RbacConfig {
            roles: vec![
                "reader".to_string(),
                "writer".to_string(),
                "admin".to_string(),
            ],
            bindings,
        }
    }

    #[test]
    fn rbac_access_denied() {
        let rbac = make_rbac();
        assert!(
            !rbac.is_allowed("snowflake", "reader"),
            "reader should not be allowed to access snowflake"
        );
    }

    #[test]
    fn rbac_access_granted() {
        let rbac = make_rbac();
        assert!(
            rbac.is_allowed("snowflake", "writer"),
            "writer should be allowed to access snowflake"
        );
    }
}
```

---

### 5. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"57.0.0"` → `"57.1.0"` に更新。
`v56900_tests::cargo_toml_version_is_56_9_0` の期待値（rolling）も `"57.0.0"` → `"57.1.0"` に更新。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `rbac_access_denied` | role "reader" が "snowflake" にアクセス不可（`is_allowed` → false） |
| `rbac_access_granted` | role "writer" が "snowflake" にアクセス可（`is_allowed` → true） |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3254 tests passed, 0 failed**、ベース 3252 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57100_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.1.0]` エントリが追加されている
- `versions/current.md` が v57.1.0 / 3254 tests を反映

---

## 備考

- `compiler.fav` / `checker.fav` は `FavToml` の Rust 定義を直接参照しない（セルフホストパスへの影響なし）
- `RbacConfig` は `toml.rs` に追加（`fav/src/toml.rs`）
- `is_allowed`: 対象 Rune にバインディングがない場合は unrestricted（true を返す）
- E0424 は `error_catalog.rs` の E0423 の直後に挿入
- `v57100_tests` は `use crate::toml::RbacConfig` を使用（`use super::*` 不要）
- ロードマップのテスト数 `3252`（ベース 3250 + 2）は v57.0.0 の実績（3252）と同値のため、
  本 spec では正しい目標値 **3254**（実績ベース 3252 + 2）を採用する
