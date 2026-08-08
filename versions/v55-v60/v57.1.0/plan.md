# Plan — v57.1.0 — RBAC（ロールベースアクセス制御）for Rune

## 実装順序

```
Cargo.toml → toml.rs（RbacConfig 追加）→ error_catalog.rs（E0424 追加）
→ driver.rs（v57100_tests 追加 + バージョンチェック更新）
→ cargo test 全通過確認 → cargo clippy クリーン確認
→ ポスト処理（CHANGELOG + current.md + roadmap 更新）
```

依存関係:
- `toml.rs` の `RbacConfig` 追加は `driver.rs` の `v57100_tests` より先に行う
  （`use crate::toml::RbacConfig` がコンパイル時に解決されるため）
- `error_catalog.rs` / `Cargo.toml` / `toml.rs` は互いに独立（並行可）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "57.1.0"
```

---

## Step 2: `fav/src/toml.rs` — `RbacConfig` 構造体追加

### 2-1: 構造体定義を既存の struct 群の末尾（`FavToml` の直前）に挿入

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

### 2-2: `FavToml` 構造体に `rbac` フィールドを追加

`runes` フィールドの直後に追加:
```rust
/// Optional RBAC configuration (v57.1.0).
pub rbac: Option<RbacConfig>,
```

### 2-3: `parse_fav_toml` の section ディスパッチに追加

section 変数のマッチング部分に以下を追加:
```rust
"[security.rbac]" => section = "security.rbac",
"[security.rbac.bindings]" => section = "security.rbac.bindings",
```

各 section のパース処理:
- `"security.rbac"` の `roles = [...]` を `Vec<String>` としてパース
- `"security.rbac.bindings"` の `key = [...]` を `HashMap<String, Vec<String>>` としてパース

### 2-4: `FavToml` のデフォルト初期化に `rbac: None` を追加

`parse_fav_toml` の `FavToml { ... }` 返却部に `rbac: None` を追加し、
パース後に `rbac_config` を `Some(...)` で設定する。

---

## Step 3: `fav/src/error_catalog.rs` — E0424 追加

E0423 エントリの直後に挿入する:

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

## Step 4: `fav/src/driver.rs` — `v57100_tests` 追加

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

## Step 5: `fav/src/driver.rs` — バージョンチェックテスト更新

### v56300_tests::cargo_toml_version_is_56_3_0

```rust
// 変更前
cargo_toml.contains("version = \"57.0.0\"")
"Cargo.toml version should be 57.0.0, got: {}"

// 変更後
cargo_toml.contains("version = \"57.1.0\"")
"Cargo.toml version should be 57.1.0, got: {}"
```

### v56900_tests::cargo_toml_version_is_56_9_0（rolling）

```rust
// 変更前
cargo_toml.contains("version = \"57.0.0\"")
"Cargo.toml version should be 57.0.0 (rolling check from v56.9.0), got: {}"

// 変更後
cargo_toml.contains("version = \"57.1.0\"")
"Cargo.toml version should be 57.1.0 (rolling check from v56.9.0), got: {}"
```

---

## Step 6: `cargo test` 全通過確認

```bash
cargo test -j 8 -- --test-threads=8
```

3254 tests passed, 0 failed を確認する。`v57100_tests` の 2 件が全通過することを確認する。

---

## Step 7: `cargo clippy` クリーン確認

```bash
cargo clippy -- -D warnings
```

---

## Step 8: ポスト処理

1. `CHANGELOG.md` に v57.1.0 エントリを追加
2. `versions/current.md` を v57.1.0 / 3254 tests に更新
3. `versions/roadmap/roadmap-v57.1-v58.0.md` の v57.1.0 実績を COMPLETE に更新
   - `3252 + 2 = 3254 tests passed, 0 failed（2026-07-27）` を追記
4. `versions/roadmap/roadmap-v55.1-v60.0.md` の v57.1.0 実績欄も COMPLETE に更新

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `parse_fav_toml` の `FavToml` 返却部に `rbac` フィールドを追加し忘れる | コンパイル時エラーで検出できる（struct literal に全フィールド必須） |
| `[security.rbac.bindings]` のネストセクション名がパーサーと食い違う | T0 で既存セクション名の文字列パターンを確認してから追加 |
| ロードマップのテスト数（3252）と実際の目標（3254）の乖離 | 本 spec・plan・tasks は 3254 を正式目標とする（ロードマップは v57.1.0 COMPLETE 更新時に修正） |
| `RbacConfig` に `Default` derive が必要（`FavToml` の初期化で使う可能性） | `#[derive(Debug, Clone, Default)]` を付与する |
