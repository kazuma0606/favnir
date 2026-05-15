# Favnir v3.2.0 Specification

## Theme: `csv` + `json` rune — データフォーマット入出力

v3.2.0 はデータエンジニアリング現場で最も普遍的な入出力形式である CSV と JSON を
型安全に扱う公式 rune を提供する。
`Schema.adapt<T>` パターンによって「型宣言がスキーマ契約」になる設計を導入する。

---

## 1. 新規型定義

### 1.1 `SchemaError`

```favnir
type SchemaError = {
    field:    String
    expected: String
    got:      String
}
```

CSV/JSON フィールドの型変換失敗時に返す。VM 側で `VMValue::Record` として生成する。

### 1.2 `CsvOptions`

```favnir
type CsvOptions = {
    delimiter: String   // デフォルト ","
    has_header: Bool    // デフォルト true
}
```

---

## 2. `#[col(n)]` アノテーション構文

ヘッダーなし CSV の位置ベースマッピング用。型フィールドに付与する。

```favnir
type Row = {
    #[col(0)] id:    Int
    #[col(1)] name:  String
    #[col(2)] value: Float
}
```

### パーサー拡張

- `#[attr_name(arg)]` 形式を型フィールドのアノテーションとして解析する
- AST: `FieldDef { name, ty, attrs: Vec<FieldAttr> }`
- `FieldAttr { name: String, arg: Option<String> }`
- 現バージョンでサポートするアノテーション: `col(n)` のみ

---

## 3. VM プリミティブ

### 3.1 `Csv.parse_raw`

```
Csv.parse_raw(text: String, delimiter: String, has_header: Bool)
    -> Result<List<Map<String, String>>, SchemaError>
```

- CSV テキストを文字列マップのリスト（行ごとの `{header: value}` マップ）に変換
- `has_header: false` の場合はキーが `"0"`, `"1"`, ... の連番マップになる
- 空行はスキップ、引用符内の改行・カンマはエスケープ処理する

### 3.2 `Csv.write_raw`

```
Csv.write_raw(rows: List<Map<String, String>>, delimiter: String) -> String
```

- 文字列マップのリストを CSV テキストに変換（1行目はヘッダー）

### 3.3 `Schema.adapt`

```
Schema.adapt(rows: List<Map<String, String>>, type_name: String)
    -> Result<List<T>, SchemaError>
```

- `type_name` で指定された型の定義をチェッカーが埋め込んだメタデータで参照し、
  各フィールドの型変換（String → Int / Float / Bool / String）を実行する
- nullable フィールド（`Option<T>`）は空文字列 → `None` にマッピング
- 変換失敗時は最初のエラーで `Err(SchemaError)` を返す
- `#[col(n)]` アノテーションがあるときは位置ベースマッピングを使う

**型変換ルール**:

| フィールド型 | 変換方法 |
|------------|---------|
| `Int`      | `str.parse::<i64>()` 失敗で SchemaError |
| `Float`    | `str.parse::<f64>()` 失敗で SchemaError |
| `Bool`     | `"true"/"1"` → `true`, `"false"/"0"` → `false`, 他は SchemaError |
| `String`   | そのまま |
| `Option<T>` | 空文字列 → `None`, 非空なら `Some` で T 変換を適用 |

### 3.4 `Json.parse_raw`

```
Json.parse_raw(text: String) -> Result<Map<String, String>, SchemaError>
```

JSON オブジェクトを flat な文字列マップに変換（ネスト不可、v3.2.0 スコープ外）。

配列 JSON は `Json.parse_array_raw`:
```
Json.parse_array_raw(text: String) -> Result<List<Map<String, String>>, SchemaError>
```

### 3.5 `Json.write_raw`

```
Json.write_raw(map: Map<String, String>) -> String
Json.write_array_raw(rows: List<Map<String, String>>) -> String
```

---

## 4. `runes/csv/csv.fav`

Favnir 製 rune。VM プリミティブ上の薄いラッパー。

```favnir
// ヘッダーあり CSV → 型付きリスト
public stage parse<T>: String -> Result<List<T>, SchemaError> =
    |text| {
        bind raw <- Csv.parse_raw(text, ",", true)
        Schema.adapt(raw, type_name_of<T>())
    }

// ヘッダーなし CSV（#[col(n)] アノテーション使用）
public stage parse_positional<T>: String -> Result<List<T>, SchemaError> =
    |text| {
        bind raw <- Csv.parse_raw(text, ",", false)
        Schema.adapt(raw, type_name_of<T>())
    }

// 型付きリスト → CSV 文字列
public stage write<T>: List<T> -> String = Schema.to_csv

// オプション付き parse
public stage parse_with_opts<T>: String -> CsvOptions -> Result<List<T>, SchemaError> =
    |text| |opts| {
        bind raw <- Csv.parse_raw(text, opts.delimiter, opts.has_header)
        Schema.adapt(raw, type_name_of<T>())
    }
```

