# Favnir v0.7.0 仕様書

更新日: 2026-04-30

## 概要

v0.7.0 は **「標準ライブラリ版」** — 実用的なデータ処理が標準ライブラリだけで書けることをテーマとするバージョン。

- **List / String / Map の完全化**: 高階関数（`map`, `filter`, `fold` 等）を含む全関数を eval.rs・vm.rs の両方で動作させる。
- **Option / Result の高階関数**: `map`, `and_then`, `unwrap_or`, `map_err` 等を追加する。
- **File I/O** (`!File` effect): ファイルの読み書きを型安全に行えるようにする。
- **JSON** (`Json.*`): parse / encode / アクセサを提供する。`Json` 型を組み込み ADT として定義する。
- **CSV** (`Csv.*`): parse / encode（行ベース・ヘッダ付き）を提供する。

> **VM の高階関数**: `List.map` / `List.filter` / `List.fold` 等は eval.rs では v0.1.0 から動作しているが、VM (vm.rs) では未実装。v0.7.0 で VM にコールバック機構を導入して対応する。

---

## スコープ

### v0.7.0 で追加するもの

- `List.*` の完全化（高階関数 + `flat_map`, `zip`, `sort`, `reverse`, `concat`, `range`, `take`, `drop`, `enumerate`）
- `String.*` の完全化（`join`, `replace`, `starts_with`, `ends_with`, `contains`, `slice`）
- `Map.*` の完全化（`merge`, `has_key`, `size`, `is_empty`, `from_list`）
- `Option.*` の高階関数（`map`, `and_then`, `unwrap_or`, `or_else`, `is_some`, `is_none`）
- `Result.*` の高階関数（`map`, `map_err`, `and_then`, `unwrap_or`, `is_ok`, `is_err`）
- `File.*` 名前空間 + `!File` エフェクト（`read`, `write`, `append`, `exists`, `read_lines`, `write_lines`）
- `Json.*` 名前空間 + `Json` 組み込み ADT（`parse`, `encode`, `get`, `at`, `as_str`, `as_int`, `as_float`, `as_bool`）
- `Csv.*` 名前空間（`parse`, `parse_with_header`, `encode`, `encode_with_header`）
- VM のコールバック機構（`VM::call_value` — 高階ビルトインからクロージャを呼び出す）

### v0.7.0 では含まないもの

- Favnir ネイティブ標準ライブラリ（`.fav` で書かれたprelude — v0.8.0以降）
- `Db.*` の VM 対応（Db.* は eval.rs のみ。VM 版は v0.8.0）
- `Http.*` の VM 対応（同上）
- 非同期 I/O
- `File.read_json` / `File.read_csv` の組み合わせ関数（ユーザーが `File.read |> Json.parse` と書く）
- 正規表現
- `std.math` (sin, cos, pow, sqrt 等)
- バイナリ I/O

---

## 標準ライブラリ API 仕様

### List

#### 既存（v0.1.0〜、VM 未対応だったもの）

```favnir
List.map(xs: List<A>, f: A -> B) -> List<B>
List.filter(xs: List<A>, pred: A -> Bool) -> List<A>
List.fold(xs: List<A>, init: B, f: B -> A -> B) -> B
```

> **注意**: `fold` の引数順は `(xs, init, f)` — 既存の eval.rs 実装に合わせる。

#### 新規追加

```favnir
// 変換
List.flat_map(xs: List<A>, f: A -> List<B>) -> List<B>
List.zip(xs: List<A>, ys: List<B>) -> List<Pair<A, B>>  // Pair = { first: A  second: B }
List.enumerate(xs: List<A>) -> List<Pair<Int, A>>        // { first: index  second: element }

// ソート・並び替え
List.sort(xs: List<A>, cmp: A -> A -> Int) -> List<A>  // cmp は ord.compare 相当
List.reverse(xs: List<A>) -> List<A>

// 結合・分割
List.concat(xs: List<A>, ys: List<A>) -> List<A>
List.take(xs: List<A>, n: Int) -> List<A>
List.drop(xs: List<A>, n: Int) -> List<A>
List.range(start: Int, end: Int) -> List<Int>           // [start, end) の整数列

// 探索
List.find(xs: List<A>, pred: A -> Bool) -> A?
List.any(xs: List<A>, pred: A -> Bool) -> Bool
List.all(xs: List<A>, pred: A -> Bool) -> Bool
List.index_of(xs: List<A>, pred: A -> Bool) -> Int?

// 変換
List.join(xs: List<String>, sep: String) -> String      // String.join と同じ
```

