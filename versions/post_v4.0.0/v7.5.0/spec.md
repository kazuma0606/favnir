# Favnir v7.5.0 Spec

Date: 2026-05-28
Theme: Rune 読み込みのセルフホスト化（TOML パーサー + Rune ローダー）

---

## 概要

v7.5.0 では、`fav rune add` / import 解決ロジックを Rust から Favnir に移す。
具体的には以下の 2 つの Rune を新規作成し、v7.6.0（CLI セルフホスト）で使えるようにする。

```
runes/toml/toml.fav          — rune.toml の簡易パーサー
runes/rune_loader/loader.fav — rune_modules/ → runes/ → ~/.fav/registry/ の順に解決
```

Rust 側の `Resolver::resolve_rune_import_file`（resolver.rs）と
`Registry` クラス（registry/mod.rs）が現在持っているロジックを Favnir 層で再現する。
v7.5.0 では Favnir 実装を追加するのみ（Rust 側は削除しない）。
v7.6.0 の CLI セルフホストで Favnir ローダーが実際に使われるようになる。

---

## Phase A — VM プリミティブ追加

### 不足しているプリミティブ

現在 vm.rs にある操作:
- `IO.file_exists_raw(path) -> Bool` ✓
- `IO.file_stat_raw(path) -> Map<String,String>` ✓ (exists / is_dir / size)
- `IO.list_dir_raw(path) -> Result<List<String>,String>` ✓
- `IO.read_file_raw(path) -> Result<String,String>` ✓
- `String.split(s, delim) -> List<String>` ✓
- `String.lines(s) -> List<String>` ✓
- `String.trim(s) -> String` ✓
- `String.starts_with(s, prefix) -> Bool` ✓
- `String.ends_with(s, suffix) -> Bool` ✓
- `String.slice(s, start, end) -> String` ✓
- `String.index_of(s, sub) -> Int` ✓（-1 if not found）
- `Env.get_raw(key) -> Option<String>` ✓

追加が必要なもの:

| プリミティブ | シグネチャ | 用途 |
|-------------|-----------|------|
| `IO.path_join_raw` | `String, String -> String` | OS 正しいパス結合 |
| `IO.home_dir_raw` | `() -> Option<String>` | `~/.fav/registry/` のパス取得 |
| `IO.cwd_raw` | `() -> String` | プロジェクトルート検出 |
| `IO.is_dir_raw` | `String -> Bool` | ディレクトリ判定（`file_stat_raw` の簡略版） |

> **注意**: `path_join` は Favnir で実装可能（`String.ends_with(base, "/")` で分岐）だが、
> Windows の `\` 区切りに対応するため vm.rs の `std::path::Path::join` を使う方が安全。

---

## Phase B — runes/toml/toml.fav

### 対象フォーマット

`rune.toml` のミニサブセットのみ対応する。フル TOML 実装は不要。

```toml
[rune]
name    = "csv"
version = "1.2.3"
entry   = "csv.fav"
effects = ["Io", "DbRead"]

[dependencies]
stdlib = "^0.1"
```

### パース戦略

1. `String.lines(src)` で行分割
2. 行ごとに分類:
   - 空行・コメント行（`#` 始まり）→ スキップ
   - `[section]` → 現在セクション更新
   - `key = "value"` → single string
   - `key = ["a", "b"]` → string array
3. 結果を `TomlDoc` レコード（`sections: Map<String, Map<String, TomlVal>>`）として返す

### 型定義

```favnir
// value は文字列または文字列リストのどちらか
type TomlVal = Str(String) | Arr(List<String>)

// セクション: Map<key, value>
type TomlSection = Map<String, TomlVal>

// ドキュメント全体: Map<section_name, TomlSection>
type TomlDoc = Map<String, TomlSection>
```

### 公開 API

```favnir
// toml ソース文字列をパースして TomlDoc を返す
public fn parse(src: String) -> Result<TomlDoc, String>

// セクション内の文字列値を取得
public fn get_str(doc: TomlDoc, section: String, key: String) -> Option<String>

// セクション内の配列値を取得
public fn get_arr(doc: TomlDoc, section: String, key: String) -> List<String>
```

### rune.toml 専用ヘルパー

```favnir
// rune.toml を読んでメタ情報を構造体で返す
type RuneMeta = {
    name:    String
    version: String
    entry:   String
    effects: List<String>
}

public fn read_rune_meta(path: String) -> Result<RuneMeta, String> !IO
```

---

## Phase C — runes/rune_loader/loader.fav

### 解決順序（Rust 版と同じ）

```
1. <project_root>/rune_modules/<name>/   — fav rune add でインストール済み
2. <project_root>/runes/<name>/         — プロジェクトローカル定義
3. ~/.fav/registry/<name>/<version>/    — ローカルレジストリ
```

### 型定義

```favnir
type ResolveResult = Found(String) | NotFound | Error(String)
```

### 公開 API

```favnir
// Rune 名からエントリーファイルパスを解決する
public fn resolve(name: String, project_root: String) -> ResolveResult !IO

// インストール済みバージョン一覧を返す（新しい順）
public fn installed_versions(name: String) -> List<String> !IO

// バージョン制約を満たす最新版を返す
// constraint: "^1.0" | ">=1.0,<2.0" | "1.2.3"（完全一致）
public fn resolve_version(name: String, constraint: String) -> Option<String> !IO
```

### セマバー比較（簡易）

`"^1.0.0"` 形式の制約のみ対応:
- `^X.Y.Z` → `>= X.Y.Z && < (X+1).0.0`
- exact: `X.Y.Z` → `== X.Y.Z`

---

## Phase D — テスト（driver.rs）

### toml_rune_tests（3 件）

1. `toml_parse_simple_test` — `name = "csv"` が `Str("csv")` としてパースされる
2. `toml_parse_array_test` — `effects = ["Io", "DbRead"]` が `Arr(["Io","DbRead"])` になる
3. `toml_get_str_test` — `get_str(doc, "rune", "name")` が `Some("csv")` を返す

### rune_loader_tests（3 件）

1. `loader_resolve_from_rune_modules_test` — `rune_modules/csv/` がある → そのパスを返す
2. `loader_resolve_from_runes_dir_test` — `rune_modules/` なし → `runes/csv/csv.fav` にフォールバック
3. `loader_installed_versions_test` — `~/.fav/registry/csv/1.0.0/` → `["1.0.0"]` を返す

---

## Phase E — ドキュメント

- `site/content/docs/runes/toml.mdx` — mini TOML パーサー リファレンス
- `site/content/docs/runes/rune-loader.mdx` — Rune 解決ロジックと上書き優先度

---

## 完了条件

- `runes/toml/toml.fav` が `fav check` を通る
- `runes/rune_loader/loader.fav` が `fav check` を通る
- `IO.path_join_raw` / `IO.home_dir_raw` / `IO.cwd_raw` / `IO.is_dir_raw` が vm.rs に追加済み
- 統合テスト 6 件以上追加
- 既存テスト 1081 件が全件通る（目標: 1090+ tests）
- ドキュメント 2 ページ追加
