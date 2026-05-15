# Favnir v0.7.0 タスク一覧

更新日: 2026-04-30 (完了)

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: VM コールバック機構 + List 完全化

### VM::call_value の追加 (`src/vm.rs`)

- [x] 1-1: dispatch ループを `run_loop(artifact, until_depth)` に分離する
- [x] 1-2: `VM::call_value(artifact, callee, args)` を実装する
- [x] 1-3: CALL ハンドラで `List.map` の高階対応を追加する
- [x] 1-4: CALL ハンドラで `List.filter` の高階対応を追加する
- [x] 1-5: CALL ハンドラで `List.fold` の高階対応を追加する
- [x] 1-6: CALL ハンドラで `List.flat_map` の高階対応を追加する
- [x] 1-7: CALL ハンドラで `List.sort` の高階対応を追加する
- [x] 1-8: CALL ハンドラで `List.find` の高階対応を追加する
- [x] 1-9: CALL ハンドラで `List.any` / `List.all` の高階対応を追加する

### eval.rs: 新規 List 関数

- [x] 1-10: `List.flat_map(xs, f)` を eval.rs に追加する
- [x] 1-11: `List.zip(xs, ys)` を eval.rs に追加する（Pair を返す）
- [x] 1-12: `List.sort(xs, cmp)` を eval.rs に追加する
- [x] 1-13: `List.range(start, end)` を eval.rs に追加する
- [x] 1-14: `List.reverse(xs)` を eval.rs に追加する
- [x] 1-15: `List.concat(xs, ys)` を eval.rs に追加する
- [x] 1-16: `List.take(xs, n)` を eval.rs に追加する
- [x] 1-17: `List.drop(xs, n)` を eval.rs に追加する
- [x] 1-18: `List.enumerate(xs)` を eval.rs に追加する（Pair{index, value}）
- [x] 1-19: `List.find(xs, pred)` を eval.rs に追加する
- [x] 1-20: `List.any(xs, pred)` / `List.all(xs, pred)` を eval.rs に追加する
- [x] 1-21: `List.index_of(xs, pred)` を eval.rs に追加する
- [x] 1-22: `List.join(xs, sep)` を eval.rs に追加する（List<String> → String）

### vm.rs: 非高階 List 関数

- [x] 1-23: vm_call_builtin に List.zip/range/reverse/concat/take/drop/enumerate/index_of/join を追加する
  - 注: List.index_of は高階のため CALL ハンドラに実装

---

## Phase 2: String / Map 完全化

### eval.rs: 新規 String 関数

- [x] 2-1: `String.join(xs, sep)` を eval.rs に追加する
- [x] 2-2: `String.replace(s, from, to)` を eval.rs に追加する
- [x] 2-3: `String.starts_with(s, prefix)` を eval.rs に追加する
- [x] 2-4: `String.ends_with(s, suffix)` を eval.rs に追加する
- [x] 2-5: `String.contains(s, sub)` を eval.rs に追加する
- [x] 2-6: `String.slice(s, start, end)` を eval.rs に追加する（char 単位）
- [x] 2-7: `String.repeat(s, n)` を eval.rs に追加する
- [x] 2-8: `String.char_at(s, idx)` を eval.rs に追加する（→ Option<String>）
- [x] 2-9: `String.to_int(s)` を eval.rs に追加する（→ Option<Int>）
- [x] 2-10: `String.to_float(s)` を eval.rs に追加する（→ Option<Float>）
- [x] 2-11: `String.from_int(n)` を eval.rs に追加する
- [x] 2-12: `String.from_float(f)` を eval.rs に追加する

### eval.rs: 新規 Map 関数

- [x] 2-13: `Map.has_key(m, key)` を eval.rs に追加する
- [x] 2-14: `Map.size(m)` を eval.rs に追加する
- [x] 2-15: `Map.is_empty(m)` を eval.rs に追加する
- [x] 2-16: `Map.merge(base, overrides)` を eval.rs に追加する
- [x] 2-17: `Map.from_list(pairs)` を eval.rs に追加する
- [x] 2-18: `Map.to_list(m)` を eval.rs に追加する（キーソート済み Pair リスト）
- [x] 2-19: `Map.map_values(m, f)` を eval.rs に追加する（高階）
- [x] 2-20: `Map.filter_values(m, pred)` を eval.rs に追加する（高階）

### vm.rs: String / Map ビルトイン

- [x] 2-21: vm_call_builtin に String 関数（非高階）を追加する
- [x] 2-22: vm_call_builtin に Map 関数（非高階）を追加する
- [x] 2-23: CALL ハンドラに Map.map_values / Map.filter_values の高階対応を追加する

---

## Phase 3: Option / Result 高階関数

### eval.rs

