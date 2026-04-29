# Favnir v0.7.0 実装計画

更新日: 2026-04-29

---

## Phase 1: List 完全化 + VM コールバック機構

### `VM::call_value` の追加 (`src/vm.rs`)

高階ビルトイン（`List.map` 等）がクロージャを呼び出すためのヘルパー。

```rust
impl VM {
    /// callee（CompiledFn / Closure / Builtin）を args で呼び出す。
    /// 現在の VM 状態（stack / frames / collect_frames / emit_log）を共有する。
    fn call_value(
        &mut self,
        artifact: &FvcArtifact,
        callee: VMValue,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        let depth_before = self.frames.len();
        // callee と args をスタックに積んで CALL を模倣
        let base = self.stack.len();
        self.stack.push(callee.clone());
        for arg in args {
            self.stack.push(arg);
        }
        // callee 種別に応じてフレームをプッシュ
        self.dispatch_call(artifact, callee, base, depth_before)?;
        // depth_before に戻るまでディスパッチループを回す
        self.run_until_depth(artifact, depth_before)
    }
}
```

`run_until_depth(artifact, target_depth)` は既存の dispatch ループを分離した関数。
フレームスタックの深さが `target_depth` 以下になったらスタックトップを返す。

### eval.rs: 新規 List 関数

`register_builtins` の `"List"` セクションに追加する:

```rust
("List", "flat_map") => {
    // args: [xs: List, f: Closure]
    let (xs, f) = (args[0], args[1]);
    // xs の各要素に f を適用し、結果の List を concat
}
("List", "zip") => {
    // args: [xs: List, ys: List]
    // Pair { first: x, second: y } のリストを返す
}
("List", "sort") => {
    // args: [xs: List, cmp: Closure] — cmp(a, b) -> Int
    // Rust の sort_by で cmp クロージャを呼び出す
}
("List", "range") => {
    // args: [start: Int, end: Int]
    // [start, end) の整数リストを返す
}
// ... reverse, concat, take, drop, enumerate, find, any, all, index_of, join
```

### vm.rs: CALL ハンドラへの高階ビルトイン追加

`CALL` 命令の `VMValue::Builtin(name)` ケースで高階関数を分岐:

```rust
VMValue::Builtin(ref name) => {
    match name.as_str() {
        "List.map" => {
            // args = [xs: VMValue::List, f: VMValue]
            let mut args = pop_args(arg_count, &mut vm.stack)?;
            let f  = args.pop().expect("f");
            let xs = args.pop().expect("xs");
            match xs {
                VMValue::List(items) => {
                    let mut result = Vec::with_capacity(items.len());
                    for item in items {
                        let v = vm.call_value(artifact, f.clone(), vec![item])?;
                        result.push(v);
                    }
                    vm.stack.remove(callee_pos);
                    vm.stack.push(VMValue::List(result));
                }
                _ => return Err(vm.error(artifact, "List.map: expected List")),
            }
        }
        "List.filter" => { /* 同様 */ }
        "List.fold"   => { /* args = [xs, init, f]; accum を init に初期化してループ */ }
        "List.flat_map" | "List.sort" | "List.find" | "List.any" | "List.all" => { /* 同様 */ }
        // 非高階: vm_call_builtin に委譲
        other => {
            let mut args = pop_args(arg_count, &mut vm.stack)?;
            vm.stack.remove(callee_pos);
            let result = vm_call_builtin(other, args, &mut vm.emit_log)
                .map_err(|e| vm.error(artifact, &e))?;
            vm.stack.push(result);
        }
    }
}
```

---

## Phase 2: String / Map 完全化

### eval.rs: 新規 String 関数

