# Favnir v4.12.0 タスクリスト — Rune Registry

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.12.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.12.0` に更新

---

## Phase 1: `fav.toml` — `[dependencies]` セクション

- [x] `fav/src/toml.rs` に `DependencySpec::Semver { name, version }` バリアントを追加
  - 注: 計画では `RuneDepSpec` 構造体の新規追加を予定していたが、既存の `DependencySpec` 列挙型に `Semver` バリアントを追加する形で実装（依存型の一貫性を維持）
- [x] `DependencySpec::name()` に `Semver` アームを追加
- [x] `parse_dep_line` を更新し文字列形式（`csv = "^1.0.0"`）を先に判定・`Semver` として返す
- [x] `FavToml` に `description: Option<String>` フィールドを追加
- [x] `FavToml` に `authors: Vec<String>` フィールドを追加
- [x] `FavToml` に `license: Option<String>` フィールドを追加
- [x] `[rune]` セクションの `parse_fav_toml` に `description` / `authors` / `license` パースを追加
- [x] `checker.rs` × 2 の `FavToml { ... }` リテラルに `description: None, authors: vec![], license: None` を追加
- [x] `resolver.rs` × 2 の `FavToml { ... }` リテラルに同上を追加
- [x] `driver.rs` × 1 の `FavToml { ... }` リテラルに同上を追加

---

## Phase 2: `fav/src/registry/mod.rs` — Registry コアロジック

- [x] `fav/src/registry/mod.rs` を新規作成
- [x] `registry_root() -> PathBuf` 関数（`HOME` / `USERPROFILE` 環境変数からパス生成）
- [x] `Registry { root: PathBuf }` 構造体を実装
  - [x] `Registry::new()` — `registry_root()` を使用
  - [x] `Registry::with_root(root: PathBuf)` — テスト用コンストラクタ
- [x] `PackageMeta` 構造体（`name`, `version`, `description`, `author`, `license`, `published`, `files`）
- [x] `PackageEntry` 構造体（`name`, `versions: Vec<String>`）
- [x] semver ヘルパー群を実装
  - [x] `parse_semver(v: &str) -> (u32, u32, u32)`
  - [x] `version_ge(a: &str, b: &str) -> bool`
  - [x] `semver_cmp(a: &str, b: &str) -> Ordering`
  - [x] `semver_compatible(version: &str, base: &str) -> bool`（`^` 制約の判定、0.x はマイナー固定）
- [x] `Registry::installed_versions(name) -> Vec<String>`（降順ソート済み）
- [x] `Registry::resolve_version(name, constraint) -> Option<String>`（`*` / `^x.y.z` / exact）
- [x] `Registry::list() -> Vec<PackageEntry>`（全パッケージを名前順で返す）
- [x] `Registry::search(query) -> Vec<PackageEntry>`（名前フィルタ）
- [x] `Registry::info(name) -> Option<PackageMeta>`（最新バージョンのメタデータ）
- [x] `Registry::publish(meta, rune_files) -> Result<(), String>`（`~/.fav/registry/name/version/` に展開）
- [x] `Registry::install(name, version, dest) -> Result<(), String>`（`./runes/name/` にコピー）
- [x] `Registry::rune_path(name) -> Option<PathBuf>`（最新バージョンの rune ディレクトリ）
- [x] `copy_dir_all(src, dst) -> std::io::Result<()>` ヘルパー（再帰的ディレクトリコピー）
- [x] `collect_rune_files(runes_dir) -> Vec<String>` ヘルパー（ファイル名一覧）
- [x] `collect_fav_files_in(dir) -> Vec<(String, Vec<u8>)>` pub 関数（walkdir で `.fav` を再帰収集）
- [x] `format_pkg_toml(meta) -> String` ヘルパー（`fav.pkg.toml` 形式でシリアライズ）
- [x] `parse_pkg_toml(content, name_fallback, version_fallback) -> PackageMeta` ヘルパー
- [x] `fav/src/main.rs` に `mod registry;` を追加
- [x] `#[cfg(test)] mod tests` — ユニットテスト 8 件
  - [x] `registry_resolve_exact_version` — 完全一致バージョン解決・非存在バージョンは None
  - [x] `registry_resolve_caret_major` — `^1.0.0` → 最新 1.x を選択（2.0.0 は除外）
  - [x] `registry_resolve_caret_minor` — `^0.3.0` → 最新 0.3.x を選択（0.4.0 は除外）
  - [x] `registry_resolve_wildcard` — `*` → 最新バージョンを選択
  - [x] `registry_resolve_not_found` — 存在しない名前 → `None`
  - [x] `registry_publish_creates_files` — tempdir に publish → ファイルが存在する
  - [x] `registry_install_copies_rune` — publish → install → `./runes/` にファイルが存在する
  - [x] `registry_list_returns_all` — 2 pkg を publish → `list()` が 2 件を返す

---

## Phase 3: `resolver.rs` — レジストリフォールバック

- [x] `fav/src/middle/resolver.rs` の `resolve_rune_import_file` を確認
- [x] `./runes/<name>/` が存在しない場合に `Registry::new().rune_path(name)` を呼ぶフォールバックを追加
- [x] フォールバック追加後も既存テスト（922 件）が全て pass することを確認

---

