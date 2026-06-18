# Favnir v11.3.0 Tasks

Date: 2026-06-06
Theme: IO エフェクト → Python 標準ライブラリ変換

---

## Phase A — Emitter import フラグ追加

- [ ] A-1: `Emitter` 構造体に `needs_csv: bool` / `needs_json: bool` / `needs_io_helpers: bool` / `needs_csv_helpers: bool` / `needs_json_helpers: bool` / `needs_schema_helpers: bool` フィールド追加
- [ ] A-2: `emit_program` の header 出力に import フラグチェックを追加
  - `needs_csv` → `import csv as _csv_mod` / `import io as _io_mod`
  - `needs_json` → `import json as _json_mod`

---

## Phase B — IO プリミティブ実変換

- [ ] B-1: `emit_apply` の `("IO", name)` フォールバックより前に個別ケース追加
  - `("IO", "read_file_raw")` → `_io_read_file_raw(path)` + `needs_io_helpers = true`
  - `("IO", "write_file_raw")` → `_io_write_file_raw(path, text)` + `needs_io_helpers = true`
- [ ] B-2: `emit_io_helpers()` メソッド追加
  - `_io_read_file_raw`: try/except + `Ok`/`Err` ラップ
  - `_io_write_file_raw`: try/except + `Ok(None)`/`Err` ラップ
- [ ] B-3: `emit_program` の dataclass 定義後に `emit_io_helpers()` 呼び出し（`needs_io_helpers` 時のみ）

---

## Phase C — Csv プリミティブ実変換

- [ ] C-1: `emit_apply` に `("Csv", "parse_raw")` の実変換追加
  - `_csv_parse_raw(text, sep, has_header)` + `needs_csv = needs_csv_helpers = true`
- [ ] C-2: `emit_csv_helpers()` メソッド追加
  - `_csv_parse_raw`: `csv.DictReader(io.StringIO(text), delimiter=sep)` + try/except
- [ ] C-3: `emit_program` に `emit_csv_helpers()` 呼び出し（`needs_csv_helpers` 時のみ）

---

## Phase D — Schema ヘルパー生成

- [ ] D-1: `emit_program` で型定義名を収集し `_SCHEMA_REGISTRY` 辞書を emit
  - 型定義が 1 件以上あり `needs_schema_helpers` のとき出力
- [ ] D-2: `emit_apply` に `("Schema", "adapt")` の実変換追加
  - `_schema_adapt(raw, type_name)` + `needs_schema_helpers = needs_json = true`
- [ ] D-3: `emit_apply` に `("Schema", "to_json_array")` の実変換追加
  - `_schema_to_json_array(rows, type_name)` + `needs_schema_helpers = needs_json = true`
- [ ] D-4: `emit_schema_helpers()` メソッド追加
  - `_schema_adapt`: `_SCHEMA_REGISTRY` 参照 + `__dataclass_fields__` で型キャスト + try/except
  - `_schema_to_json_array`: `_json_mod.dumps([asdict(_r) for _r in rows])`
- [ ] D-5: `emit_program` に `emit_schema_helpers()` 呼び出し（`needs_schema_helpers` 時のみ）

---

## Phase E — Json プリミティブ実変換

- [ ] E-1: `emit_apply` の `("Json", name)` フォールバックより前に個別ケース追加
  - `("Json", "encode_raw")` / `("Json", "write_raw")` → `_json_mod.dumps(val)` + `needs_json = true`
  - `("Json", "decode_raw")` / `("Json", "parse_raw")` → `_json_decode_raw(s)` + `needs_json = needs_json_helpers = true`
  - `("Json", "write_array_raw")` → `_json_mod.dumps(val)` + `needs_json = true`
- [ ] E-2: `emit_json_helpers()` メソッド追加
  - `_json_decode_raw`: `json.loads(s)` + try/except + `Ok`/`Err` ラップ
- [ ] E-3: `emit_program` に `emit_json_helpers()` 呼び出し（`needs_json_helpers` 時のみ）

---

## Phase F — emit 順序整備

- [ ] F-1: `emit_program` のヘルパー出力順序を整理
  1. imports
  2. Ok/Err クラス
  3. @dataclass 定義
  4. `_SCHEMA_REGISTRY`（型定義あり + schema helpers 使用時）
  5. `_io_*` helpers
  6. `_csv_*` helpers
  7. `_schema_*` helpers
  8. `_json_*` helpers
  9. fn / stage / seq 定義
  10. `if __name__ == "__main__":`

---

## Phase G — テスト（8件）

- [ ] G-1: `v11300_tests` モジュール追加
  - [ ] `transpile_io_read_file` — `IO.read_file_raw` → ヘルパー定義 + 呼び出し
  - [ ] `transpile_io_write_file` — `IO.write_file_raw` → ヘルパー定義 + 呼び出し
  - [ ] `transpile_csv_parse_raw` — `Csv.parse_raw` → `import csv` + ヘルパー定義
  - [ ] `transpile_schema_registry` — type 定義 → `_SCHEMA_REGISTRY` 辞書生成
  - [ ] `transpile_schema_adapt` — `Schema.adapt` → `_schema_adapt` ヘルパー含む
  - [ ] `transpile_schema_to_json_array` — `Schema.to_json_array` → `_json_mod.dumps([asdict...])` 含む
  - [ ] `transpile_json_encode` — `Json.encode_raw` → `_json_mod.dumps(...)` 含む
  - [ ] `transpile_analyze_fav_smoke` — `analyze.fav` 全体をトランスパイルして `py_compile` 通過
- [ ] G-2: `cargo test v11300 --lib` — 8 件通過
- [ ] G-3: `cargo test --lib` — 全件通過

---

## Phase H — バージョン更新 + コミット

- [ ] H-1: `fav/Cargo.toml` version → `"11.3.0"`
- [ ] H-2: `cargo build` で `Cargo.lock` 更新
- [ ] H-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `IO.read_file_raw(path)` → `_io_read_file_raw` ヘルパー（try/except + Ok/Err）| |
| `Csv.parse_raw(...)` → `_csv_parse_raw` ヘルパー + `import csv` | |
| `Schema.adapt(raw, "T")` → `_schema_adapt` + `_SCHEMA_REGISTRY` | |
| `Schema.to_json_array(rows, "T")` → `json.dumps([asdict(r) for r in rows])` | |
| `Json.encode_raw(v)` → `_json_mod.dumps(v)` | |
| `analyze.fav` トランスパイル → `uv run python -m py_compile` 通過 | |
| `cargo test v11300 --lib` 8 件通過 | |
| `cargo test --lib` 全件通過 | |