```rust
("String", "join") => {
    // args: [xs: List<String>, sep: String]
    // xs の各要素を sep で結合
}
("String", "replace") => {
    // args: [s: String, from: String, to: String]
    s.replace(&from, &to)
}
("String", "starts_with") => {
    // args: [s: String, prefix: String]
    Value::Bool(s.starts_with(&prefix))
}
("String", "ends_with")   => { /* 同様 */ }
("String", "contains")    => { /* s.contains(&sub) */ }
("String", "slice") => {
    // args: [s: String, start: Int, end: Int]
    // 文字単位（chars().collect::<Vec<_>>()[start..end]）
}
("String", "repeat")   => { /* s.repeat(n) */ }
("String", "char_at")  => { /* chars().nth(idx) → Option<String> */ }
("String", "to_int")   => { /* s.parse::<i64>().ok() → some/none */ }
("String", "to_float") => { /* s.parse::<f64>().ok() → some/none */ }
("String", "from_int")   => { /* n.to_string() */ }
("String", "from_float") => { /* f.to_string() */ }
```

### eval.rs: 新規 Map 関数

```rust
("Map", "has_key")       => { Value::Bool(m.contains_key(&key)) }
("Map", "size")          => { Value::Int(m.len() as i64) }
("Map", "is_empty")      => { Value::Bool(m.is_empty()) }
("Map", "merge") => {
    // args: [base: Map, overrides: Map]
    // base をコピーして overrides のエントリで上書き
}
("Map", "from_list") => {
    // args: [pairs: List<Pair<String, V>>]
    // Pair.first をキー、Pair.second を値として Map を構築
}
("Map", "to_list") => {
    // キーをソートして Pair のリストを返す
}
("Map", "map_values") => {
    // args: [m: Map, f: Closure] — 高階: eval.rs は eval_apply で処理
}
("Map", "filter_values") => {
    // args: [m: Map, pred: Closure] — 高階
}
```

### vm.rs: String / Map ビルトインの追加

非高階関数は `vm_call_builtin` に追加する。
高階（`Map.map_values`, `Map.filter_values`）は CALL ハンドラに追加する。

---

## Phase 3: Option / Result 高階関数

### eval.rs + vm.rs

```rust
// Option
("Option", "map") => {
    // args: [o: Option<A>, f: A -> B]
    // some(inner) → eval_apply(f, inner) → some(result)
    // none        → none
}
("Option", "and_then") => {
    // some(inner) → eval_apply(f, inner) （f は Option<B> を返す）
    // none        → none
}
("Option", "unwrap_or")   => { /* some(v) → v, none → default */ }
("Option", "or_else")     => { /* none → eval_apply(f, ()) */ }
("Option", "is_some")     => { /* Variant("some", _) → Bool(true) */ }
("Option", "is_none")     => { /* Variant("none", _) → Bool(true) */ }
("Option", "to_result")   => { /* some(v) → ok(v), none → err(err_val) */ }

// Result
("Result", "map") => {
    // ok(inner)  → eval_apply(f, inner) → ok(result)
    // err(e)     → err(e) そのまま
}
("Result", "map_err")  => { /* err(e) → err(eval_apply(f, e)), ok(v) → ok(v) */ }
("Result", "and_then") => { /* ok(inner) → eval_apply(f, inner), err(e) → err(e) */ }
("Result", "unwrap_or")=> { /* ok(v) → v, err(_) → default */ }
("Result", "is_ok")    => { /* ok(_) → true */ }
("Result", "is_err")   => { /* err(_) → true */ }
("Result", "to_option")=> { /* ok(v) → some(v), err(_) → none */ }
```

vm.rs では `Option.map` / `Option.and_then` / `Result.map` / `Result.and_then` を CALL ハンドラ内で `call_value` を使って処理する。残りは `vm_call_builtin` に追加する。

---

## Phase 4: `!File` エフェクト + `File.*` ビルトイン

### `src/ast.rs`

```rust
pub enum Effect {
    Pure, Io, Db, Network, Emit(Type), Trace, File,  // File を追加
}
```

`Effect` を使う全 `match` / `display` / `merge_effect` / `format_effects` に `File` アームを追加する。

### `src/checker.rs`

```rust
fn check_builtin_call(&mut self, ns: &str, method: &str, span: &Span) {
    match ns {
        "File" => {
            if !self.has_effect(|e| matches!(e, Effect::File)) {
                self.type_error("E036", "File.* requires !File effect", span);
            }
        }
        // 既存: "Db" → E007, "Http" → E008, ...
    }
}
```