`zip` / `enumerate` が返す `Pair<A,B>` は既存の record 型として扱う（`{ first: A  second: B }`）。

---

### String

#### 既存（完全対応済み）

```favnir
String.trim(s: String) -> String
String.lower(s: String) -> String
String.upper(s: String) -> String
String.split(s: String, delim: String) -> List<String>
String.length(s: String) -> Int
String.is_empty(s: String) -> Bool
```

#### 新規追加

```favnir
String.join(xs: List<String>, sep: String) -> String
String.replace(s: String, from: String, to: String) -> String
String.starts_with(s: String, prefix: String) -> Bool
String.ends_with(s: String, suffix: String) -> Bool
String.contains(s: String, sub: String) -> Bool
String.slice(s: String, start: Int, end: Int) -> String  // [start, end) バイトでなく文字単位
String.repeat(s: String, n: Int) -> String
String.char_at(s: String, idx: Int) -> String?           // idx 番目の文字（1文字のString）
String.to_int(s: String) -> Int?                         // 変換失敗は none
String.to_float(s: String) -> Float?
String.from_int(n: Int) -> String                        // Int.show.show と同等
String.from_float(f: Float) -> String
```

---

### Map

> **v0.7.0 の制約**: Map のキーは `String` 固定。型表記は `Map<V>`（値の型のみ引数に持つ）。
> `Map<K, V>` への汎化は将来バージョンで対応する。

#### 既存（完全対応済み）

```favnir
Map.get(m: Map<V>, key: String) -> V?
Map.set(m: Map<V>, key: String, value: V) -> Map<V>
Map.keys(m: Map<V>) -> List<String>
Map.values(m: Map<V>) -> List<V>
Map.delete(m: Map<V>, key: String) -> Map<V>
```

> `Map<V>` の内部表現は `Record = HashMap<String, Value>` — v0.6.0 から継続。

#### 新規追加

```favnir
Map.has_key(m: Map<V>, key: String) -> Bool
Map.size(m: Map<V>) -> Int
Map.is_empty(m: Map<V>) -> Bool
Map.merge(base: Map<V>, overrides: Map<V>) -> Map<V>   // overrides が優先
Map.from_list(pairs: List<Pair<String, V>>) -> Map<V>  // List<(key, value)> から構築
Map.to_list(m: Map<V>) -> List<Pair<String, V>>        // ソート済みキー順
Map.map_values(m: Map<A>, f: A -> B) -> Map<B>         // 高階: 値を変換
Map.filter_values(m: Map<A>, pred: A -> Bool) -> Map<A>
```

---

### Option

#### 既存（コンストラクタのみ）

```favnir
Option.some(v: A) -> A?
Option.none() -> A?
```

#### 新規追加

```favnir
Option.map(o: A?, f: A -> B) -> B?
Option.and_then(o: A?, f: A -> B?) -> B?              // flatMap
Option.unwrap_or(o: A?, default: A) -> A
Option.or_else(o: A?, f: () -> A?) -> A?
Option.is_some(o: A?) -> Bool
Option.is_none(o: A?) -> Bool
Option.to_result(o: A?, err: E) -> Result<A, E>        // none → err(err)
```

---

### Result

#### 既存（コンストラクタのみ）

```favnir
Result.ok(v: A) -> A!
Result.err(e: E) -> A!
```

#### 新規追加

```favnir
Result.map(r: A!, f: A -> B) -> B!
Result.map_err(r: A!, f: E -> F) -> Result<A, F>
Result.and_then(r: A!, f: A -> B!) -> B!              // flatMap
Result.unwrap_or(r: A!, default: A) -> A
Result.is_ok(r: A!) -> Bool
Result.is_err(r: A!) -> Bool
Result.to_option(r: A!) -> A?                          // err → none
```

---

### File （新規）