## Phase 4: `driver.rs` — `cmd_publish` / `cmd_install` / `cmd_registry`

### `cmd_install`

- [x] `cmd_install(pkg_name_arg: Option<&str>, force: bool)` を実装（旧 `cmd_install()` を置き換え）
  - [x] 引数なし時は `fav.toml [dependencies]` の全件をインストール
  - [x] 引数あり時はその 1 件のみ（`fav.toml` にない場合はエラー）
  - [x] `DependencySpec::Semver` → `Registry::resolve_version` でバージョン解決 → `Registry::install`
  - [x] `./runes/<name>/` が存在する場合はスキップ（`--force` で上書き）
  - [x] `DependencySpec::Path` / `Registry` → 既存の lockfile 処理を維持
  - [x] インストール件数を表示

### `cmd_publish`

- [x] `cmd_publish(name_override, version_override, dry_run, force)` を実装（旧 `cmd_publish()` を置き換え）
  - [x] `fav.toml` から `name` / `version` / `description` / `authors` / `license` を取得
  - [x] `collect_fav_files_in(&root.join("runes"))` で rune ファイル一覧を収集・表示
  - [x] `[publish] Archive:` / `[publish] Registry:` を表示
  - [x] `dry_run = true` 時は `Done (dry run — no changes made)` を表示して終了
  - [x] `force = false` かつ既存バージョンの場合はエラー終了
  - [x] `Registry::publish` を呼びファイルを展開・完了メッセージを表示

### `cmd_registry`

- [x] `cmd_registry(subcommand: Option<&str>, args: &[String])` を実装（新規）
  - [x] `list` / 引数なし: `Registry::list()` を整形表示（`{:<16}` 幅揃え）
  - [x] `search <q>`: `Registry::search(q)` の結果を description 付きで表示
  - [x] `info <name>`: `Registry::info(name)` のメタデータを表示（versions, files 含む）
  - [x] 不明サブコマンド時は usage を表示してエラー終了

---

## Phase 5: CLI 配線 (main.rs)

- [x] `cmd_registry` を `use driver::{...}` インポートリストに追加
- [x] `Some("install")` アームを更新（`--force` フラグ + 位置引数 `pkg_name` パース）
- [x] `Some("publish")` アームを更新（`--name`, `--version`, `--dry-run`, `--force` パース）
- [x] `Some("registry")` アームを新規追加（サブコマンド + 残り引数を転送）
- [x] HELP テキストの `install` / `publish` 説明を更新、`registry` コマンドを追記

---

## Phase 6: 統合テスト (driver.rs)

- [x] `driver::registry_tests` モジュールを追加 — 7 件
  - [x] `publish_dry_run_does_not_panic` — `--dry-run` 時に panic しない（fav.toml + runes/ ありで実行）
  - [x] `install_from_local_registry` — `Registry::with_root` で publish → install → `./runes/<name>/` にファイルが存在する
  - [x] `registry_list_shows_installed` — 2 pkg publish 後 `reg.list()` が 2 件返す
  - [x] `registry_search_filters_by_name` — `reg.search("csv")` が csv・csv_ext を返し email を返さない
  - [x] `registry_info_shows_metadata` — `reg.info("csv")` が name/version/license/files を正しく返す
  - [x] `install_respects_version_constraint` — `^1.0.0` が 1.2.0 を選択し install が成功する
  - [x] `collect_fav_files_in_works` — tempdir に .fav ファイルを作成し収集できることを確認

---

## 完了条件

- [x] `cargo build` が通る
- [x] 既存テスト（922 件）が全て pass
- [x] 新規テスト 15 件が pass（ユニット 8 + 統合 7）
- [x] `fav publish --dry-run` が全ステップを表示する
- [x] `fav install` が `fav.toml [dependencies]` を読んで `./runes/` に展開する
- [x] `fav registry list` / `search` / `info` が正しく動作する
- [x] `import rune "X"` 時に `./runes/X/` がなければレジストリから解決できる
- [x] `^` バージョン制約が正しく動作する（`^1.0.0` → `1.x.x` の最新を選択）

---

## 実装メモ

- **`DependencySpec::Semver` 採用**: 計画では `RuneDepSpec` 新構造体だったが、既存の `DependencySpec` 列挙型に `Semver` バリアントを追加する形を採用。`dependencies: Vec<DependencySpec>` のままで変更なし
- **`collect_fav_files_in`**: `registry/mod.rs` に `pub` 関数として置き、`driver.rs` から `use crate::registry::collect_fav_files_in` でインポート
- **FavToml リテラル 5 箇所**: `driver.rs` ×1、`resolver.rs` ×2、`checker.rs` ×2 に `description: None, authors: vec![], license: None` を追加
- **テスト件数**: ユニット 8 件 (`registry::tests`) + 統合 7 件 (`driver::registry_tests`) = **15 件**、合計 **937 件**
- **`walkdir` / `chrono`**: v4.11.0 で追加済みのため新規 Cargo.toml 変更なし
- **`^0.x.y` の解釈**: `semver_compatible` にてマイナー固定（`^0.3.0` は `0.3.x` のみ）
- **resolver フォールバック**: `resolve_rune_import_file` でローカルパスが存在しない場合のみ `Registry::new().rune_path()` を呼ぶ（既存テストへの影響なし）