### `src/eval.rs`

```rust
("File", "read") => {
    // args: [path: String]
    let path = args[0].as_str()?;
    match std::fs::read_to_string(&path) {
        Ok(s)  => Ok(Value::Str(s)),
        Err(e) => Err(RuntimeError::new(format!("E037: {e}"), span)),
    }
}
("File", "read_lines") => {
    let s = std::fs::read_to_string(&path)?;
    Ok(Value::List(s.lines().map(|l| Value::Str(l.to_string())).collect()))
}
("File", "write") => {
    // args: [path: String, content: String]
    std::fs::write(&path, content.as_bytes())?;
    Ok(Value::Unit)
}
("File", "write_lines") => { /* lines.join("\n") を write */ }
("File", "append") => {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().append(true).create(true).open(&path)?;
    f.write_all(content.as_bytes())?;
    Ok(Value::Unit)
}
("File", "exists") => {
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}
("File", "delete") => {
    std::fs::remove_file(&path)?;
    Ok(Value::Unit)
}
```

vm.rs の `vm_call_builtin` にも同等の実装を追加する（`std::fs` は vm.rs からも直接使える）。

### `src/main.rs`

`format_effects` に `File` を追加し、HELP のエフェクト表に `!File` を記載する。

---

## Phase 5: JSON (`Json.*`)

### `Cargo.toml`

```toml
[dependencies]
serde_json = "1"
```

### `Json` の内部表現

`Json` 型は Favnir の既存 Value バリアントを組み合わせてエンコードする:

| JSON 型 | Favnir Value |
|---|---|
| `null` | `Value::Variant("json_null", None)` |
| `true` / `false` | `Value::Variant("json_bool", Some(Value::Bool(...)))` |
| 整数 | `Value::Variant("json_int", Some(Value::Int(...)))` |
| 浮動小数 | `Value::Variant("json_float", Some(Value::Float(...)))` |
| 文字列 | `Value::Variant("json_str", Some(Value::Str(...)))` |
| 配列 | `Value::Variant("json_array", Some(Value::List(...)))` |
| オブジェクト | `Value::Variant("json_object", Some(Value::Record(...)))` |

この方針により既存の `Value` 型に変更を加えず、Json 値を `Value::Variant` としてエンコードできる。

### `eval.rs` + `vm.rs`: 変換ヘルパー

```rust
/// serde_json::Value → Favnir Value（Json 表現）
fn serde_to_favnir(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null      => Value::Variant("json_null".into(), None),
        serde_json::Value::Bool(b)   => Value::Variant("json_bool".into(),
                                            Some(Box::new(Value::Bool(b)))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Variant("json_int".into(), Some(Box::new(Value::Int(i))))
            } else {
                Value::Variant("json_float".into(),
                    Some(Box::new(Value::Float(n.as_f64().unwrap_or(0.0)))))
            }
        }
        serde_json::Value::String(s) => Value::Variant("json_str".into(),
                                            Some(Box::new(Value::Str(s)))),
        serde_json::Value::Array(xs) => Value::Variant("json_array".into(),
                                            Some(Box::new(Value::List(
                                                xs.into_iter().map(serde_to_favnir).collect()
                                            )))),
        serde_json::Value::Object(m) => {
            let map: HashMap<_, _> = m.into_iter()
                .map(|(k, v)| (k, serde_to_favnir(v))).collect();
            Value::Variant("json_object".into(), Some(Box::new(Value::Record(map))))
        }
    }
}

/// Favnir Value（Json 表現）→ serde_json::Value
fn favnir_to_serde(v: &Value) -> Option<serde_json::Value> {
    match v {
        Value::Variant(tag, payload) => match tag.as_str() {
            "json_null"   => Some(serde_json::Value::Null),
            "json_bool"   => Some(serde_json::Value::Bool(payload_bool(payload)?)),
            "json_int"    => Some(serde_json::json!(payload_int(payload)?)),
            "json_float"  => Some(serde_json::json!(payload_float(payload)?)),
            "json_str"    => Some(serde_json::Value::String(payload_str(payload)?)),
            "json_array"  => { /* List を再帰変換 */ }
            "json_object" => { /* Record を再帰変換 */ }
            _ => None,
        },
        _ => None,
    }
}
```

