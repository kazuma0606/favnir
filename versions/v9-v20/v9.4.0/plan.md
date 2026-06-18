# Favnir v9.4.0 実装計画 — json・csv・gen Rune 拡張 + W004

作成日: 2026-06-01

---

## 前提

- ベース: v9.3.0 (1173 tests)
- Rust 変更: `vm.rs` のみ（`Gen.uuid_raw` / `Gen.uuid_v7_raw` / `Gen.nano_id_raw` / `Json.pretty_raw` の追加）
- Rune ファイルパス: `C:\Users\yoshi\favnir\runes\`

### 既存 Rune の確認（変更前）

| Rune | 既存関数 |
|---|---|
| `runes/json/json.fav` | `parse<T>`, `parse_list<T>`, `write<T>`, `write_list<T>` |
| `runes/csv/csv.fav` | `parse<T>`, `write<T>`, `parse_positional<T>`, `parse_with_opts<T>` |
| `runes/gen/primitives.fav` | `int_val`, `float_val`, `bool_val`, `string_val`, `choice` |

### vm.rs 既存 builtin（確認済み）

- `Json.encode` / `Json.encode_pretty` — VMValue → JSON 文字列（実装済み）
- `Json.parse_raw` / `Json.parse_array_raw` — JSON 文字列 → VMValue（実装済み）
- `Csv.parse_raw` / `Csv.write_raw` — CSV 文字列 ↔ 行リスト（実装済み）
- `Gen.uuid_raw` / `Gen.uuid_v7_raw` / `Gen.nano_id_raw` — **未実装（要追加）**
- `Json.pretty_raw` — **未実装（要追加）**

---

## Phase A: vm.rs — 新規 builtin 追加

### A-1: `Json.pretty_raw` builtin

`Json.encode_pretty` は VMValue を整形 JSON にするが、
`Json.pretty_raw` は **JSON 文字列を受け取り** 整形済み文字列を返す。

```rust
"Json.pretty_raw" => {
    let s = vm_string(args.into_iter().next()...)?;
    let v: serde_json::Value = serde_json::from_str(&s)
        .map_err(|e| format!("Json.pretty_raw: invalid JSON: {}", e))?;
    Ok(VMValue::Str(serde_json::to_string_pretty(&v).unwrap()))
}
```

### A-2: `Gen.uuid_raw` builtin

```rust
"Gen.uuid_raw" => {
    Ok(VMValue::Str(uuid::Uuid::new_v4().to_string()))
}
```

### A-3: `Gen.uuid_v7_raw` builtin

UUID v7 は `uuid` crate 1.6+ で `Uuid::now_v7()` が利用可能。
Cargo.toml の `uuid` crate バージョンを確認し、必要なら `features = ["v4", "v7"]` を追加。

```rust
"Gen.uuid_v7_raw" => {
    Ok(VMValue::Str(uuid::Uuid::now_v7().to_string()))
}
```

### A-4: `Gen.nano_id_raw` builtin

`rand` crate（既存依存）を使い URL-safe 文字セット（`[a-zA-Z0-9_-]`）から n 文字生成。

```rust
"Gen.nano_id_raw" => {
    let n = vm_int(args.into_iter().next()...)? as usize;
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
    let mut rng = rand::thread_rng();
    let id: String = (0..n)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect();
    Ok(VMValue::Str(id))
}
```

---

## Phase B: json Rune 拡張（`runes/json/json.fav`）

### B-1: `decode<T>` を追加

`parse<T>` の簡易版。SchemaError ではなく String のエラーを返す。

```favnir
public fn decode<T>(text: String) -> Result<T, String> {
    match Json.parse_raw(text) {
        Err(e) => Result.err(e)
        Ok(raw) =>
            match Schema.adapt_one(raw, type_name_of<T>()) {
                Err(e) => Result.err(Schema.error_message(e))
                Ok(v)  => Result.ok(v)
            }
    }
}
```

> 注: `Schema.error_message` が存在するか確認。なければ `"decode failed"` の固定文字列で代替。
> シンプルにするなら `decode` = `parse` のエイリアスとして `SchemaError` ではなく `String` 型変換を行う。

### B-2: `encode<T>` を追加

`write<T>` の別名。`Json.encode` builtin を呼ぶ。

```favnir
public fn encode<T>(value: T) -> String {
    Json.encode(value)
}
```

### B-3: `pretty` を追加

```favnir
public fn pretty(text: String) -> String {
    Json.pretty_raw(text)
}
```

---

## Phase C: csv Rune 拡張（`runes/csv/csv.fav`）

### C-1: `read<T>` を追加

ファイルパスから直接読んで `parse<T>` を呼ぶ。純粋 Favnir で実装できる。

```favnir
public fn read<T>(path: String) -> Result<List<T>, String> !IO {
    match IO.read_file_raw(path) {
        Err(e) => Result.err(String.concat("read error: ", e))
        Ok(text) =>
            match parse<T>(text) {
                Err(e) => Result.err(Schema.error_message(e))
                Ok(rows) => Result.ok(rows)
            }
    }
}
```

> 注: `SchemaError → String` 変換に `Schema.error_message` が使えない場合は
> `parse<T>` を直接 `Csv.parse_raw` + `Schema.adapt` に展開して String エラーで統一する。

### C-2: `write_file<T>` を追加

```favnir
public fn write_file<T>(path: String, rows: List<T>) -> Unit !IO {
    bind csv_text <- write<T>(rows)
    match IO.write_file_raw(path, csv_text) {
        Err(e) => IO.write_stderr_raw(String.concat("write_file error: ", e))
        Ok(_)  => ()
    }
}
```

---

## Phase D: gen Rune 拡張（`runes/gen/primitives.fav`）

### D-1: `uuid` / `uuid_v7` / `nano_id` を追加

```favnir
public fn uuid() -> String !Gen {
    Gen.uuid_raw()
}