`!File` エフェクトを持つ関数群。ファイルシステムへのアクセスを型安全にマークする。

```favnir
File.read(path: String) -> String !File               // ファイル全体をStringで返す
File.read_lines(path: String) -> List<String> !File   // 行ごとのリスト（改行を除く）
File.write(path: String, content: String) -> Unit !File
File.write_lines(path: String, lines: List<String>) -> Unit !File
File.append(path: String, content: String) -> Unit !File
File.exists(path: String) -> Bool !File
File.delete(path: String) -> Unit !File
```

エラー処理: ファイルが見つからない場合などの I/O エラーは実行時エラー（`VMError`）として伝播する。
将来的には `Result<String, FileError>` を返す版を追加予定。

#### `!File` エフェクト

チェッカーに `Effect::File` を追加する。

```rust
// src/ast.rs
pub enum Effect {
    // ...既存...
    File,
}
```

チェック:
- `File.*` 関数を呼び出す場合、呼び出し元の関数に `!File` が必要（E036）。

---

### Json （新規）

#### `Json` 型

`Json` は組み込み ADT として扱う。型チェッカーとインタープリタに直接登録する（ユーザーは `type Json = ...` を書かない）。

Favnir の型構文で書けば以下に相当する（実際にユーザーがこのコードを書く必要はない）:

```favnir
type Json =
    | json_null
    | json_bool(Bool)
    | json_int(Int)
    | json_float(Float)
    | json_str(String)
    | json_array(List<Json>)       // 再帰型
    | json_object(List<JsonField>) // 再帰型

type JsonField = { key: String  value: Json }
```

> **実装方針**: `Json` / `JsonField` は既存の `Value::Variant` + `Value::Record` + `Value::List` を使ってエンコードする（`Value` 型への追加なし）。
> - `json_null` → `Value::Variant("json_null", None)`
> - `json_array([...])` → `Value::Variant("json_array", Some(Value::List([...])))`
> - `json_object([...])` → `Value::Variant("json_object", Some(Value::Record(HashMap)))`
>
> これにより eval.rs・vm.rs ともに変更箇所を最小に抑える。

#### `Json.*` 関数

```favnir
// 構築
Json.null() -> Json
Json.bool(b: Bool) -> Json
Json.int(n: Int) -> Json
Json.float(f: Float) -> Json
Json.str(s: String) -> Json
Json.array(xs: List<Json>) -> Json
Json.object(fields: List<JsonField>) -> Json

// パース / エンコード
Json.parse(s: String) -> Json?             // 失敗は none
Json.encode(j: Json) -> String             // compact JSON（改行なし）
Json.encode_pretty(j: Json) -> String      // インデント付き JSON

// アクセサ（型安全）
Json.get(j: Json, key: String) -> Json?    // json_object のフィールドを取得
Json.at(j: Json, idx: Int) -> Json?        // json_array の要素を取得
Json.as_str(j: Json) -> String?
Json.as_int(j: Json) -> Int?
Json.as_float(j: Json) -> Float?
Json.as_bool(j: Json) -> Bool?
Json.as_array(j: Json) -> List<Json>?
Json.is_null(j: Json) -> Bool

// 変換ヘルパー
Json.keys(j: Json) -> List<String>?        // json_object のキー一覧
Json.length(j: Json) -> Int?               // json_array / json_object の要素数
```

#### 利用例

```favnir
public fn main() -> Unit !Io !File {
    bind raw    <- File.read("data.json")
    bind result <- Json.parse(raw)
    match result {
        none    => IO.println("parse error")
        some(j) =>
            match Json.get(j, "name") {
                some(name) => IO.println(Json.as_str(name) |> match {
                    some(s) => s
                    none    => "(not a string)"
                })
                none => IO.println("no name field")
            }
    }
}
```

---

### Csv （新規）

```favnir
// パース
Csv.parse(s: String) -> List<List<String>>
// 例: "a,b\n1,2\n" → [["a","b"],["1","2"]]

Csv.parse_with_header(s: String) -> List<Map<String>>
// 1行目をヘッダとして使い、各行を Map にする
// 例: "name,age\nAlice,30\n" → [{"name":"Alice","age":"30"}]

// エンコード
Csv.encode(rows: List<List<String>>) -> String
Csv.encode_with_header(header: List<String>, rows: List<List<String>>) -> String

// ユーティリティ
Csv.from_records(records: List<Map<String>>) -> String
// Map のリストを CSV に変換（キーをソートしてヘッダ行を生成）
```

