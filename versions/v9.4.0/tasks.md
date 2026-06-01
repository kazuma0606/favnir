# Favnir v9.4.0 Tasks

Date: 2026-06-01
Theme: json・csv・gen Rune 拡張 + W004 lint ルール

---

## Phase A: vm.rs — 新規 builtin 追加

- [x] A-1: `Json.pretty_raw(s: String) -> String` を vm.rs に追加
  — `serde_json::from_str` → `serde_json::to_string_pretty` で整形
- [x] A-2: `Gen.uuid_raw() -> String` を vm.rs に追加
  — `uuid::Uuid::new_v4().to_string()`
- [x] A-3: `Gen.uuid_v7_raw() -> String` を vm.rs に追加
  — `uuid::Uuid::now_v7().to_string()`
  — `Cargo.toml` の `uuid` features に `"v7"` を追加
- [x] A-4: `Gen.nano_id_raw(n: Int) -> String` を vm.rs に追加
  — `rand` crate で `[a-zA-Z0-9_-]` から n 文字生成

---

## Phase B: json Rune 拡張（`runes/json/json.fav`）

- [x] B-1: `public fn encode<T>(value: T) -> String` を追加
  — `Schema.to_json(value, type_name_of<T>())` 直接呼び出し
- [x] B-2: `public fn decode<T>(text: String) -> Result<T, String>` を追加
  — `parse<T>` ベース、エラー型を `String` に変換
- [x] B-3: `public fn pretty(text: String) -> String` を追加
  — `Json.pretty_raw(text)` を呼ぶ
- [x] B-4: 動作確認（`json_pretty_raw_formats` 等テスト通過）

---

## Phase C: csv Rune 拡張（`runes/csv/csv.fav`）

- [x] C-1: `public fn read<T>(path: String) -> Result<List<T>, String> !IO` を追加
  — `IO.read_file_raw` → `Csv.parse_raw` → `Schema.adapt` を直接インライン
  — 注: 他のジェネリック関数（`parse<T>`）を呼ぶと Rust pipeline で stack overflow
- [x] C-2: `public fn write_file<T>(path: String, rows: List<T>) -> Result<Unit, String> !IO` を追加
  — `Schema.to_csv` を直接呼び出し（`write<T>` を呼ぶと同様に overflow）
- [x] C-3: エラー型変換は `Err(_) => Result.err("csv.read: schema error")` に簡略化
- [x] C-4: `col_annotation_maps_by_position` テスト通過確認

---

## Phase D: gen Rune 拡張（`runes/gen/primitives.fav`）

- [x] D-1: `public fn uuid() -> String !Gen` を追加 — `Gen.uuid_raw()`
- [x] D-2: `public fn uuid_v7() -> String !Gen` を追加 — `Gen.uuid_v7_raw()`
- [x] D-3: `public fn nano_id(n: Int) -> String !Gen` を追加 — `Gen.nano_id_raw(n)`
- [x] D-4: `src/checker.rs` に Gen v9.4.0 型シグネチャを追加
- [x] D-5: `fav/self/checker.fav` に `gen_fn` 追加、`builtin_ret_ty` / `ns_to_effect` に Gen を追加
  — `else if` 構文を使用（v8.10.0 以降対応）

---

## Phase E: W004 lint ルール（`fav/self/compiler.fav`）

- [x] E-1: `lint_fn` 関数の現在の実装を確認
- [x] E-2: `fn lint_fn_w004(fd: FnDef) -> List<LintWarning>` を追加
  — `List.length(fd.params) >= 4` → W004 警告を返す
- [x] E-3: `lint_fn` に `lint_fn_w004` を追加（`List.concat` で統合）
- [x] E-4: self-check 通過確認

---

## Phase F: 統合テスト（`fav/src/driver.rs`）

- [x] F-1〜F-7: `v940_tests` — json/gen/csv builtins テスト 7 件通過
- [x] F-8: `lint_w004_too_many_params` — fn 4 引数で W004 が検出されること
- [x] F-9: `lint_w004_three_params_ok` — fn 3 引数で W004 が検出されないこと
- [x] F-10: `cargo test v940` — 全件通過確認

---

## Phase G: self-check + Bootstrap 検証

- [x] G-1: `fav check fav/self/checker.fav` — self-check 通過
- [x] G-2: `cargo test bootstrap` — `bytecode_A == bytecode_B` 維持確認
- [x] G-3: `cargo test` — 1182 件通過

---

## Phase H: ドキュメント・バージョン更新

- [x] H-1: `fav/Cargo.toml` の `version` を `"9.4.0"` に更新
- [x] H-2: `fav/self/cli.fav` のバージョン文字列を `"9.4.0"` に更新
- [x] H-3: `versions/v9.4.0/tasks.md` 完了チェックを入れる（本ファイル）
- [x] H-4: `memory/MEMORY.md` に v9.4.0 完了を記録
- [x] H-5: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `json.encode(v)` が JSON 文字列を返す | ✓ |
| `json.decode(s)` が Record を返す | ✓ |
| `json.pretty(s)` が整形済み JSON を返す | ✓ |
| `csv.read<T>(path)` がファイルを読んで List を返す | ✓ |
| `csv.write_file<T>(path, rows)` がファイルを作成する | ✓ |
| `gen.uuid()` が UUID v4 文字列を返す | ✓ |
| `gen.uuid_v7()` が UUID v7 文字列を返す | ✓ |
| `gen.nano_id(n)` が n 文字の文字列を返す | ✓ |
| W004 が `fn` 引数 4 個以上で警告を出す | ✓ |
| `checker.fav` の `builtin_ret_ty` / `ns_to_effect` に Gen 追加 | ✓ |
| `cargo test` 全件通過（1182 件）| ✓ |
