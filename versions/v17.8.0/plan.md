# v17.8.0 — 実装計画

## 方針

- v17.8.0 は他のバージョンと独立して実装可能（lexer / AST の変更不要）
- 新規ファイルは `fav/src/registry/` ディレクトリに配置
- registry API への実際の HTTP 通信はテスト時にはスタブ（ダミーレスポンス）で代替
- `semver` crate の追加は不要（自前の軽量実装で対応）

## 実装ステップ

### Step 1: `fav/src/toml.rs` 拡張（`[dev-dependencies]` / `[registry]`）

**変更内容:**

1. `FavToml` 構造体に以下を追加:
   ```rust
   pub dev_dependencies: Vec<DependencySpec>,
   pub registry_url: Option<String>,
   ```
2. `parse_fav_toml` に `[dev-dependencies]` セクション解析を追加
3. `[registry]` の `url` キーを解析して `registry_url` に格納
4. `FavToml::default()` / `FavToml::new()` の初期化も更新

### Step 2: `fav/src/lock.rs` 拡張（`checksum` / `source` フィールド）

**変更内容:**

1. `LockedPackage` 構造体に以下を追加:
   ```rust
   pub checksum: Option<String>,
   pub source: Option<String>,
   ```
2. `LockFile::to_toml` に `checksum` / `source` の書き出しを追加
3. `LockFile::load` に `checksum` / `source` の読み込みを追加

### Step 3: `fav/src/registry/` ディレクトリ新規作成

3 ファイルを作成:

#### `fav/src/registry/mod.rs`

```rust
pub mod client;
pub mod resolver;
```

#### `fav/src/registry/resolver.rs`

- `SemVer { major, minor, patch }` 構造体
- `VersionReq { Caret / Tilde / Exact / Any }` enum
- `parse_semver(s: &str) -> Option<SemVer>`
- `parse_version_req(s: &str) -> Option<VersionReq>`
- `matches_req(req: &VersionReq, v: &SemVer) -> bool`
- `resolve_best(req: &VersionReq, available: &[SemVer]) -> Option<SemVer>`

`^` ルール:
- major > 0: `>=major.minor.patch, <(major+1).0.0`
- major == 0, minor > 0: `>=0.minor.patch, <0.(minor+1).0`
- major == 0, minor == 0: `=0.0.patch`

#### `fav/src/registry/client.rs`

```rust
pub struct RegistryClient { pub base_url: String, pub token: Option<String> }

pub struct PackageInfo {
    pub name: String,
    pub versions: Vec<String>,
    pub latest: String,
}

impl RegistryClient {
    pub fn new(base_url: &str) -> Self
    pub fn fetch_package(&self, name: &str) -> Result<PackageInfo, String>
    pub fn publish(&self, name: &str, version: &str, dry_run: bool) -> Result<(), String>
}
```

`fetch_package` は `ureq` で `GET {base_url}/packages/{name}` を呼ぶ。
テスト時は実際のネットワーク通信を避けるため、`REGISTRY_MOCK=1` 環境変数があればダミーレスポンスを返す。

### Step 4: `fav/src/driver.rs` コマンド追加

#### `cmd_add(name: &str, version_str: Option<&str>, dev: bool)`

1. `fav.toml` を読む（なければ error）
2. パッケージ名から `@` でバージョンを分割（`csv@2.1.0` → name="csv", version="2.1.0"）
3. `RegistryClient::fetch_package` でバージョン一覧取得
4. `version_str` がなければ `latest` を使用
5. semver req 文字列を `"^{version}"` として構築
6. `DependencySpec::Semver { name, version }` を `fav.toml` の `dependencies`（または `dev_dependencies`）に追加
7. `save_fav_toml` で書き出し
8. `fav.lock` の `LockedPackage` に追加して `LockFile::save`

#### `cmd_update(name: Option<&str>)`

1. `fav.toml` を読む
2. 対象パッケージ（`name` が Some なら絞り込み、None なら全て）を取得
3. 各パッケージについて registry から利用可能バージョンを取得
4. `resolve_best` で semver 制約を満たす最新版を選択
5. `fav.lock` を更新