#### 利用例（v0.7.0 完了条件）

```favnir
// CSV を読んで変換して JSON に書き出す
public fn main() -> Unit !Io !File {
    bind csv_src <- File.read("input.csv")
    bind rows    <- Csv.parse_with_header(csv_src)
    bind json_rows <- List.map(rows, |row|
        Json.object(List.map(Map.to_list(row), |pair|
            { key: pair.first  value: Json.str(pair.second) }
        ))
    )
    bind output <- Json.encode_pretty(Json.array(json_rows))
    File.write("output.json", output);
    IO.println("done")
}
```

---

## VM のコールバック機構

### 課題

`List.map` / `List.filter` / `List.fold` / `List.sort` 等の高階ビルトインは、VM 内でクロージャを呼び出す必要がある。現在の `vm_call_builtin` はアーティファクトや VM 状態にアクセスできない。

### 解決策: `VM::call_value`

高階ビルトインを CALL ハンドラ内でインライン処理する。`vm_call_builtin` は純粋関数のみに限定し、高階ビルトインは CALL ディスパッチに直接実装する。

```
CALL で callee = Builtin("List.map") を検出
  → args = [f: VMValue, xs: VMValue::List] を取り出す
  → xs の各要素 elem に対して:
       stack.push(f.clone())
       stack.push(elem)
       vm_call_one(artifact, &mut frames, &mut stack, ...)
       // "vm_call_one" は 1 回の CALL 命令相当の処理（フレームを積んでループを回す）
  → 結果を集めて VMValue::List をプッシュ
```

#### `vm_call_value` ヘルパー設計

```rust
impl VM {
    /// 1つの値（CompiledFn / Closure / Builtin）を指定した引数で呼び出す。
    /// VM の現在の状態（stack / frames / collect_frames / emit_log）を共有して実行する。
    fn call_value(
        &mut self,
        artifact: &FvcArtifact,
        callee: VMValue,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError>
}
```

`call_value` は：
1. 現在の stack 末尾に callee と args を積む
2. 通常の CALL 命令と同じフレームプッシュ処理を行う
3. 新フレームが終了するまで dispatch ループを回す（`frames.len() == before_len` になったら抜ける）
4. スタックトップを結果として返す

これにより emit_log / collect_frames が共有され、副作用を持つクロージャも正しく動作する。

#### 対応する高階ビルトイン（CALL ハンドラ内で処理）

| ビルトイン | 引数 | VM 処理 |
|---|---|---|
| `List.map` | `(xs, f)` | xs の各要素に call_value(f, [elem]) |
| `List.filter` | `(xs, pred)` | call_value(pred, [elem]) → Bool でフィルタ |
| `List.fold` | `(xs, init, f)` | accum = init; call_value(f, [accum, elem]) |
| `List.flat_map` | `(xs, f)` | call_value(f, [elem]) → List を concat |
| `List.sort` | `(xs, cmp)` | Rust の sort_by で call_value(cmp, [a, b]) |
| `List.find` | `(xs, pred)` | call_value(pred, [elem]) → Bool で探索 |
| `List.any` | `(xs, pred)` | 同上、短絡評価 |
| `List.all` | `(xs, pred)` | 同上、短絡評価 |
| `List.enumerate` | `(xs, f)` | call_value(f, [Int(i), elem]) |
| `Map.map_values` | `(m, f)` | call_value(f, [v]) |
| `Map.filter_values` | `(m, pred)` | call_value(pred, [v]) → Bool |
| `Option.map` | `(o, f)` | call_value(f, [inner]) if some |
| `Option.and_then` | `(o, f)` | call_value(f, [inner]) if some |
| `Result.map` | `(r, f)` | call_value(f, [inner]) if ok |
| `Result.and_then` | `(r, f)` | call_value(f, [inner]) if ok |

---

## `!File` エフェクト

### チェッカーへの追加

`src/ast.rs` の `Effect` 列挙体に `File` を追加する:

