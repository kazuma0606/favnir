# Favnir v11.3.0 仕様書

作成日: 2026-06-06
テーマ: IO エフェクト → Python 標準ライブラリ変換

---

## 背景と目的

v11.1.0〜v11.2.0 で AST → Python の基盤（型定義・fn・stage・seq）が完成した。
v11.3.0 では `!IO` エフェクトを伴う VM プリミティブ（ファイル I/O・CSV・JSON・Schema）を
Python 標準ライブラリに変換する。現状はすべて `_csv_parse_raw(...)` / `_schema_adapt(...)` 等の
プレースホルダー出力になっているため、これを実変換に置き換える。

**目標**: airgap の `analyze.fav` を `fav transpile --target python` で変換し、
`uv run python -m py_compile` が通る Python ファイルが生成されること。

---

## 変換対応表

### IO プリミティブ

| Fav | Python | 戻り型 |
|---|---|---|
| `IO.read_file_raw(path)` | `open(path, encoding='utf-8').read()` — try/except で `Ok`/`Err` ラップ | `Result<String, String>` |
| `IO.write_file_raw(path, text)` | `open(path, 'w', encoding='utf-8').write(text)` — try/except | `Result<Unit, String>` |
| `IO.println(s)` | `print(s)` | （v11.1.0 実装済み） |
| `IO.argv()` | `sys.argv[1:]` | （v11.2.0 実装済み） |

### CSV プリミティブ

| Fav | Python | 戻り型 |
|---|---|---|
| `Csv.parse_raw(text, sep, has_header)` | `csv.DictReader(io.StringIO(text), delimiter=sep)` — try/except で `Ok`/`Err` ラップ | `Result<List<Map<String,String>>, String>` |

### Schema ヘルパー

| Fav | Python | 備考 |
|---|---|---|
| `Schema.adapt(raw, "T")` | 型名から dataclass を検索してリスト変換 | `_SCHEMA_REGISTRY` グローバル辞書を emit |
| `Schema.to_json_array(rows, "T")` | `json.dumps([asdict(r) for r in rows])` | |

### JSON プリミティブ

| Fav | Python | 戻り型 |
|---|---|---|
| `Json.encode_raw(val)` | `json.dumps(val)` | `String` |
| `Json.decode_raw(s)` | `json.loads(s)` — try/except で `Ok`/`Err` ラップ | `Result<Any, String>` |
| `Json.write_raw(m)` | `json.dumps(m)` | `String` |

---

## 生成コード設計

### `IO.read_file_raw` の生成

```python
# インラインヘルパーとして先頭に emit（一度だけ）
def _io_read_file_raw(path: str):
    try:
        with open(path, encoding='utf-8') as _f:
            return Ok(_f.read())
    except Exception as _e:
        return Err(str(_e))
```

### `Csv.parse_raw` の生成

```python
import csv as _csv_mod
import io as _io_mod

def _csv_parse_raw(text: str, sep: str, has_header: bool):
    try:
        _r = _csv_mod.DictReader(_io_mod.StringIO(text), delimiter=sep)
        return Ok([dict(_row) for _row in _r])
    except Exception as _e:
        return Err(str(_e))
```

### `Schema.adapt` の生成

```python
# 型定義から自動生成
_SCHEMA_REGISTRY = {
    "TxnRow": TxnRow,
}

def _schema_adapt(raw, type_name: str):
    try:
        _cls = _SCHEMA_REGISTRY[type_name]
        return Ok([_cls(**{k: _cls.__dataclass_fields__[k].type(v)
                           if k in _cls.__dataclass_fields__ else v
                           for k, v in _row.items()
                           if k in _cls.__dataclass_fields__})
                   for _row in raw])
    except Exception as _e:
        return Err(str(_e))
```

※ 型キャスト: `str`→`str`, `int`→`int`, `float`→`float` のみ対応。`Bool` は `lambda v: v.lower() == "true"` で変換。

### `Schema.to_json_array` の生成

```python
def _schema_to_json_array(rows, type_name: str) -> str:
    return _json_mod.dumps([asdict(_r) for _r in rows])
```

---

## import 自動追加

| 使用 | 追加 import |
|---|---|
| `IO.read_file_raw` / `IO.write_file_raw` | — (`open` は builtin) |
| `Csv.parse_raw` | `import csv as _csv_mod` / `import io as _io_mod` |
| `Schema.to_json_array` / `Json.*` | `import json as _json_mod` |

実装: `Emitter` に `needs_csv: bool` / `needs_json: bool` フィールドを追加し、
`emit_apply` 内で使用を検出してフラグを立て、`emit_program` の header で出力。

---

## analyze.fav トランスパイル検証

`infra/e2e-demo/airgap/src/analyze.fav` を変換した Python が以下を満たすこと:

1. `uv run python -m py_compile <file>` — 構文エラーなし
2. `IO.read_file_raw` → `_io_read_file_raw(path)` ヘルパー含む
3. `Csv.parse_raw` → `_csv_parse_raw(...)` ヘルパー含む
4. `Schema.adapt` → `_schema_adapt(...)` + `_SCHEMA_REGISTRY` 含む
5. `Schema.to_json_array` → `_schema_to_json_array(...)` 含む