---

## 5. `runes/json/json.fav`

```favnir
// JSON オブジェクト → 型付き値
public stage parse<T>: String -> Result<T, SchemaError> =
    |text| {
        bind raw <- Json.parse_raw(text)
        Schema.adapt_one(raw, type_name_of<T>())
    }

// JSON 配列 → 型付きリスト
public stage parse_list<T>: String -> Result<List<T>, SchemaError> =
    |text| {
        bind raw <- Json.parse_array_raw(text)
        Schema.adapt(raw, type_name_of<T>())
    }

// 型付き値 → JSON 文字列
public stage write<T>: T -> String = Schema.to_json

// 型付きリスト → JSON 配列文字列
public stage write_list<T>: List<T> -> String = Schema.to_json_array
```

---

## 6. `Schema.to_csv` / `Schema.to_json` / `Schema.to_json_array`

VM プリミティブ。型の全フィールドを文字列に変換して CSV/JSON 化する。

```
Schema.to_csv(rows: List<T>, type_name: String) -> String
Schema.to_json(value: T, type_name: String) -> String
Schema.to_json_array(rows: List<T>, type_name: String) -> String
```

---

## 7. `type_name_of<T>()` 組み込み関数

チェッカーが呼び出し時の型引数 T の名前を解決して定数文字列として埋め込む特殊形式。
`Schema.adapt` / `Schema.to_*` に型情報を渡すブリッジ。

---

## 8. エラーコード追加

| コード | タイトル |
|--------|---------|
| E0501 | schema field missing — CSV/JSON に型の必須フィールドが存在しない |
| E0502 | schema type mismatch — フィールドの値が型変換できない |
| E0503 | col index out of range — `#[col(n)]` が CSV カラム数を超えている |
| E0504 | json parse error — JSON テキストの構文エラー |
| E0505 | csv parse error — CSV テキストの構文エラー（引用符の不一致等） |

---

## 9. `chain` との統合

`csv.parse<T>` は `Result<List<T>, SchemaError>` を返すため、
既存の `chain` キーワードとシームレスに統合できる。

```favnir
seq LoadPipeline =
    ReadFile
    |> csv.parse<Row> |> chain   // SchemaError 時はパイプライン中断
    |> Validate
    |> PrintLn
```

---

## 10. 利用例

### ヘッダーあり CSV の読み書き

```favnir
import rune "csv"

type User = {
    id:   Int
    name: String
    age:  Int
}

public fn main() -> Unit !Io {
    bind text <- File.read("users.csv")
    bind users <- csv.parse<User>(text) ?? []
    IO.println($"Loaded {List.length(users)} users")
    bind out <- csv.write<User>(users)
    IO.println(out)
}
```

### JSON の読み書き

```favnir
import rune "json"

type Config = {
    host: String
    port: Int
}

public fn main() -> Unit !Io {
    bind text <- File.read("config.json")
    bind config <- json.parse<Config>(text) ?? Config { host: "localhost"  port: 8080 }
    IO.println($"Connecting to {config.host}:{config.port}")
}
```

---

## 11. 完了条件

- `csv.parse<Row>` でヘッダーありCSVが `Result<List<Row>, SchemaError>` として読み込める
- `csv.parse_positional<Row>` で `#[col(n)]` アノテーションによる位置マッピングが動く
- カラム型変換失敗時に `SchemaError` が返る（`Any` に逃げない）
- `csv.write<Row>` で `List<Row>` をCSV文字列に書き出せる
- `json.parse<T>` / `json.write<T>` / `json.parse_list<T>` / `json.write_list<T>` が動く
- `chain` との組み合わせで変換失敗時にパイプラインが中断する
- E0501〜E0505 が適切なケースで発生する
- 既存テストが全て通る

---

## 12. 非ゴール（v3.2.0 スコープ外）

- ネスト JSON（配列内オブジェクト内オブジェクト等）の深い再帰変換
- CSV の CRLF 以外の改行の自動検出（Unix LF のみサポート）
- ストリーミング CSV（行単位の逐次処理）—  v3.6.0 の Incremental と合わせて検討
- `Map<K, V>` 型（汎用マップ）— 将来候補
- TSV / PSV 等のカスタム区切り文字（`CsvOptions.delimiter` で対応済み）