```rust
pub enum Effect {
    Pure, Io, Db, Network, Emit(Type), Trace, File,  // File を追加
}
```

`src/checker.rs`:
- `File.*` 関数の呼び出し時に `current_effects` に `Effect::File` が含まれるか確認（E036）。
- `format_effects` / `merge_effect` に `File` を追加する。

### エラーコード

| コード | 内容 |
|--------|------|
| E036 | `!File` エフェクトなしで `File.*` を使用 |
| E037 | ファイル I/O 実行時エラー（読み込み失敗・書き込み失敗等） |

---

## Cargo.toml の変更

JSON パース / CSV パースの実装に外部クレートを使う:

```toml
[dependencies]
# 既存
rusqlite = { version = "0.31", features = ["bundled"] }
ureq = "2"

# 新規追加
serde_json = "1"   # Json.parse / Json.encode 用
csv = "1"          # Csv.parse / Csv.encode 用
```

> 代替: serde_json の代わりに手書きパーサを実装する選択肢もあるが、serde_json を使う方が信頼性が高く実装コストが低い。

---

## 実装フェーズ

### Phase 1: List / String / Map の完全化

**eval.rs**:
- `List.flat_map`, `List.zip`, `List.sort`, `List.reverse`, `List.concat`, `List.range`, `List.take`, `List.drop`, `List.enumerate`, `List.find`, `List.any`, `List.all`, `List.index_of`
- `String.join`, `String.replace`, `String.starts_with`, `String.ends_with`, `String.contains`, `String.slice`, `String.repeat`, `String.char_at`, `String.to_int`, `String.to_float`, `String.from_int`, `String.from_float`
- `Map.has_key`, `Map.size`, `Map.is_empty`, `Map.merge`, `Map.from_list`, `Map.to_list`, `Map.map_values`, `Map.filter_values`

**vm.rs**:
- `VM::call_value` ヘルパーを追加
- 高階ビルトインを CALL ハンドラに追加（上記テーブルの全関数）
- 非高階ビルトインを `vm_call_builtin` に追加

### Phase 2: Option / Result の高階関数

**eval.rs + vm.rs**:
- `Option.map`, `Option.and_then`, `Option.unwrap_or`, `Option.or_else`, `Option.is_some`, `Option.is_none`, `Option.to_result`
- `Result.map`, `Result.map_err`, `Result.and_then`, `Result.unwrap_or`, `Result.is_ok`, `Result.is_err`, `Result.to_option`

### Phase 3: File I/O

- `src/ast.rs`: `Effect::File` 追加
- `src/checker.rs`: E036 チェックを追加
- `src/eval.rs`: `File.*` ビルトインを追加（`std::fs` 使用）
- `src/vm.rs`: `vm_call_builtin` に `File.*` を追加
- `src/main.rs`: `format_effects` に `File` を追加

### Phase 4: JSON

- `Cargo.toml`: `serde_json = "1"` を追加
- `src/eval.rs`: `Json.*` ビルトインを追加（serde_json → `Value::Variant` / `Value::Record` / `Value::List` に変換）
- `src/vm.rs`: `vm_call_builtin` に `Json.*` を追加
- `src/compiler.rs`: `Json` / `JsonField` を組み込みグローバルとして登録

### Phase 5: CSV

- `Cargo.toml`: `csv = "1"` を追加
- `src/eval.rs`: `Csv.*` ビルトインを追加
- `src/vm.rs`: `vm_call_builtin` に `Csv.*` を追加

### Phase 6: テスト

- 各ビルトインの単体テスト（eval.rs と vm.rs の両方）
- `fav run` と `fav exec` の出力一致テスト（高階関数を含む）
- 完了条件のサンプル（CSV → JSON 変換）の動作確認
- 既存テスト全パスの確認

---

## エラーコード（v0.7.0 追加分）

| コード | フェーズ | 内容 |
|--------|----------|------|
| E036 | check | `!File` エフェクトなしで `File.*` を使用 |
| E037 | exec/run | ファイル I/O エラー（not found, permission denied 等） |

---

## 既存コードへの影響

