# Favnir v12.7.0 Tasks

Date: 2026-06-08
Theme: `fav doc --builtins` + `fav explain <code>`

---

## Phase A — `BuiltinPrimitive` 静的テーブル

- [x] A-1: `BuiltinPrimitive` 構造体を driver.rs に定義（namespace / name / signature / effects / returns_result / description）
- [x] A-2: `builtin_primitives() -> Vec<BuiltinPrimitive>` を実装
  - IO（12 関数）: println / print / println_int / println_float / println_bool / read_line / read_file_raw / write_file_raw / make_dir_raw / argv / timestamp / sleep_ms
  - Csv（2 関数）: parse_raw / to_string_raw
  - Schema（3 関数）: to_json_array / adapt / validate
  - Json（3 関数）: encode_raw / decode_raw / pretty_raw
  - Gen（5 関数）: uuid / uuid_v7 / nano_id / one_raw / hint_one_raw
  - AWS（4 関数）: s3_get_raw / s3_put_raw / sqs_send_raw / sqs_receive_raw
  - Postgres（3 関数）: execute_raw / query_raw / infer_table_raw
  - Snowflake（2 関数）: execute_raw / query_raw
  - Http（3 関数）: get_raw / post_raw / serve_raw
  - Llm（3 関数）: complete_raw / chat_raw / extract_raw

---

## Phase B — `cmd_doc_builtins` 実装

- [x] B-1: `render_builtins_markdown(primitives: &[BuiltinPrimitive]) -> String` を実装
  - namespace ごとに `## Namespace` ヘッダ
  - 各関数に `### Name` + シグネチャ + description
- [x] B-2: JSON 出力パス（`serde_json::to_string_pretty`）を実装
- [x] B-3: `cmd_doc_builtins(format: &str, out: Option<&str>)` を実装
  - `format="json"` → JSON、それ以外 → Markdown
  - `out=Some(path)` → ファイル書き出し、`None` → stdout

---

## Phase C — `cmd_explain_code` 実装

- [x] C-1: `get_explain_text(code: &str) -> Option<&'static str>` を実装
  - E0001〜E0018 の説明文（コード + タイトル + 背景 + 修正例 + 関連）
  - W001〜W007 の説明文
- [x] C-2: `cmd_explain_code(code: &str)` を実装
  - `get_explain_text` が `None` → stderr に `"unknown error code: {code}"` + exit 1
  - `Some(text)` → stdout に出力

---

## Phase D — main.rs の変更

- [x] D-1: `Some("doc")` 分岐に `--builtins` フラグを追加
  - `--builtins` → `cmd_doc_builtins(format, out)`
  - それ以外 → 既存の `cmd_doc(path, out_dir)`（変更なし）
- [x] D-2: `Some("explain")` 分岐に `cmd_explain_code` を追加
  - `--lineage` / `compiler` フロー → 既存のまま
  - 上記以外で引数が `E` または `W` で始まる文字列 → `cmd_explain_code(code)`

---

## Phase E — テスト追加

- [x] E-1: `doc_builtins_json_is_array` — JSON 出力が配列であること
- [x] E-2: `doc_builtins_csv_parse_raw` — `Csv.parse_raw` エントリが JSON に存在すること
- [x] E-3: `doc_builtins_postgres_returns_result` — `Postgres.execute_raw` の `returns_result` が `true`
- [x] E-4: `doc_builtins_markdown_has_namespace_headers` — Markdown に `## IO` / `## Csv` / `## Postgres` が含まれる
- [x] E-5: `explain_e0018_output` — E0018 の説明に "already bound" または "束縛" が含まれる
- [x] E-6: `explain_w006_output` — W006 の説明に "Result" が含まれる
- [x] E-7: `explain_unknown_returns_none` — 未知コードで `get_explain_text` が `None` を返す
- [x] E-8: `version_is_12_7_0`
- [x] E-9: `cargo test v12700` — 8 件通過確認

---

## Phase F — バージョン更新・コミット

- [x] F-1: `fav/Cargo.toml` version → `"12.7.0"`
- [x] F-2: `cargo test` — 全通過（1400 件）
- [x] F-3: `git commit -m "feat: v12.7.0 — fav doc --builtins + fav explain <code>"`
- [x] F-4: `git push` → CI 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `fav doc --builtins` で全 Namespace の Markdown が出る | ✅ |
| `fav doc --builtins --format json` で JSON 配列が出る | ✅ |
| `fav explain E0018` で修正方法付きの説明が出る | ✅ |
| `fav explain W006` で Result 廃棄の警告説明が出る | ✅ |
| 未知コードで exit 1 | ✅ |
| `cargo test v12700` 8 件通過 | ✅ |
| `cargo test` 全通過 | ✅ 1400 件 |
