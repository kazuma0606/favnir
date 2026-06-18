# v17.8.0 — パッケージシステム成熟 タスク

## ステータス: 完了

---

## タスク一覧

### T1: `fav/src/toml.rs` — `[dev-dependencies]` / `[registry]` 対応

- [x] `FavToml` 構造体に `dev_dependencies: Vec<DependencySpec>` フィールド追加
- [x] `FavToml` 構造体に `registry_url: Option<String>` フィールド追加
- [x] `parse_fav_toml` に `[dev-dependencies]` セクション解析を追加
- [x] `parse_fav_toml` に `[registry]` の `url` キー解析を追加
- [x] `FavToml` のデフォルト初期化を更新（新フィールドを `Vec::new()` / `None` に）
- [x] `fav_toml_to_string`（または `save_fav_toml`）の実装を追加/更新（`[dependencies]` の書き出し対応）

### T2: `fav/src/lock.rs` — `checksum` / `source` フィールド追加

- [x] `LockedPackage` 構造体に `checksum: Option<String>` フィールド追加
- [x] `LockedPackage` 構造体に `source: Option<String>` フィールド追加
- [x] `LockFile::to_toml` に `checksum` / `source` の書き出しを追加
- [x] `LockFile::load` に `checksum` / `source` の読み込みを追加
- [x] 既存の `LockedPackage` 生成箇所（`cmd_install` 等）のコンパイルエラーを修正

### T3: `fav/src/registry/resolver.rs` — Semver 解決

- [x] `SemVer { major: u32, minor: u32, patch: u32 }` 構造体を実装
- [x] `VersionReq { Caret(SemVer) / Tilde(SemVer) / Exact(SemVer) / Any }` enum を実装
- [x] `parse_semver(s: &str) -> Option<SemVer>` を実装（`"2.1.0"` → `SemVer { 2, 1, 0 }`）
- [x] `parse_version_req(s: &str) -> Option<VersionReq>` を実装（`"^2.0.0"` / `"~2.1.0"` / `"=2.1.3"` / `"*"`）
- [x] `matches_req(req: &VersionReq, v: &SemVer) -> bool` を実装
  - Caret: major > 0 → same major; major == 0, minor > 0 → same minor; 0.0.x → exact
  - Tilde: same major.minor
  - Exact: exact match
  - Any: always true
- [x] `resolve_best(req: &VersionReq, available: &[SemVer]) -> Option<SemVer>` を実装（最新版を返す）

### T4: `fav/src/registry/client.rs` — Registry API クライアント

- [x] `PackageInfo { name: String, versions: Vec<String>, latest: String }` 構造体を実装
- [x] `RegistryClient { base_url: String, token: Option<String> }` 構造体を実装
- [x] `RegistryClient::new(base_url: &str) -> Self` を実装
- [x] `RegistryClient::fetch_package(&self, name: &str) -> Result<PackageInfo, String>` を実装
  - `REGISTRY_MOCK=1` 環境変数があればダミーレスポンス（`versions: ["1.0.0", "2.0.0", "2.1.0"]`, `latest: "2.1.0"`）を返す
  - 本番: `ureq` で `GET {base_url}/packages/{name}` を呼び JSON パース
- [x] `RegistryClient::publish(&self, name: &str, version: &str, dry_run: bool) -> Result<(), String>` を実装

### T5: `fav/src/registry/mod.rs` 作成

- [x] `pub mod client;` を追加
- [x] `pub mod resolver;` を追加

### T6: `fav/src/lib.rs` — `registry` module 追加

- [x] `pub mod registry;` を `lib.rs` に追加

### T7: `fav/src/driver.rs` — `cmd_add` / `cmd_update` / `cmd_remove` / `cmd_login` 追加

- [x] `cmd_add(name: &str, version_str: Option<&str>, dev: bool)` を実装
  - `@` でパッケージ名とバージョンを分割
  - `RegistryClient::fetch_package` でバージョン一覧取得
  - semver req 文字列を `"^{version}"` として構築
  - `fav.toml` の `dependencies`（または `dev_dependencies`）に追加して保存
  - `fav.lock` を更新して保存
