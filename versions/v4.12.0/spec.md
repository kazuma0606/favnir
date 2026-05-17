# Favnir v4.12.0 仕様書 — Rune Registry

作成日: 2026-05-17

---

## 概要

Rune を発行・配布・インストールするための **ローカル Rune Registry** を実装する。
`fav publish` でプロジェクトの Rune をローカルレジストリに登録し、`fav install` で依存 Rune を取得できる。
ホスト型レジストリ（favnir.dev/registry 等）は v5.x で対応する。v4.12.0 はローカルファイルシステムを基盤とする。

**主な追加機能:**
- `fav.toml [dependencies]` — 外部 Rune 依存の宣言（名前 + バージョン制約）
- `~/.fav/registry/` — ローカルレジストリディレクトリ
- `.fav.pkg` — Rune パッケージ形式（gzip tar アーカイブ）
- `fav publish` — カレントプロジェクトの Rune をローカルレジストリに登録
- `fav install` — `fav.toml [dependencies]` を読んで Rune を `./runes/` に展開
- `fav registry list` — インストール済み Rune を一覧表示
- `fav registry search <query>` — 名前でフィルタ
- `fav registry info <name>` — パッケージのメタデータを表示
- Resolver 統合 — `import rune "X"` 時にローカル `./runes/X/` がなければレジストリを参照

---

## `fav.toml [dependencies]` セクション

```toml
[package]
name    = "myapp"
version = "0.1.0"

[dependencies]
csv    = "1.0.0"
email  = "^0.3.0"
http   = { version = "2.1.0", registry = "local" }
```

### バージョン制約

| 表記 | 意味 |
|------|------|
| `"1.0.0"` | 完全一致 |
| `"^1.0.0"` | `>=1.0.0 <2.0.0`（マイナー互換） |
| `"^0.3.0"` | `>=0.3.0 <0.4.0`（0.x はパッチ互換） |
| `"*"` | 最新バージョン |

### `RuneDepSpec` 構造体（toml.rs）

```rust
#[derive(Debug, Clone)]
pub struct RuneDepSpec {
    pub version:  String,            // "1.0.0" / "^1.0.0" / "*"
    pub registry: Option<String>,    // None = "local" (デフォルト)
}
```

`fav.toml` では文字列形式と `{ version, registry }` テーブル形式の両方を受け付ける。

---

## ローカルレジストリ構造

```
~/.fav/
└── registry/
    ├── csv/
    │   ├── 1.0.0/
    │   │   ├── fav.pkg.toml      # パッケージメタデータ
    │   │   └── runes/
    │   │       └── csv/
    │   │           ├── parse.fav
    │   │           ├── write.fav
    │   │           └── csv.fav   # バレル
    │   └── 1.1.0/
    │       └── ...
    └── email/
        └── 0.3.1/
            └── ...
```

### `fav.pkg.toml`（パッケージメタデータ）

```toml
name        = "csv"
version     = "1.0.0"
description = "CSV read/write utilities for Favnir"
author      = "favnir-team"
license     = "MIT"
published   = "2026-05-17T00:00:00Z"
```

---

## `.fav.pkg` アーカイブ形式

`fav publish` が生成するアーカイブ。gzip 圧縮された tar ファイル。

```
<name>-<version>.fav.pkg  (= .tar.gz)
├── fav.pkg.toml
└── runes/
    └── <name>/
        └── *.fav
```

`fav publish` は `/tmp/<name>-<version>.fav.pkg` を生成したあと、
自動的にローカルレジストリ（`~/.fav/registry/<name>/<version>/`）に展開する。

---

## `fav publish` コマンド

```
fav publish [--name <name>] [--version <version>] [--dry-run]
```

### 動作

1. カレントディレクトリの `fav.toml` を読む（`name` / `version` / `description` 等）
2. `runes/` 以下の `.fav` ファイルをすべて収集
3. `fav.pkg.toml` を生成
4. `.fav.pkg`（gzip tar）を `/tmp/<name>-<version>.fav.pkg` に書き出す
5. `~/.fav/registry/<name>/<version>/` に展開
6. 既存バージョンがある場合は上書き確認（`--force` で強制上書き）

### `--dry-run` 出力例

```
[publish] Package: csv v1.0.0
[publish] Files:
  runes/csv/parse.fav
  runes/csv/write.fav
  runes/csv/csv.fav
[publish] Archive: /tmp/csv-1.0.0.fav.pkg (DRY RUN)
[publish] Registry: ~/.fav/registry/csv/1.0.0/ (DRY RUN)
[publish] Done (dry run — no changes made)
```

### 実行時出力例

```
[publish] Package: csv v1.0.0
[publish] Files: 3 .fav files
[publish] Archive: /tmp/csv-1.0.0.fav.pkg (1.2 KB)
[publish] Registry: ~/.fav/registry/csv/1.0.0/
[publish] Done — csv@1.0.0 published to local registry
```

---

## `fav install` コマンド

```
fav install [<name>[@<version>]]
```

引数なしの場合は `fav.toml [dependencies]` をすべてインストールする。

### 動作

1. `fav.toml` を読んで `[dependencies]` を取得（引数指定時はその 1 件のみ）
2. 各依存について `~/.fav/registry/<name>/` から条件を満たす最新バージョンを選択
3. `./runes/<name>/` にファイルをコピー（既存の場合はスキップ、`--force` で上書き）
4. インストール結果を表示

### バージョン解決

```rust
fn resolve_version(constraint: &str, available: &[String]) -> Option<String> {
    // available は semver ソート済み降順
    match constraint {
        "*" => available.first().cloned(),
        v if v.starts_with('^') => {
            let base = &v[1..];  // "^1.2.0" → "1.2.0"
            available.iter().find(|av| semver_compatible(av, base)).cloned()
        }
        exact => available.iter().find(|av| *av == exact).cloned(),
    }
}
```