public fn uuid_v7() -> String !Gen {
    Gen.uuid_v7_raw()
}

public fn nano_id(n: Int) -> String !Gen {
    Gen.nano_id_raw(n)
}
```

### D-2: checker.rs に型シグネチャを追加

`src/checker.rs` の gen 関数テーブルに追加:

```rust
("gen.uuid",    vec![],              "String"),
("gen.uuid_v7", vec![],              "String"),
("gen.nano_id", vec!["Int"],         "String"),
```

### D-3: checker.fav に型シグネチャを追加

`fav/self/checker.fav` の gen 関数セクションに追加（gen_fn 相当の箇所）:

```
"gen.uuid"    → "String"        (引数なし)
"gen.uuid_v7" → "String"        (引数なし)
"gen.nano_id" → "Int|String"    (Int → String)
```

---

## Phase E: W004 lint ルール（`fav/self/compiler.fav`）

### E-1: `lint_fn_w004` を追加

`fn` のパラメータ数が 4 以上で警告を出す。
`List.length` は compiler.fav 内から呼べることを確認済み（gen_demo で使用実績あり）。

```favnir
fn lint_fn_w004(fd: FnDef) -> List<LintWarning> {
    bind n <- List.length(fd.params)
    if n >= 4 {
        List.singleton(LintWarning {
            code:    "W004"
            message: String.concat("fn ", String.concat(fd.name,
                         String.concat(" の引数が ",
                         String.concat(Int.to_string(n),
                         " 個です。レコード型へのまとめを検討してください"))))
            name:    fd.name
        })
    } else {
        List.empty()
    }
}
```

### E-2: `lint_fn` を更新

既存 `lint_fn` に W004 チェックを追加:

```favnir
fn lint_fn(fd: FnDef) -> List<LintWarning> {
    bind w003s <- lint_fn_w003(fd)
    bind w004s <- lint_fn_w004(fd)
    List.concat(w003s, w004s)
}
```

> 注: `lint_fn` が既に W003 を処理しているならそこに追記。構造を確認してから。

### E-3: `lint_item` の `IFn` ブランチを確認

`lint_item` で `IFn(fd) => lint_fn(fd)` が呼ばれているはずなので、`lint_fn` の変更だけで連動する。

---

## Phase F: 統合テスト（`fav/src/driver.rs`）

### F-1: `json_rune_v940_tests` モジュール

```rust
#[test]
fn json_decode_returns_record() { ... }  // json.decode で Record が返る

#[test]
fn json_encode_roundtrip() { ... }       // encode → decode のラウンドトリップ

#[test]
fn json_pretty_formats() { ... }         // pretty で改行・インデントが付く
```

### F-2: `csv_rune_v940_tests` モジュール

```rust
#[test]
fn csv_read_parses_file() { ... }        // tempdir に CSV 書いて read<T> で読む

#[test]
fn csv_write_file_creates() { ... }      // write_file<T> でファイルが作成される
```

### F-3: `gen_rune_v940_tests` モジュール

```rust
#[test]
fn gen_uuid_returns_v4() { ... }         // 36文字・ハイフン区切り

#[test]
fn gen_uuid_v7_returns_v7() { ... }      // 36文字・ハイフン区切り

#[test]
fn gen_nano_id_len() { ... }             // nano_id(12) が 12 文字
```

### F-4: `lint_w004_tests` モジュール

```rust
#[test]
fn lint_w004_too_many_params() { ... }   // fn f(a,b,c,d: Int) → W004 検出

#[test]
fn lint_w004_three_params_ok() { ... }   // fn f(a,b,c: Int) → 警告なし
```

---

## Phase G: self-check + Bootstrap 検証

- G-1: `fav check fav/self/compiler.fav` — self-check 通過
- G-2: `cargo test bootstrap` — `bytecode_A == bytecode_B` 維持確認
- G-3: `cargo test` — 全件通過（1183 件以上）

---

## Phase H: バージョン更新

- H-1: `fav/Cargo.toml` version → `"9.4.0"`
- H-2: `fav/self/cli.fav` バージョン文字列 → `"9.4.0"`
- H-3: `versions/v9.4.0/tasks.md` 完了チェック
- H-4: `memory/MEMORY.md` に v9.4.0 完了を記録
- H-5: commit

---

## 実装上の注意

### `List.length` の使用

`List.length` は `compiler.fav` から呼び出し可能（vm.rs の `"List.length"` builtin 実装済み、gen_demo でも使用済み）。

### uuid v7 の Cargo.toml 確認

`Cargo.toml` の uuid 依存が `features = ["v4"]` のみの場合は `"v7"` を追記する:
```toml
uuid = { version = "1", features = ["v4", "v7"] }
```

### `Schema.error_message` の存在確認

`runes/csv/csv.fav` / `runes/json/json.fav` で使用している `Schema.error_message` が
存在するか `runes/` を確認する。存在しない場合は固定文字列または `Debug.show` で代替。

### W004 の `lint_fn` 構造確認

実装前に `compiler.fav` の `lint_fn` 関数を確認し、W003 処理との統合方法を見てから追加する。

---

## テスト数見込み

| フェーズ | テスト数 |
|---|---|
| F-1 json tests | 3 件 |
| F-2 csv tests | 2 件 |
| F-3 gen tests | 3 件 |
| F-4 W004 tests | 2 件 |
| 既存（v9.3.0） | 1173 件 |
| **合計** | **1183 件以上** |
