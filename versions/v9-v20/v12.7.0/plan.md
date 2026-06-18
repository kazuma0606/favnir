# Favnir v12.7.0 実装計画

Date: 2026-06-08

---

## Phase A — `BuiltinPrimitive` 静的テーブル

### A-1: 構造体定義（driver.rs）

```rust
#[derive(serde::Serialize)]
struct BuiltinPrimitive {
    namespace:      &'static str,
    name:           &'static str,
    signature:      &'static str,
    effects:        &'static [&'static str],
    returns_result: bool,
    description:    &'static str,
}
```

### A-2: `builtin_primitives()` 関数を定義

全 Namespace の Primitive を静的スライスで返す関数。
各エントリは `BuiltinPrimitive { ... }` のリテラル。

**登録する Primitive 一覧:**

#### IO（8 関数）
| 名前 | シグネチャ | effects | returns_result |
|---|---|---|---|
| IO.println | (value: String) -> Unit | !IO | false |
| IO.print | (value: String) -> Unit | !IO | false |
| IO.println_int | (value: Int) -> Unit | !IO | false |
| IO.println_float | (value: Float) -> Unit | !IO | false |
| IO.println_bool | (value: Bool) -> Unit | !IO | false |
| IO.read_line | () -> String | !IO | false |
| IO.read_file_raw | (path: String) -> Result\<String, String\> | !IO | true |
| IO.write_file_raw | (path: String, content: String) -> Result\<Unit, String\> | !IO | true |
| IO.make_dir_raw | (path: String) -> Result\<Unit, String\> | !IO | true |
| IO.argv | () -> List\<String\> | !IO | false |
| IO.timestamp | () -> Int | !IO | false |
| IO.sleep_ms | (ms: Int) -> Unit | !IO | false |

#### Csv（2 関数）
| 名前 | シグネチャ | effects | returns_result |
|---|---|---|---|
| Csv.parse_raw | (text: String, sep: String, header: Bool) -> Result\<List\<Record\>, String\> | !IO | true |
| Csv.to_string_raw | (rows: List\<Record\>, sep: String, header: Bool) -> String | none | false |

#### Schema（3 関数）
| 名前 | シグネチャ |
|---|---|
| Schema.to_json_array | (rows: List\<Record\>, type_name: String) -> String |
| Schema.adapt | (row: Record, type_name: String) -> Result\<Record, String\> |
| Schema.validate | (row: Record, type_name: String) -> Result\<Unit, String\> |

#### Json（3 関数）
| 名前 | シグネチャ |
|---|---|
| Json.encode_raw | (value: Any) -> String |
| Json.decode_raw | (json: String) -> Result\<Any, String\> |
| Json.pretty_raw | (json: String) -> String |

#### Gen（5 関数）
| 名前 | シグネチャ |
|---|---|
| Gen.uuid | () -> String |
| Gen.uuid_v7 | () -> String |
| Gen.nano_id | () -> String |
| Gen.one_raw | (type_name: String) -> Result\<Any, String\> |
| Gen.hint_one_raw | (type_name: String, hints: String) -> Result\<Any, String\> |

#### AWS（4 関数）
| 名前 | シグネチャ | effects |
|---|---|---|
| AWS.s3_get_raw | (bucket: String, key: String) -> Result\<String, String\> | !AWS |
| AWS.s3_put_raw | (bucket: String, key: String, body: String) -> Result\<Unit, String\> | !AWS |
| AWS.sqs_send_raw | (url: String, body: String) -> Result\<Unit, String\> | !AWS |
| AWS.sqs_receive_raw | (url: String) -> Result\<String, String\> | !AWS |

#### Postgres（3 関数）
| 名前 | シグネチャ | effects |
|---|---|---|
| Postgres.execute_raw | (sql: String, params: String) -> Result\<Unit, String\> | !Postgres |
| Postgres.query_raw | (sql: String, params: String) -> Result\<String, String\> | !Postgres |
| Postgres.infer_table_raw | (table: String) -> Result\<String, String\> | !Postgres |

#### Snowflake（2 関数）
| 名前 | シグネチャ | effects |
|---|---|---|
| Snowflake.execute_raw | (sql: String, params: String) -> Result\<Unit, String\> | !Snowflake |
| Snowflake.query_raw | (sql: String, params: String) -> Result\<String, String\> | !Snowflake |

#### Http（3 関数）
| 名前 | シグネチャ | effects |
|---|---|---|
| Http.get_raw | (url: String, headers: String) -> Result\<String, String\> | !Http |
| Http.post_raw | (url: String, headers: String, body: String) -> Result\<String, String\> | !Http |
| Http.serve_raw | (port: Int, routes: List\<Map\<String,String\>\>, handler: String) -> Unit | !Http |

#### Llm（3 関数）
| 名前 | シグネチャ | effects |
|---|---|---|
| Llm.complete_raw | (prompt: String) -> Result\<String, String\> | !Llm |
| Llm.chat_raw | (messages: String) -> Result\<String, String\> | !Llm |
| Llm.extract_raw | (schema: String, text: String) -> Result\<String, String\> | !Llm |

---

## Phase B — `cmd_doc_builtins` 関数

### B-1: Markdown 出力

