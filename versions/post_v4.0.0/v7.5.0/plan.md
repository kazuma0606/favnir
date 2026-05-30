# Favnir v7.5.0 Plan

Date: 2026-05-28
Theme: Rune 読み込みのセルフホスト化（TOML パーサー + Rune ローダー）

---

## 実装順序

```
Phase A: VM プリミティブ追加（path_join / home_dir / cwd / is_dir）
Phase B: runes/toml/toml.fav — 簡易 TOML パーサー
Phase C: runes/rune_loader/loader.fav — 3 段階 Rune 解決
Phase D: テスト（driver.rs）
Phase E: ドキュメント
Phase F: 最終確認
```

依存関係:
```
A → B（IO プリミティブが toml.fav で必要）
A + B → C（loader が toml.fav + IO プリミティブを使う）
B + C → D（テストが両 Rune を使う）
```

---

## Phase A: VM プリミティブ追加

### A-1: IO.path_join_raw

```rust
"IO.path_join_raw" => {
    let base = vm_string(pop, "IO.path_join_raw")?;
    let seg  = vm_string(pop, "IO.path_join_raw")?;
    let joined = std::path::Path::new(&base).join(&seg)
                   .to_string_lossy().to_string();
    Ok(VMValue::Str(joined))
}
```

checker.rs: `("IO", "path_join_raw") => Some(Type::Str)`

### A-2: IO.home_dir_raw

```rust
"IO.home_dir_raw" => {
    let home = dirs::home_dir()  // または std::env::var("HOME") / "USERPROFILE"
                .map(|p| VMValue::Str(p.to_string_lossy().to_string()));
    Ok(match home {
        Some(v) => VMValue::Variant("some".into(), Some(Box::new(v))),
        None    => VMValue::Variant("none".into(), None),
    })
}
```

checker.rs: `("IO", "home_dir_raw") => Some(Type::Option(Box::new(Type::Str)))`

> `dirs` クレートを使うか、環境変数 `HOME` / `USERPROFILE` の fallback で実装する。
> クレート追加を避けたい場合は `std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))` で代用。

### A-3: IO.cwd_raw

```rust
"IO.cwd_raw" => {
    match std::env::current_dir() {
        Ok(p)  => Ok(VMValue::Str(p.to_string_lossy().to_string())),
        Err(e) => Err(format!("IO.cwd_raw: {e}")),
    }
}
```

checker.rs: `("IO", "cwd_raw") => Some(Type::Str)`

### A-4: IO.is_dir_raw

```rust
"IO.is_dir_raw" => {
    let path = vm_string(pop, "IO.is_dir_raw")?;
    Ok(VMValue::Bool(std::path::Path::new(&path).is_dir()))
}
```

checker.rs: `("IO", "is_dir_raw") => Some(Type::Bool)`

---

## Phase B: runes/toml/toml.fav

### 設計方針

Favnir の制約（bind inside closure 不可、ジェネリクス不可、ミュータブル不可）に対応するため、
再帰関数 + アキュムレータパターンで実装する。

### ファイル構成

```
runes/toml/rune.toml
runes/toml/toml.fav
```

### 型定義

```favnir
type TomlVal = Str(String) | Arr(List<String>)

// パーサーの作業状態
type ParseState = {
    section: String
    doc:     Map<String, Map<String, TomlVal>>
}
```

### 行パーサーの実装

各行を分類する関数:
```favnir
fn classify_line(line: String) -> String {
    bind trimmed <- String.trim(line)
    if String.length(trimmed) == 0 { "empty" }
    else { if String.starts_with(trimmed, "#") { "comment" }
    else { if String.starts_with(trimmed, "[") { "section" }
    else { "keyval" } } }
}
```

セクション名の抽出（`[rune]` → `"rune"`）:
```favnir
fn extract_section(line: String) -> String {
    bind trimmed <- String.trim(line)
    bind inner   <- String.slice(trimmed, 1, String.length(trimmed) - 1)
    String.trim(inner)
}
```

key-value 行のパース（`key = "value"` or `key = ["a","b"]`）:
```favnir
fn parse_keyval(line: String) -> Option<KVEntry>
// KVEntry = { key: String, val: TomlVal }
```

### 主要関数

```favnir
fn parse_lines(lines: List<String>, state: ParseState) -> ParseState

public fn parse(src: String) -> Result<TomlDoc, String>

public fn get_str(doc: TomlDoc, section: String, key: String) -> Option<String>

public fn get_arr(doc: TomlDoc, section: String, key: String) -> List<String>

public fn read_rune_meta(path: String) -> Result<RuneMeta, String> !IO
```