### 出力例

```
[install] Reading fav.toml...
[install] Resolving csv@^1.0.0 → 1.0.0
[install] Copying csv@1.0.0 → ./runes/csv/
[install] Done — 1 package installed
```

---

## `fav registry` サブコマンド

### `fav registry list`

インストール済み（`~/.fav/registry/` 内）の全パッケージを一覧表示する。

```
csv          1.0.0, 1.1.0
email        0.3.1
http         2.1.0
```

### `fav registry search <query>`

パッケージ名に `<query>` を含むものを表示する。

```
$ fav registry search csv
csv          1.1.0    CSV read/write utilities for Favnir
```

### `fav registry info <name>`

パッケージの最新バージョンのメタデータを表示する。

```
$ fav registry info csv
Name:        csv
Versions:    1.0.0, 1.1.0 (latest)
Description: CSV read/write utilities for Favnir
Author:      favnir-team
License:     MIT
Published:   2026-05-17
Files:
  runes/csv/parse.fav
  runes/csv/write.fav
  runes/csv/csv.fav
```

---

## Resolver 統合

`import rune "X"` の解決順序を変更する：

1. `./runes/X/` — プロジェクトローカル（既存の挙動）
2. `~/.fav/registry/X/<resolved-version>/runes/X/` — レジストリ（新規）

`resolver.rs` の `resolve_rune_path` に第 2 段フォールバックを追加する。

```rust
fn resolve_rune_path(name: &str, project_root: &Path) -> Option<PathBuf> {
    // 1. ローカル
    let local = project_root.join("runes").join(name);
    if local.exists() {
        return Some(local);
    }
    // 2. レジストリ
    if let Some(home) = dirs_path() {
        let reg = home.join(".fav").join("registry").join(name);
        if reg.exists() {
            // 最新バージョンを選択
            if let Some(ver) = latest_installed_version(&reg) {
                let rune_dir = reg.join(ver).join("runes").join(name);
                if rune_dir.exists() {
                    return Some(rune_dir);
                }
            }
        }
    }
    None
}
```

---

## `fav/src/registry/mod.rs`

新規ファイル。Registry 操作の中核ロジックをすべて実装する。

```rust
pub struct Registry {
    pub root: PathBuf,   // ~/.fav/registry/
}

impl Registry {
    pub fn new() -> Self { ... }
    pub fn list(&self) -> Vec<PackageEntry> { ... }
    pub fn search(&self, query: &str) -> Vec<PackageEntry> { ... }
    pub fn info(&self, name: &str) -> Option<PackageMeta> { ... }
    pub fn publish(&self, pkg: &PackageMeta, rune_files: &[(String, Vec<u8>)]) -> Result<(), String> { ... }
    pub fn install(&self, name: &str, version: &str, dest: &Path) -> Result<(), String> { ... }
    pub fn resolve_version(&self, name: &str, constraint: &str) -> Option<String> { ... }
    pub fn installed_versions(&self, name: &str) -> Vec<String> { ... }
}

pub struct PackageEntry {
    pub name:     String,
    pub versions: Vec<String>,
}

pub struct PackageMeta {
    pub name:        String,
    pub version:     String,
    pub description: String,
    pub author:      String,
    pub license:     String,
    pub published:   String,
}
```

`~/.fav/` のホームディレクトリは `std::env::var("HOME")` または Windows では `USERPROFILE` から取得する（`dirs` クレートは追加しない）。

---

## テスト方針

### ユニットテスト（`fav/src/registry/mod.rs` 内 — 目標 8 件）

| テスト | 内容 |
|--------|------|
| `registry_resolve_exact_version` | 完全一致バージョン解決 |
| `registry_resolve_caret_major` | `^1.0.0` → 最新 1.x を選択 |
| `registry_resolve_caret_minor` | `^0.3.0` → 最新 0.3.x を選択 |
| `registry_resolve_wildcard` | `*` → 最新バージョンを選択 |
| `registry_resolve_not_found` | 存在しない name → `None` |
| `registry_publish_creates_files` | tempdir に publish → ファイル存在確認 |
| `registry_install_copies_rune` | tempdir から install → `./runes/` にコピー |
| `registry_list_returns_all` | tempdir に 2 pkg → `list()` が 2 件返す |

### 統合テスト（`driver.rs` — 目標 7 件）

| テスト | 内容 |
|--------|------|
| `publish_dry_run_prints_steps` | `fav publish --dry-run` が全ステップを表示 |
| `install_from_local_registry` | publish 後に install で `./runes/` に展開される |
| `registry_list_shows_installed` | `fav registry list` が登録済みを表示 |
| `registry_search_filters_by_name` | `fav registry search` が名前でフィルタ |
| `registry_info_shows_metadata` | `fav registry info` がメタデータを表示 |
| `resolver_falls_back_to_registry` | `import rune` がレジストリにフォールバック |
| `install_respects_version_constraint` | `^1.0.0` 制約が正しく解決される |

---

## 既知の制約

- v4.12.0 はローカルレジストリのみ（`registry = "local"` が唯一のソース）
- リモート（favnir.dev/registry、GitHub 等）は v5.x で対応
- パッケージ署名・整合性検証（SHA256 チェックサム）は未対応
- `fav publish` は公開鍵認証なし（ローカル専用のため）
- Semver の `>=`・`<=`・`~`（チルダ）は未対応（`exact`, `^`, `*` のみ）
- `fav.toml` の `[dependencies]` に宣言しても型チェッカーは Rune の型情報を自動取得しない（`fav install` で `./runes/` に展開後は既存の Rune import が機能する）