```rust
fn render_builtins_markdown(primitives: &[BuiltinPrimitive]) -> String {
    let mut out = String::from("# Favnir Built-in Primitives\n\n");
    // namespace ごとにグループ化
    let mut current_ns = "";
    for p in primitives {
        if p.namespace != current_ns {
            current_ns = p.namespace;
            out.push_str(&format!("## {}\n\n", p.namespace));
        }
        out.push_str(&format!("### {}\n", p.name));
        out.push_str(&format!("`{}`", p.signature));
        if !p.effects.is_empty() {
            let effects: Vec<_> = p.effects.iter().collect();
            out.push_str(&format!(" {}", effects.join(" ")));
        }
        out.push('\n');
        out.push('\n');
        out.push_str(p.description);
        out.push_str("\n\n---\n\n");
    }
    out
}
```

### B-2: JSON 出力

`serde_json::to_string_pretty(&primitives)` で出力。

### B-3: `cmd_doc_builtins(format: &str, out: Option<&str>)` を実装

```rust
pub fn cmd_doc_builtins(format: &str, out: Option<&str>) {
    let prims = builtin_primitives();
    let content = match format {
        "json" => serde_json::to_string_pretty(&prims).expect("json"),
        _ => render_builtins_markdown(&prims),
    };
    match out {
        Some(path) => std::fs::write(path, &content).expect("write"),
        None => print!("{}", content),
    }
}
```

---

## Phase C — `cmd_explain_code` 関数

### C-1: エラーコード説明マップを定義

`driver.rs` に静的な `HashMap` または `match` で
E0001〜E0018 / W001〜W007 の説明文を定義する。

```rust
pub fn cmd_explain_code(code: &str) {
    let explanation = match code {
        "E0001" => "E0001: Undefined variable\n\n...",
        "E0018" => "E0018: Variable already bound\n\n...",
        "W006"  => "W006: Discarding Result value\n\n...",
        _ => {
            eprintln!("unknown error code: {}", code);
            process::exit(1);
        }
    };
    println!("{}", explanation);
}
```

各説明文には:
- コード + タイトル
- 背景（日本語）
- 修正例（コードブロック、誤・正）
- 関連コード

を含める。

---

## Phase D — main.rs の変更

### D-1: `fav doc --builtins` の追加

既存の `Some("doc") =>` 分岐に `--builtins` の分岐を追加:

```rust
Some("doc") => {
    let builtins = args.iter().any(|a| a == "--builtins");
    if builtins {
        let format = args.windows(2)
            .find(|w| w[0] == "--format")
            .map(|w| w[1].as_str())
            .unwrap_or("markdown");
        let out = args.windows(2)
            .find(|w| w[0] == "--out")
            .map(|w| w[1].as_str());
        cmd_doc_builtins(format, out);
    } else {
        // 既存の cmd_doc(path, out_dir)
    }
}
```

### D-2: `fav explain <code>` の追加

既存の `Some("explain") =>` 分岐に `cmd_explain_code` を追加:

```rust
Some("explain") => {
    // 既存の --lineage フロー
    if args.iter().any(|a| a == "--lineage") { ... }
    // 既存の compiler フロー
    else if args.get(2).map_deref() == Some("compiler") { ... }
    // 新規: エラーコード説明
    else if let Some(code) = args.get(2) {
        cmd_explain_code(code);
    } else {
        eprintln!("usage: fav explain <code>");
        process::exit(1);
    }
}
```

---

## Phase E — テスト追加

`driver.rs` の `v12700_tests` モジュールに以下を追加:

```rust
#[cfg(test)]
mod v12700_tests {
    fn doc_builtins_json_is_array()              { ... }
    fn doc_builtins_csv_parse_raw()              { ... }
    fn doc_builtins_postgres_returns_result()    { ... }
    fn doc_builtins_markdown_has_namespace_header() { ... }
    fn explain_e0018_output()                    { ... }
    fn explain_w006_output()                     { ... }
    fn version_is_12_7_0()                       { ... }
}
```

`cmd_explain_code` の未知コードテストは `process::exit(1)` を呼ぶため
通常の `#[test]` では検証困難。ロジック部分（マッチなし時の返却値）を
関数に切り出してテストする。

---

## Phase F — バージョン更新・コミット

- `fav/Cargo.toml` version → `"12.7.0"`
- `cargo test` 全通過確認
- `git commit -m "feat: v12.7.0 — fav doc --builtins + fav explain <code>"`
- `git push`

---

## 実装上の注意

### 1. 静的スライスの lifetime

`BuiltinPrimitive` の `effects` フィールドは `&'static [&'static str]` 型。
Rust のコンパイル時定数として定義できる。

```rust
const IO_PRINTLN: BuiltinPrimitive = BuiltinPrimitive {
    effects: &["!IO"],
    ...
};
```

または `Vec<&'static str>` にして実行時に生成してもよい。
`serde::Serialize` が `&'static [&'static str]` に対応していることを確認。

### 2. 既存の `fav explain --lineage` との共存

`fav explain --lineage <file>` は既存の機能。
新機能 `fav explain E0018` はコード引数（`--` で始まらない）で区別可能。
`args.get(2)` が `--lineage` でなく `--` で始まらない文字列なら新機能。

### 3. 出力の冪等性

`fav doc --builtins` の出力は常に同じ（static table）。
CI で `fav doc --builtins --format json | jq .` が通ることを確認できる。

### 4. serde の `&'static [&'static str]` への対応

`serde::Serialize` は `&[T]` に対応しているが、
`#[derive(Serialize)]` で `effects: &'static [&'static str]` を使う場合、
`serde(borrow)` アノテーションが必要になる可能性がある。
問題が出た場合は `Vec<&'static str>` に変更。