| ファイル | 変更 |
|---|---|
| `src/ast.rs` | `Effect::File` を追加 |
| `src/checker.rs` | E036 チェック / `format_effects` / `merge_effect` に `File` を追加 |
| `src/eval.rs` | `List.*` / `String.*` / `Map.*` / `Option.*` / `Result.*` / `File.*` / `Json.*` / `Csv.*` ビルトインを追加 |
| `src/vm.rs` | `VM::call_value` 追加; 高階ビルトインを CALL ハンドラに追加; `vm_call_builtin` に新ビルトインを追加 |
| `src/compiler.rs` | `Json` / `JsonField` / `Csv` を組み込みグローバルとして登録 |
| `src/main.rs` | `format_effects` に `File` を追加; HELP 文字列を更新; 組み込み関数の一覧を更新 |
| `Cargo.toml` | `serde_json = "1"`, `csv = "1"` を追加 |

---

## 完了条件

1. 以下のサンプルが `fav run` / `fav exec` の両方で動作する:
   ```favnir
   public fn main() -> Unit !Io !File {
       bind csv_src <- File.read("input.csv")
       bind rows    <- Csv.parse_with_header(csv_src)
       bind json_rows <- List.map(rows, |row|
           Json.object(List.map(Map.to_list(row), |pair|
               { key: pair.first  value: Json.str(pair.second) }
           ))
       )
       bind output <- Json.encode_pretty(Json.array(json_rows))
       File.write("output.json", output);
       IO.println("done")
   }
   ```
2. `List.map` / `List.filter` / `List.fold` が VM（`fav exec`）でも動作する
3. `File.*` 関数に対する `!File` エフェクト検査が動作する（E036）
4. `Json.parse` / `Json.encode` が正しく動作する
5. `Csv.parse_with_header` / `Csv.encode` が正しく動作する
6. 既存テストが全パス（デグレなし）
7. 各新規ビルトインに単体テストがある

---

## 設計メモ

### eval.rs と vm.rs の役割分担

v0.7.0 では 2 つの実行パスが存在する。役割を明確に分けておく。

| 項目 | eval.rs | vm.rs |
|---|---|---|
| 位置づけ | **暫定** — tree-walking インタープリタ | **本命** — bytecode VM（`fav build` + `fav exec`） |
| 目的 | `fav run` / `fav check` の高速フィードバック | 実行バイナリ（`.fvc` artifact）による本番実行 |
| 高階関数 | v0.1.0 から動作中 | v0.7.0 で `call_value` 機構を導入して対応 |
| 廃止予定 | v0.8.0 でビルトインを共通化し始める | 長期的にはこちらが唯一の実行パスになる |

v0.7.0 の実装方針:
- **新規ビルトイン**は必ず eval.rs と vm.rs の両方に実装する
- eval.rs の実装が「参照実装」で、vm.rs の実装が「本命」
- テストでは eval.rs の結果と vm.rs（build + exec）の結果を一致確認する

v0.8.0 以降での解消案:
- **Option A**: ビルトイン実装を `src/builtins.rs` に集約し、eval.rs と vm.rs から共通呼び出し（最有力候補）
- **Option B**: ビルトインを Favnir ネイティブコード（prelude.fav）として実装し、artifact に埋め込む（理想、複雑）

### `Map<V>` の型表現

現在の `Map` 型は `Map<V>` として表記されているが、内部的には `Value::Record(HashMap<String, Value>)` を流用している。`Map.get` の返り値型が `V?` になっている。型チェッカー上は `Map<String>` 等と書くが、実行時は動的型付けになる。

v0.7.0 では型チェッカーの扱いをそのまま維持し、実行時の動的ディスパッチに頼る。将来的には `Map<K, V>` の完全なジェネリクス対応が必要。

### `Json` 型と再帰

`json_array(List<Json>)` / `json_object(List<JsonField>)` は再帰的な型。Favnir の型チェッカーは現在 ADT の再帰を「名前で参照するため OK」として扱えるか要確認。

実装方針:
- `Json` を組み込み型として型チェッカーに直接登録し、再帰的な型定義をハードコードする
- ユーザーが `type Json = ...` を書く必要はない
- 将来的には Favnir の型システムが真の再帰型をサポートする（v1.0.0 以降）
