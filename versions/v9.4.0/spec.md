# Favnir v9.4.0 仕様書 — json・csv・gen Rune 拡張 + W004 lint ルール

作成日: 2026-06-01

---

## 概要

データエンジニアが日常的に扱う JSON・CSV を型安全に読み書きできる API を Rune に追加する。
合わせて `gen` Rune に UUID 生成を追加し、ID 採番をパイプラインに自然に組み込めるようにする。
v9.3.0 から延期した W004 lint ルールもこのバージョンで実装する。

**対象ファイル**

| ファイル | 変更種別 |
|---|---|
| `runes/json/json.fav` | 追加（`encode` / `decode` / `pretty`）|
| `runes/csv/csv.fav` | 追加（`read<T>` / `write_file<T>`）|
| `runes/gen/primitives.fav` | 追加（`uuid` / `uuid_v7` / `nano_id`）|
| `fav/src/backend/vm.rs` | 追加（`Gen.uuid_raw` / `Gen.uuid_v7_raw` / `Gen.nano_id_raw` / `Json.pretty_raw`）|
| `fav/src/checker.rs` | 追加（gen 新関数のスキーマ）|
| `fav/self/checker.fav` | 追加（gen 新関数の型シグネチャ）|
| `fav/self/compiler.fav` | 追加（W004 lint ルール）|
| `fav/src/driver.rs` | 追加（統合テスト）|

---

## json Rune 拡張（`runes/json/json.fav`）

### 既存 API（変更なし）

```favnir
public fn parse<T>(text: String) -> Result<T, SchemaError>
public fn parse_list<T>(text: String) -> Result<List<T>, SchemaError>
public fn write<T>(value: T) -> String
public fn write_list<T>(rows: List<T>) -> String
```

### 新規追加

```favnir
// parse<T> の短縮エイリアス（Result の Err 型が String）
public fn decode<T>(text: String) -> Result<T, String>

// write<T> の短縮エイリアス
public fn encode<T>(value: T) -> String

// JSON 文字列を整形済み（インデント付き）文字列に変換
public fn pretty(text: String) -> String
```

**設計方針**
- `decode` は `Json.parse_raw` を直接使用（SchemaError を介さない簡易版）
- `encode` は `Json.encode` builtin を呼び出す（vm.rs 既存）
- `pretty` は `Json.pretty_raw` builtin を呼び出す（vm.rs 新規追加）

---

## csv Rune 拡張（`runes/csv/csv.fav`）

### 既存 API（変更なし）

```favnir
public fn parse<T>(text: String) -> Result<List<T>, SchemaError>
public fn parse_positional<T>(text: String) -> Result<List<T>, SchemaError>
public fn write<T>(rows: List<T>) -> String       // → CSV 文字列を返す
public fn parse_with_opts<T>(text: String, opts: CsvOptions) -> Result<List<T>, SchemaError>
```

### 新規追加

```favnir
// ファイルパスから直接読み込み（IO.read_file_raw + parse<T> の合成）
public fn read<T>(path: String) -> Result<List<T>, String> !IO

// CSV 文字列をファイルに書き込む（write<T> + IO.write_file_raw の合成）
public fn write_file<T>(path: String, rows: List<T>) -> Unit !IO
```

**設計方針**
- `read<T>` と `write_file<T>` は Favnir コードで実装（新規 Rust builtin 不要）
- `read<T>`: `IO.read_file_raw(path)` → `parse<T>(text)`
- `write_file<T>`: `write<T>(rows)` → `IO.write_file_raw(path, csv_text)`
- エラー型は `String`（IOエラーと CSV パースエラーを統一）

---

## gen Rune 拡張（`runes/gen/primitives.fav`）

### 既存 API（変更なし）

```favnir
public fn int_val(min: Int, max: Int) -> Int !Random
public fn float_val() -> Float !Random
public fn bool_val() -> Bool !Random
public fn string_val(len: Int) -> String !Random
public fn choice(items: List<String>) -> Option<String> !Random
```

### 新規追加

```favnir
// UUID v4（ランダム）
public fn uuid() -> String !Gen

// UUID v7（タイムスタンプ付き、DB インデックス効率良）
public fn uuid_v7() -> String !Gen

// URL-safe ランダム文字列（n 文字）
public fn nano_id(n: Int) -> String !Gen
```

**vm.rs に追加する builtin**

| builtin 名 | 内容 |
|---|---|
| `Gen.uuid_raw` | `uuid::Uuid::new_v4().to_string()` |
| `Gen.uuid_v7_raw` | `uuid::Uuid::now_v7().to_string()`（uuid crate v1.6+ 対応）|
| `Gen.nano_id_raw(n)` | `rand` crate で URL-safe 文字列を n 文字生成 |

---

## W004 lint ルール（`fav/self/compiler.fav`）

### 定義

`fn` 定義の引数が **4 個以上**の場合に警告する。

> ノート: `TypeExpr` に `TeTuple` バリアントがないため、`stage` の入力型ではなく `fn` のパラメータ数で判定する。

```
W004: fn <name> の引数が <n> 個です。レコード型へのまとめを検討してください
```

### 適用対象

- `IFn(fd)`: `List.length(fd.params) >= 4` → W004

### 実装

```favnir
fn lint_fn_w004(fd: FnDef) -> List<LintWarning>
```

- `List.length(fd.params) >= 4` の場合に `LintWarning { code: "W004" ... }` を返す
- `lint_item` の `IFn` ブランチに追加（`lint_fn` が呼ぶ）

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `json.decode(s)` が JSON 文字列をパースして Record を返す | |
| `json.encode(v)` が任意の値を JSON 文字列にシリアライズする | |
| `json.pretty(s)` が整形済み JSON 文字列を返す | |
| `csv.read<T>(path)` がファイルを読んで型付き List を返す | |
| `csv.write_file<T>(path, rows)` がファイルに書き込む | |
| `gen.uuid()` が UUID v4 文字列を返す（`!Gen` エフェクト）| |
| `gen.uuid_v7()` が UUID v7 文字列を返す（`!Gen` エフェクト）| |
| `gen.nano_id(n)` が n 文字の URL-safe 文字列を返す（`!Gen` エフェクト）| |
| W004 が `fn` 引数 4 個以上で警告を出す | |
| `fav check fav/self/compiler.fav` が self-check を通る | |
| `cargo test bootstrap` が bytecode_A == bytecode_B を維持する | |
| `cargo test` 全件通過（1183 件以上）| |