- [x] `cmd_update(name: Option<&str>)` を実装
  - `fav.toml` を読み、対象パッケージの semver req を取得
  - registry から最新の適合バージョンを取得
  - `fav.lock` を更新
- [x] `cmd_remove(name: &str)` を実装
  - `fav.toml` の `dependencies` / `dev_dependencies` から削除
  - `fav.lock` から削除
- [x] `cmd_login()` を実装（スタブ: `~/.fav/credentials` にダミートークンを書き込み "Logged in" を表示）
- [x] `cmd_publish` に `dry_run: bool` 引数を追加（`--dry-run` フラグ対応）

### T8: `fav/src/main.rs` — CLI ルーティング追加

- [x] `Some("add")` ブランチを追加: `--dev` フラグとパッケージ名を解析して `cmd_add` を呼ぶ
- [x] `Some("update")` ブランチを追加: オプションのパッケージ名を解析して `cmd_update` を呼ぶ
- [x] `Some("remove")` ブランチを追加: パッケージ名を解析して `cmd_remove` を呼ぶ
- [x] `Some("login")` ブランチを追加: `cmd_login` を呼ぶ
- [x] `Some("publish")` ブランチに `--dry-run` フラグ解析を追加

### T9: `fav/src/driver.rs` — `v178000_tests` 追加

- [x] `v177000_tests` の `version_is_17_7_0` テストを削除（v177000_tests モジュールの version check のみ削除）
- [x] `v178000_tests` モジュールを追加

```rust
#[cfg(test)]
mod v178000_tests {
    use super::*;
    use crate::lock::{LockFile, LockedPackage};
    use crate::registry::resolver::{parse_version_req, parse_semver, resolve_best};

    #[test]
    fn version_is_17_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"17.8.0\""), "Cargo.toml should have version 17.8.0");
    }

    #[test]
    fn fav_toml_dependencies_parse() {
        let src = r#"
[package]
name = "my-pipeline"
version = "1.0.0"

[dependencies]
csv = "^2.0.0"
bigquery = "^1.0.0"

[dev-dependencies]
test-fixtures = "^1.0.0"

[registry]
url = "https://registry.favnir.dev"
"#;
        let toml = crate::toml::parse_fav_toml(src).expect("parse");
        assert!(toml.dependencies.iter().any(|d| d.name() == "csv"),
            "csv should be in dependencies");
        assert!(toml.dev_dependencies.iter().any(|d| d.name() == "test-fixtures"),
            "test-fixtures should be in dev_dependencies");
        assert_eq!(toml.registry_url.as_deref(), Some("https://registry.favnir.dev"));
    }

    #[test]
    fn fav_lock_generates() {
        let mut lock = LockFile::default();
        lock.packages.push(LockedPackage {
            name: "csv".to_string(),
            version: "2.1.0".to_string(),
            resolved_path: "registry:https://registry.favnir.dev".to_string(),
            checksum: Some("sha256:abc123".to_string()),
            source: Some("registry:https://registry.favnir.dev".to_string()),
        });
        let toml_str = lock.to_toml();
        assert!(toml_str.contains("checksum"), "lock should contain checksum");
        assert!(toml_str.contains("source"), "lock should contain source");
        assert!(toml_str.contains("csv"), "lock should contain package name");
    }

    #[test]
    fn semver_caret_resolve() {
        let req = parse_version_req("^2.0.0").expect("parse req");
        let available = vec![
            parse_semver("1.9.9").unwrap(),
            parse_semver("2.0.0").unwrap(),
            parse_semver("2.1.0").unwrap(),
            parse_semver("3.0.0").unwrap(),
        ];
        let best = resolve_best(&req, &available).expect("resolve");
        assert_eq!(best.major, 2);
        assert_eq!(best.minor, 1);
        assert_eq!(best.patch, 0);
    }

    #[test]
    fn cmd_add_updates_toml() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let toml_path = dir.path().join("fav.toml");
        fs::write(&toml_path, "[package]\nname = \"test\"\nversion = \"1.0.0\"\n").unwrap();

        unsafe { std::env::set_var("REGISTRY_MOCK", "1") };
        let result = cmd_add_to_file(toml_path.to_str().unwrap(), "csv", None, false);
        unsafe { std::env::remove_var("REGISTRY_MOCK") };

        assert!(result.is_ok(), "cmd_add should succeed: {:?}", result);
        let contents = fs::read_to_string(&toml_path).unwrap();
        assert!(contents.contains("csv"), "fav.toml should contain csv after add");
    }
}
```

