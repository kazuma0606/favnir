# v5.4.0 Spec — `rune_modules/` Import Resolution

## Overview

v5.3.0 で `rune install csv` → `./rune_modules/csv/` へのインストールが完成した。
v5.4.0 では **`import csv` が `rune_modules/` を参照する** ようにし、エコシステムを完結させる。

あわせて全 15 標準 Rune をレジストリに公開し、サイトのドキュメントを更新する。

---

## Feature 1: `rune_modules/` Import Resolution

### 現状

`import csv` の解決順序（現在）:

1. `fav.toml` の `[runes] path` で指定されたディレクトリ（デフォルト `./runes/`）
2. `~/.fav/registry/` (local registry fallback)
3. `fav.toml` なし（standalone）→ **rune import は一切解決されない**

### 変更後

`import csv` の解決順序（v5.4.0）:

1. **`./rune_modules/csv/`** — `rune install` でインストールされた場所（新規・最優先）
   - `rune_modules/<name>/rune.toml` の `entry` フィールドで entry ファイル名を決定
   - entry がない / rune.toml がない場合は `<name>.fav` にフォールバック
2. `fav.toml` の `[runes] path` / `./runes/` — 従来の場所（後方互換）
3. `~/.fav/registry/` — local registry fallback（従来通り）

### 対象ファイル

**`fav/src/middle/resolver.rs`** — `resolve_rune_import_file()`:
- `root` の有無に関わらず `rune_modules/` をチェック
- `root` がある場合: `{root}/rune_modules/<name>/`
- `root` がない場合: CWD の `./rune_modules/<name>/`（standalone 対応）

**`fav/src/driver.rs`** — `load_all_items()` の `ImportDecl { is_rune: true }` 分岐:
- `toml.runes_dir(root)` の前に `rune_modules/<name>/` を確認
- `(toml, root)` が None の場合（standalone）: `source_file_dir/rune_modules/<name>/` を確認

### rune.toml entry 読み取り

```
rune_modules/<name>/rune.toml
  [rune]
  entry = "csv.fav"
```

→ `rune_modules/<name>/csv.fav` を使用

`rune.toml` が存在しない or `entry` が空の場合: `rune_modules/<name>/<name>.fav`

---

## Feature 2: Standalone Script Mode

`fav.toml` のない単一ファイルスクリプトでも `import csv` が動作する。

```
my-script/
  main.fav        # import csv
  rune_modules/
    csv/
      csv.fav
      rune.toml
```

```bash
fav run main.fav  # rune_modules/csv/ から csv を解決
```

解決パス: `source_file_dir/rune_modules/<name>/`（ソースファイルと同じディレクトリの `rune_modules/`）

---

## Feature 3: 全 15 標準 Rune の公開

以下 15 Rune を全て `rune publish` でレジストリに公開する：

| Rune | version |
|---|---|
| auth | 0.1.0 |
| aws | 0.1.0 |
| csv | 0.1.0 |
| db | 0.1.0 |
| duckdb | 0.1.0 |
| env | 0.1.0 |
| gen | 0.1.0 |
| grpc | 0.1.0 |
| http | 0.1.0 |
| incremental | 0.1.0 |
| json | 0.1.0 |
| log | 0.1.0 |
| parquet | 0.1.0 |
| stat | 0.1.0 |
| validate | 0.1.0 |

---

## Feature 4: サイトドキュメント更新

### 新規ページ

**`site/content/docs/rune-cli.mdx`**:
- `rune install` / `uninstall` / `list` / `info` / `search` / `update` / `publish` の説明
- インストール手順（`cp fav rune` または symlink）
- `rune.toml` フォーマット

**`site/content/docs/installation.mdx` 更新**:
- `rune` コマンドの追記

### Rune カタログページ更新

`site/app/runes/page.tsx` — ライブ Registry API からルーン一覧を取得（現在はハードコード）

---

## 完了条件

- `rune install csv && fav run main.fav` (with `import csv`) が正常動作する
- `fav.toml` なしの standalone script でも `import csv` が解決される
- 全 15 Rune が Registry に存在する（`rune search` で確認）
- サイトに `rune` CLI ドキュメントページが存在する
- 965 + 新規テストが全 pass