### `Json.*` ビルトイン

```rust
("Json", "parse") => {
    // args: [s: String]
    match serde_json::from_str::<serde_json::Value>(&s) {
        Ok(v)  => Ok(Value::Variant("some".into(), Some(Box::new(serde_to_favnir(v))))),
        Err(_) => Ok(Value::Variant("none".into(), None)),
    }
}
("Json", "encode") => {
    match favnir_to_serde(&args[0]) {
        Some(v) => Ok(Value::Str(v.to_string())),
        None    => Err(RuntimeError::new("Json.encode: not a Json value", span)),
    }
}
("Json", "encode_pretty") => {
    match favnir_to_serde(&args[0]) {
        Some(v) => Ok(Value::Str(serde_json::to_string_pretty(&v).unwrap())),
        None    => Err(RuntimeError::new("Json.encode_pretty: not a Json value", span)),
    }
}
("Json", "get") => {
    // args: [j: Json, key: String]
    // json_object の Record から key を取得 → some/none
}
("Json", "at") => {
    // args: [j: Json, idx: Int]
    // json_array の List から idx 番目を取得
}
// as_str, as_int, as_float, as_bool, as_array, is_null, keys, length
```

### `src/compiler.rs`

`Json` / `JsonField` をグローバルテーブルに `Builtin` として登録する（ユーザーが `type Json` を書かなくても参照できるようにする）:

```rust
for name in &["Json", "JsonField", "Csv", "File"] {
    if !ctx.globals.contains_key(*name) {
        let idx = globals.len() as u16;
        ctx.globals.insert(name.to_string(), idx);
        globals.push(IRGlobal { name: name.to_string(), kind: IRGlobalKind::Builtin });
    }
}
```

---

## Phase 6: CSV (`Csv.*`)

### `Cargo.toml`

```toml
[dependencies]
csv = "1"
```

### `Csv.*` ビルトイン

```rust
("Csv", "parse") => {
    // args: [s: String]
    // csv::Reader でパース → List<List<String>>
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(s.as_bytes());
    let rows: Vec<Value> = rdr.records()
        .filter_map(|r| r.ok())
        .map(|row| {
            Value::List(row.iter().map(|f| Value::Str(f.to_string())).collect())
        })
        .collect();
    Ok(Value::List(rows))
}
("Csv", "parse_with_header") => {
    // args: [s: String]
    // 1行目をヘッダとして、各行を Map<String, String> に変換
    let mut rdr = csv::Reader::from_reader(s.as_bytes());
    let headers = rdr.headers()?.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let rows: Vec<Value> = rdr.records()
        .filter_map(|r| r.ok())
        .map(|row| {
            let map: HashMap<_, _> = headers.iter().zip(row.iter())
                .map(|(k, v)| (k.clone(), Value::Str(v.to_string())))
                .collect();
            Value::Record(map)
        })
        .collect();
    Ok(Value::List(rows))
}
("Csv", "encode") => {
    // args: [rows: List<List<String>>]
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(vec![]);
    // 各行を書き込んで String に変換
}
("Csv", "encode_with_header") => {
    // args: [header: List<String>, rows: List<List<String>>]
}
("Csv", "from_records") => {
    // args: [records: List<Map<String, String>>]
    // 全 Map のキーをソートして header を生成し CSV を構築
}
```

---

## Phase 7: テストとサンプル

### 単体テスト方針

各新規ビルトインについて、eval.rs（tree-walking）と vm.rs（VM）の両方でテストを書く。

#### eval.rs テスト例