#### `cmd_remove(name: &str)`

1. `fav.toml` の `dependencies` から該当エントリを削除
2. `dev_dependencies` からも削除
3. `fav.lock` から該当 `LockedPackage` を削除
4. 両ファイルを保存

#### `cmd_login()`

1. `~/.fav/credentials` にダミートークンを書き込む（v17.8.0 はスタブ）
2. "Logged in" メッセージを表示

### Step 5: `fav/src/main.rs` CLI オプション追加

`fav add`, `fav update`, `fav remove`, `fav login` のルーティングを追加:

```rust
Some("add") => {
    let mut dev = false;
    let mut pkg_name = None;
    // --dev フラグ + パッケージ名を解析
    cmd_add(name, version, dev);
}
Some("update") => { cmd_update(name); }
Some("remove") => { cmd_remove(name); }
Some("login") => { cmd_login(); }
```

また `publish` に `--dry-run` オプションを追加（既存 `cmd_publish` を拡張）。

### Step 6: `fav/src/lib.rs` に `registry` module を追加

```rust
pub mod registry;
```

### Step 7: `fav/src/driver.rs` — `v178000_tests` 追加

`v177000_tests` の `version_is_17_7_0` テストを削除（v178000 に移行）し、新しい 5 件を追加:

1. `version_is_17_8_0`: Cargo.toml に `"17.8.0"` が含まれる
2. `fav_toml_dependencies_parse`: `[dependencies]` + `[dev-dependencies]` + `[registry]` を持つ TOML 文字列を `parse_fav_toml` で解析し、各フィールドを確認
3. `fav_lock_generates`: `LockFile` に `LockedPackage` を追加し `to_toml()` の出力に `checksum` と `source` が含まれることを確認
4. `semver_caret_resolve`: `parse_version_req("^2.0.0")` が `2.1.0` を選択し `1.9.9` を除外することを確認
5. `cmd_add_updates_toml`: `tempfile` で `fav.toml` を作成し `cmd_add` を呼んで `dependencies` エントリが追加されることを確認（`REGISTRY_MOCK=1` 環境変数使用）

### Step 8: バージョン更新

- `fav/Cargo.toml`: `17.7.0` → `17.8.0`
- `cargo build` で `Cargo.lock` 更新

### Step 9: ドキュメント

`site/content/docs/packages/getting-started.mdx` — `fav add` / `fav update` / `fav remove` の使い方ガイド

---

## 依存関係グラフ

```
Step 1 (toml.rs)
Step 2 (lock.rs)
Step 3 (registry/)       ← Steps 1, 2 に依存
    |
Step 4 (driver.rs)       ← Steps 1, 2, 3 に依存
    |
Step 5 (main.rs)         ← Step 4 に依存
Step 6 (lib.rs)          ← Step 3 に依存
    |
Step 7 (v178000_tests)   ← Steps 1-6 すべて完了後
    |
Step 8 (バージョン更新)
Step 9 (ドキュメント)
```

Steps 1 と 2 は並列実施可能。
Step 3 の `resolver.rs` と `client.rs` も並列実施可能。

---

## テストの `REGISTRY_MOCK` 戦略

`RegistryClient::fetch_package` の先頭に以下を追加:

```rust
if std::env::var("REGISTRY_MOCK").is_ok() {
    return Ok(PackageInfo {
        name: name.to_string(),
        versions: vec!["1.0.0".to_string(), "2.0.0".to_string(), "2.1.0".to_string()],
        latest: "2.1.0".to_string(),
    });
}
```

テスト内で `unsafe { std::env::set_var("REGISTRY_MOCK", "1") }` を設定してから実行。

---

## 注意事項

- `save_fav_toml` は既存の実装がない場合は新規作成が必要。既存 `FavToml` は読み取り専用の場合は書き出し機能を追加する。
- `cmd_publish` は既存実装あり。`--dry-run` フラグを追加するだけ。
- `fav.lock` の `resolved_path` は registry パッケージの場合は空文字列または省略可能とする。
- `LockedPackage` の `resolved_path` フィールドは既存のため、後方互換のため Optional にはしない。registry パッケージは `"registry:{url}"` を入れる。
