# Favnir v11.3.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: Emitter に import フラグ追加 + header 更新
    ↓
Phase B: IO プリミティブ実変換（read_file / write_file）
    ↓
Phase C: Csv プリミティブ実変換
    ↓
Phase D: Schema ヘルパー生成（adapt / to_json_array）
    ↓
Phase E: Json プリミティブ実変換
    ↓
Phase F: ヘルパー関数を先頭に emit
    ↓
Phase G: テスト（v11300_tests）
    ↓
Phase H: バージョン更新・コミット
```

---

## Phase A — import フラグ

`emit_python.rs` の `Emitter` 構造体に以下を追加:

```rust
struct Emitter {
    // ...既存フィールド...
    needs_csv:  bool,
    needs_json: bool,
}
```

`emit_program` の header 出力部分:

```rust
// 既存
out.push_str("import sys\n");
// 追加
if self.needs_csv  { out.push_str("import csv as _csv_mod\nimport io as _io_mod\n"); }
if self.needs_json { out.push_str("import json as _json_mod\n"); }
```

---

## Phase B — IO プリミティブ

`emit_apply` の `("IO", name)` フォールバックを個別ケースに展開:

```rust
("IO", "read_file_raw") if a.len() == 1 => {
    self.needs_io_helpers = true;
    format!("_io_read_file_raw({})", a[0])
}
("IO", "write_file_raw") if a.len() == 2 => {
    self.needs_io_helpers = true;
    format!("_io_write_file_raw({}, {})", a[0], a[1])
}
```

生成ヘルパー関数（`emit_io_helpers`）:

```python
def _io_read_file_raw(path: str):
    try:
        with open(path, encoding='utf-8') as _f:
            return Ok(_f.read())
    except Exception as _e:
        return Err(str(_e))

def _io_write_file_raw(path: str, text: str):
    try:
        with open(path, 'w', encoding='utf-8') as _f:
            _f.write(text)
        return Ok(None)
    except Exception as _e:
        return Err(str(_e))
```

---

## Phase C — Csv プリミティブ

```rust
("Csv", "parse_raw") if a.len() == 3 => {
    self.needs_csv = true;
    self.needs_csv_helpers = true;
    format!("_csv_parse_raw({}, {}, {})", a[0], a[1], a[2])
}
```

生成ヘルパー関数（`emit_csv_helpers`）:

```python
def _csv_parse_raw(text: str, sep: str, has_header: bool):
    try:
        _r = _csv_mod.DictReader(_io_mod.StringIO(text), delimiter=sep)
        return Ok([dict(_row) for _row in _r])
    except Exception as _e:
        return Err(str(_e))
```

---

## Phase D — Schema ヘルパー

### `_SCHEMA_REGISTRY` の生成

`emit_program` の型定義処理後に、登録済み dataclass 名を収集して辞書を emit:

```rust
// emit_program 内
let mut type_names: Vec<String> = vec![];
for item in &program.items {
    if let Item::TypeDef(td) = item {
        type_names.push(td.name.clone());
    }
}
if !type_names.is_empty() {
    self.line("_SCHEMA_REGISTRY = {");
    for name in &type_names {
        self.line(&format!("    \"{}\": {},", name, name));
    }
    self.line("}");
}
```

### `_schema_adapt` ヘルパー

```python
def _schema_adapt(raw, type_name: str):
    _TYPE_CAST = {"str": str, "int": int, "float": float}
    try:
        _cls = _SCHEMA_REGISTRY[type_name]
        _fields = _cls.__dataclass_fields__
        def _cast(k, v):
            _ann = _fields[k].type if hasattr(_fields[k], 'type') else 'str'
            _ann_str = _ann if isinstance(_ann, str) else getattr(_ann, '__name__', 'str')
            return _TYPE_CAST.get(_ann_str, str)(v)
        return Ok([_cls(**{k: _cast(k, v) for k, v in _row.items() if k in _fields})
                   for _row in raw])
    except Exception as _e:
        return Err(str(_e))
```

### `_schema_to_json_array`

```python
def _schema_to_json_array(rows, type_name: str) -> str:
    return _json_mod.dumps([asdict(_r) for _r in rows])
```

---

## Phase E — Json プリミティブ

```rust
("Json", "encode_raw") | ("Json", "write_raw") if a.len() == 1 => {
    self.needs_json = true;
    format!("_json_mod.dumps({})", a[0])
}
("Json", "decode_raw") | ("Json", "parse_raw") if a.len() == 1 => {
    self.needs_json = true;
    self.needs_json_helpers = true;
    format!("_json_decode_raw({})", a[0])
}
```

生成ヘルパー:

```python
def _json_decode_raw(s: str):
    try:
        return Ok(_json_mod.loads(s))
    except Exception as _e:
        return Err(str(_e))
```

---

## Phase F — ヘルパー関数の emit 順序

`emit_program` の出力順序:

```
1. imports (sys / csv / json 等)
2. Ok / Err クラス定義
3. @dataclass 定義（既存）
4. _SCHEMA_REGISTRY 辞書（型定義がある場合）
5. _io_read_file_raw / _io_write_file_raw（使用時のみ）
6. _csv_parse_raw（使用時のみ）
7. _schema_adapt / _schema_to_json_array（使用時のみ）
8. _json_decode_raw（使用時のみ）
9. fn / stage / seq 定義（既存）
10. if __name__ == "__main__": main()（既存）
```

---

## Phase G — テスト（8件）

| テスト名 | 検証内容 |
|---|---|
| `transpile_io_read_file` | `IO.read_file_raw` → `_io_read_file_raw` ヘルパー含む |
| `transpile_io_write_file` | `IO.write_file_raw` → `_io_write_file_raw` ヘルパー含む |
| `transpile_csv_parse_raw` | `Csv.parse_raw` → `_csv_parse_raw` + `import csv` ヘルパー含む |
| `transpile_schema_registry` | type 定義 → `_SCHEMA_REGISTRY` 辞書生成 |
| `transpile_schema_adapt` | `Schema.adapt` → `_schema_adapt` ヘルパー含む |
| `transpile_schema_to_json_array` | `Schema.to_json_array` → `_json_mod.dumps([asdict...])` |
| `transpile_json_encode` | `Json.encode_raw` → `_json_mod.dumps(...)` |
| `transpile_analyze_fav_smoke` | `analyze.fav` をトランスパイルして `py_compile` が通る |

---

## Phase H — バージョン更新

- `fav/Cargo.toml` version → `"11.3.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