```rust
#[test]
fn test_list_flat_map() {
    let src = "fn f(xs: List<Int>) -> List<Int> { List.flat_map(xs, |x| [x, x]) }";
    // [[1,1],[2,2]] ではなく [1,1,2,2] が返る
}

#[test]
fn test_list_sort() {
    let src = "fn f(xs: List<Int>) -> List<Int> {
        List.sort(xs, |a, b| Int.ord.compare(a, b))
    }";
    // [3,1,2] → [1,2,3]
}

#[test]
fn test_option_and_then() {
    let src = r#"fn f(o: Int?) -> Int? { Option.and_then(o, |x| Option.some(x + 1)) }"#;
    // some(4) → some(5); none → none
}

#[test]
fn test_file_read_write() {
    // tempfile で一時ファイルを作成し File.write / File.read が往復できる
}

#[test]
fn test_json_parse_encode_roundtrip() {
    let src = r#"fn f(s: String) -> String? {
        Option.map(Json.parse(s), |j| Json.encode(j))
    }"#;
    // '{"a":1}' → some('{"a":1}')
}

#[test]
fn test_csv_parse_with_header() {
    let src = r#"fn f(s: String) -> List<Map<String>> {
        Csv.parse_with_header(s)
    }"#;
    // "name,age\nAlice,30\n" → [{"name":"Alice","age":"30"}]
}
```

#### vm.rs テスト例（統合テスト）

```rust
#[test]
fn vm_integration_list_map_via_closure() {
    let source = r#"
public fn main() -> List<Int> {
    List.map([1, 2, 3], |x| x * 2)
}
"#;
    // [2, 4, 6]
}

#[test]
fn vm_integration_option_and_then() {
    let source = r#"
public fn main() -> Int? {
    Option.and_then(Option.some(4), |x| Option.some(x + 1))
}
"#;
    // some(5)
}
```

### 完了条件サンプルファイル

`examples/csv_to_json.fav` を追加する:

```favnir
// examples/csv_to_json.fav — v0.7.0 完了条件デモ

public fn main() -> Unit !Io !File {
    bind csv_src  <- File.read("examples/data/sample.csv")
    bind rows     <- Csv.parse_with_header(csv_src)
    bind json_rows <- List.map(rows, |row|
        Json.object(List.map(Map.to_list(row), |pair|
            { key: pair.first  value: Json.str(pair.second) }
        ))
    )
    bind output   <- Json.encode_pretty(Json.array(json_rows))
    File.write("examples/data/output.json", output);
    IO.println("done")
}
```

`examples/data/sample.csv` も追加する:

```
name,age,city
Alice,30,Tokyo
Bob,25,Osaka
Charlie,35,Kyoto
```

---

## 設計メモ

### `VM::call_value` の実装詳細

`call_value` は既存の dispatch ループを再利用するため、ループを関数に分離するリファクタリングが必要になる。

現在の `VM::run_with_emits` は大きな `loop { match opcode { ... } }` を持つ。これを:

1. `VM::dispatch_one(artifact) -> Result<bool, VMError>` — 1命令実行、フレームが空になったら `false` を返す
2. `VM::run_loop(artifact, until_depth: usize) -> Result<VMValue, VMError>` — `until_depth` になるまでループ

に分離し、`call_value` から `run_loop(artifact, current_depth + 1)` を呼ぶ。

### eval.rs と vm.rs の二重管理

v0.7.0 は引き続き二重管理で進める。新規ビルトインを追加するたびに両方に実装する。

将来の解消方針（v0.8.0 候補）:
- ビルトイン実装を `src/builtins.rs` に集約し、eval.rs と vm.rs の両方から呼ぶ共通関数を定義する
- 高階関数のコールバックは `trait Callable` で抽象化する

### Favnir にリストリテラルがない問題

現在 Favnir にはリストリテラル構文（`[1, 2, 3]`）が存在しない。
`List.range(1, 4)` や `collect { yield 1; yield 2; yield 3; () }` で代替する。

v0.7.0 のテストでは `List.range` や `collect` を使ってリストを構築する。

### `String.slice` の文字境界

`String.slice(s, start, end)` は Rust の `&str` のバイト境界ではなく、Unicode スカラー値（`char`）単位でスライスする。`chars().skip(start).take(end - start).collect::<String>()` で実装する。