### ハマりポイント

- **配列パース**: `["Io", "DbRead"]` の解析は `String.trim` + `String.split(inner, ",")` で処理
- **引用符除去**: `"csv"` → `csv` は `String.slice(s, 1, len - 1)` で対応
- **bind inside closure 不可**: `parse_lines` は fold の代わりに List.fold を使うが、
  closure 内でのセクション更新は `{section: new_sec, doc: new_doc}` というレコードを返す形にする

---

## Phase C: runes/rune_loader/loader.fav

### ファイル構成

```
runes/rune_loader/rune.toml
runes/rune_loader/loader.fav
```

### 3 段階解決の実装

```favnir
public fn resolve(name: String, project_root: String) -> ResolveResult !IO {
    bind rune_modules_path <- IO.path_join_raw(
        IO.path_join_raw(project_root, "rune_modules"), name)
    if IO.is_dir_raw(rune_modules_path) {
        // rune.toml の entry を読むか、デフォルト <name>.fav を使う
        bind entry <- read_entry_file(rune_modules_path, name)
        Found(IO.path_join_raw(rune_modules_path, entry))
    } else {
        bind runes_path <- IO.path_join_raw(
            IO.path_join_raw(project_root, "runes"), name)
        if IO.is_dir_raw(runes_path) {
            bind entry <- read_entry_file(runes_path, name)
            Found(IO.path_join_raw(runes_path, entry))
        } else {
            resolve_from_registry(name)
        }
    }
}
```

### セマバー比較

```favnir
// "1.2.3" → { major: 1, minor: 2, patch: 3 }
type SemVer = { major: Int, minor: Int, patch: Int }

fn parse_semver(v: String) -> Option<SemVer>

// "^1.0" 制約チェック
fn matches_constraint(version: String, constraint: String) -> Bool
```

---

## Phase D: テスト（driver.rs）

### toml_rune_tests

1. `toml_parse_simple_test`
   - 入力: `"[rune]\nname = \"csv\"\n"`
   - 期待: `get_str(doc, "rune", "name") == Some("csv")`

2. `toml_parse_array_test`
   - 入力: `"[rune]\neffects = [\"Io\", \"DbRead\"]\n"`
   - 期待: `get_arr(doc, "rune", "effects")` の length == 2

3. `toml_get_str_missing_test`
   - 存在しないキーへの `get_str` → `None`

### rune_loader_tests

4. `loader_resolve_rune_modules_test`
   - `IO.is_dir_raw` が真を返すパスをモックする（inline で直接パスを渡す）
   - 解決結果が `Found` になることを確認

5. `loader_semver_matches_caret_test`
   - `matches_constraint("1.2.3", "^1.0")` → `true`
   - `matches_constraint("2.0.0", "^1.0")` → `false`

6. `loader_semver_exact_test`
   - `matches_constraint("1.2.3", "1.2.3")` → `true`
   - `matches_constraint("1.2.4", "1.2.3")` → `false`

---

## Phase E: ドキュメント

- `site/content/docs/runes/toml.mdx`
- `site/content/docs/runes/rune-loader.mdx`

---

## 注意点

### String.slice の仕様

`String.slice(s, start, end)` は `s[start..end]`（end は exclusive）。

### 引用符除去

`"csv"` → `csv` の変換:
```favnir
fn strip_quotes(s: String) -> String {
    bind trimmed <- String.trim(s)
    if String.starts_with(trimmed, "\"") {
        String.slice(trimmed, 1, String.length(trimmed) - 1)
    } else {
        trimmed
    }
}
```

### dirs クレート vs 環境変数

`IO.home_dir_raw` の実装方針:
- `dirs` クレートを追加するよりも `Env.get_raw("HOME")` で代用できる（`!IO` ではなく `!Env`）
- または `IO.home_dir_raw` を以下で実装:
  ```rust
  std::env::var("HOME")
      .or_else(|_| std::env::var("USERPROFILE"))
      .map(VMValue::Str)
  ```
  → `Option<String>` として返す

### bind inside closure 対策

`toml.fav` の `parse_lines` 関数では `List.fold` の closure 内に `bind` が書けない。
代わりに closure 内の処理を別関数 `process_line(state, line)` に切り出して呼ぶ。