### T10: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `17.7.0` → `17.8.0` に更新
- [x] `cargo build` で `Cargo.lock` 更新

### T11: ドキュメント

- [x] `site/content/docs/packages/getting-started.mdx` を新規作成
  - `fav add` / `fav update` / `fav remove` の使い方
  - `fav.toml` `[dependencies]` / `[dev-dependencies]` の書き方
  - semver バージョン指定の形式（`^` / `~` / `=` / `*`）
  - `fav.lock` の役割（自動生成、git commit 推奨）

---

## テスト（v178000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_8_0` | Cargo.toml に "17.8.0" が含まれる |
| `fav_toml_dependencies_parse` | `[dependencies]` / `[dev-dependencies]` / `[registry]` が解析される |
| `fav_lock_generates` | `fav.lock` に `checksum` / `source` フィールドが含まれる |
| `semver_caret_resolve` | `^2.0.0` が `2.1.0` を選択し `1.9.9` / `3.0.0` を除外する |
| `cmd_add_updates_toml` | `cmd_add` が `fav.toml` の `[dependencies]` を更新する |

---

## 完了条件チェックリスト

- [x] `FavToml` が `dev_dependencies` / `registry_url` フィールドを持つ
- [x] `LockedPackage` が `checksum` / `source` フィールドを持つ
- [x] `resolver.rs` の `^` / `~` / `=` / `*` semver 解決が正しく動作する
- [x] `client.rs` の `fetch_package` が `REGISTRY_MOCK=1` でダミーレスポンスを返す
- [x] `cmd_add` が `fav.toml` と `fav.lock` を更新する
- [x] `cmd_update` / `cmd_remove` が動作する
- [x] `fav publish --dry-run` が公開内容を表示して終了する
- [x] `cargo test v178000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし

---

## 優先度

T1（toml.rs）
→ T2（lock.rs）
→ T3（resolver.rs）  ← T1/T2 と並列可
→ T4（client.rs）    ← T3 と並列可
→ T5（mod.rs）       ← T3/T4 完了後
→ T6（lib.rs）       ← T5 完了後
→ T7（driver.rs コマンド）← T1〜T6 完了後
→ T8（main.rs ルーティング）← T7 完了後
→ T9（v178000_tests）← T1〜T8 完了後
→ T10（バージョン更新）
→ T11（ドキュメント）

---

## 補足: `DependencySpec::name()` メソッド

テストコード内で `d.name()` を呼ぶため、`DependencySpec` に `name(&self) -> &str` メソッドを追加する必要がある:

```rust
impl DependencySpec {
    pub fn name(&self) -> &str {
        match self {
            DependencySpec::Path { name, .. } => name,
            DependencySpec::Registry { name, .. } => name,
            DependencySpec::Semver { name, .. } => name,
        }
    }
}
```

## 補足: `cmd_add_to_file` テストヘルパー

`v178000_tests` の `cmd_add_updates_toml` では `cmd_add_to_file(path, name, version, dev)` というテスト用ヘルパーを使う。
`driver.rs` に追加するか、あるいは `cmd_add` の引数にファイルパスを渡せるよう設計する。
簡略化として、`cmd_add` が `fav.toml` を現在のディレクトリから探す場合は `std::env::set_current_dir` で tempdir に移動してから呼ぶ。