- [x] 3-1: `Option.map(o, f)` を eval.rs に追加する
- [x] 3-2: `Option.and_then(o, f)` を eval.rs に追加する
- [x] 3-3: `Option.unwrap_or(o, default)` を eval.rs に追加する
- [x] 3-4: `Option.or_else(o, f)` を eval.rs に追加する
- [x] 3-5: `Option.is_some(o)` / `Option.is_none(o)` を eval.rs に追加する
- [x] 3-6: `Option.to_result(o, err_val)` を eval.rs に追加する
- [x] 3-7: `Result.map(r, f)` を eval.rs に追加する
- [x] 3-8: `Result.map_err(r, f)` を eval.rs に追加する
- [x] 3-9: `Result.and_then(r, f)` を eval.rs に追加する
- [x] 3-10: `Result.unwrap_or(r, default)` を eval.rs に追加する
- [x] 3-11: `Result.is_ok(r)` / `Result.is_err(r)` を eval.rs に追加する
- [x] 3-12: `Result.to_option(r)` を eval.rs に追加する

### vm.rs

- [x] 3-13: CALL ハンドラに Option.map / Option.and_then / Option.or_else の高階対応を追加する
- [x] 3-14: CALL ハンドラに Result.map / Result.map_err / Result.and_then の高階対応を追加する
- [x] 3-15: vm_call_builtin に残りの Option/Result 関数（非高階）を追加する

---

## Phase 4: !File エフェクト + File.* ビルトイン

- [x] 4-1: `ast.rs` の `Effect` enum に `File` バリアントを追加する
- [x] 4-2: Effect の全 match/display/merge_effect/format_effects に `File` アームを追加する
- [x] 4-3: `checker.rs` に `File.*` の `!File` effect チェック（E036）を追加する
- [x] 4-4: `eval.rs` に `File.read` / `File.read_lines` を追加する
- [x] 4-5: `eval.rs` に `File.write` / `File.write_lines` / `File.append` を追加する
- [x] 4-6: `eval.rs` に `File.exists` / `File.delete` を追加する
- [x] 4-7: `vm.rs` の vm_call_builtin に File.* を追加する
- [x] 4-8: `compiler.rs` の Builtin 登録に `"File"` を追加する
- [x] 4-9: `main.rs` の format_effects / HELP に `!File` を追加する

---

## Phase 5: Json.* ビルトイン

- [x] 5-1: `Cargo.toml` に `serde_json = "1"` を追加する
- [x] 5-2: `eval.rs` に `serde_to_favnir` / `favnir_to_serde` ヘルパーを実装する
- [x] 5-3: `eval.rs` に `Json.parse` / `Json.encode` / `Json.encode_pretty` を追加する
- [x] 5-4: `eval.rs` に `Json.get` / `Json.at` / `Json.keys` / `Json.length` を追加する
- [x] 5-5: `eval.rs` に `Json.as_str` / `Json.as_int` / `Json.as_float` / `Json.as_bool` / `Json.as_array` / `Json.is_null` を追加する
- [x] 5-6: `vm.rs` の vm_call_builtin に Json.* を追加する
- [x] 5-7: `compiler.rs` の Builtin 登録に `"Json"` を追加する

---

## Phase 6: Csv.* ビルトイン

- [x] 6-1: `Cargo.toml` に `csv = "1"` を追加する
- [x] 6-2: `eval.rs` に `Csv.parse` / `Csv.parse_with_header` を追加する
- [x] 6-3: `eval.rs` に `Csv.encode` / `Csv.encode_with_header` / `Csv.from_records` を追加する
- [x] 6-4: `vm.rs` の vm_call_builtin に Csv.* を追加する
- [x] 6-5: `compiler.rs` の Builtin 登録に `"Csv"` を追加する

---

## Phase 7: テスト + サンプルファイル

- [x] 7-1: eval.rs テスト: list_range/reverse/concat/take_drop/flat_map/zip/sort/find_any_all/join を追加する
- [x] 7-2: eval.rs テスト: option_and_then/is_some, result_map_and_then/map_err を追加する
- [x] 7-3: eval.rs テスト: string_join/replace/slice/predicates/to_from_int を追加する
- [x] 7-4: eval.rs テスト: map_merge/from_list_to_list/has_key_is_empty を追加する
- [x] 7-5: eval.rs テスト: file_read_write_roundtrip を追加する（tempfile 使用）
- [x] 7-6: eval.rs テスト: json_parse_encode_roundtrip を追加する
- [x] 7-7: eval.rs テスト: csv_parse_with_header を追加する
- [x] 7-8: vm.rs 統合テスト: vm_integration_list_map_closure を追加する
- [x] 7-9: vm.rs 統合テスト: vm_integration_option_and_then を追加する
- [x] 7-10: `examples/data/sample.csv` を追加する
- [x] 7-11: `examples/csv_to_json.fav` を追加する
- [x] 7-12: roadmap.md の v0.7.0 完了日を更新する
- [x] 7-13: MEMORY.md を v0.7.0 完了内容で更新する
