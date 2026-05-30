# Favnir v8.2.0 Tasks

Date: 2026-05-29
Theme: stdlib Favnir 化 — List / String / Map 高レベル関数のセルフホスト

---

## Phase A: list_stdlib.fav（List 高レベル関数）

- [x] A-1: `fav/self/stdlib/` ディレクトリ作成
- [x] A-2: `list_stdlib.fav` 新規作成 — `intersperse`
  — `List.fold` でアキュムレータに間に挟みながら構築
  — `List.singleton` は `List<T>` を返すため EIf 型不一致 → `List.push(List.empty(), x)` で代用
- [x] A-3: `fav check fav/self/stdlib/list_stdlib.fav` — エラーなし確認

---

## Phase B: string_stdlib.fav（String 高レベル関数）

- [x] B-1: `fav/self/stdlib/string_stdlib.fav` 新規作成 — `capitalize`
  — `String.slice(s, 0, 1)` → `String.upper` + rest
- [x] B-2: `indent`
  — `String.lines` → `List.map(|line| indent_line(prefix, line))` → `String.join("\n")`
- [x] B-3: `fav check fav/self/stdlib/string_stdlib.fav` — エラーなし確認

---

## Phase C: map_stdlib.fav（Map 高レベル関数）

- [x] C-1: skip — すべての対象 Map 関数が既に vm.rs に実装済み

---

## Phase D: stdlib_fav_runner.rs（ローダー＋vm.rs 統合）

- [x] D-1: `src/stdlib_fav_runner.rs` 新規作成
  — `OnceLock<Arc<FvcArtifact>>` ×2（list / string）
  — `get_list_stdlib_artifact()` / `get_string_stdlib_artifact()`
- [x] D-2: `pub fn call_list_stdlib(fname, args) -> Result<Value, VMError>`
  — `pub fn call_string_stdlib(fname, args) -> Result<Value, VMError>`
- [x] D-3: `main.rs` に `mod stdlib_fav_runner;` を追加
- [x] D-4: `src/backend/vm.rs` の `vm_call_builtin` に dispatch 追加
  — `"List.intersperse"` / `"String.capitalize"` / `"String.indent"` を Favnir 実装へ
- [x] D-5: `List.scan` を `VM::exec` に Rust 実装（closure 引数のため）
- [x] D-6: `cargo build` — 型エラーなし確認

---

## Phase E: 統合テスト（7 件）

- [x] E-1: `list_intersperse_basic` — `["a","b","c"]` を `","` で intersperse → `["a",",","b",",","c"]`
- [x] E-2: `list_intersperse_empty` — 空リストで intersperse → `[]`
- [x] E-3: `list_scan_prefix_sums` — `[1,2,3]` の prefix sums → `[0,1,3,6]`
- [x] E-4: `string_capitalize_basic` — `"hello world"` → `"Hello world"`
- [x] E-5: `string_capitalize_empty` — `""` → `""`
- [x] E-6: `string_indent_multiline` — `"hello\nworld"` を 2 spaces → `"  hello\n  world"`
- [x] E-7: `string_indent_single` — `"hello"` を 4 spaces → `"    hello"`

---

## Phase F: 最終確認・ドキュメント

- [x] F-1: `fav check fav/self/stdlib/list_stdlib.fav` — no errors
- [x] F-2: `fav check fav/self/stdlib/string_stdlib.fav` — no errors
- [x] F-3: `cargo test` — 1113 tests passing（+7 新規）
- [x] F-4: checker.fav — `str_fn` に `lines/words/capitalize/indent` 追加
- [x] F-5: checker.fav — `list_fn` に `scan` 追加
- [x] F-6: checker.rs — `List.intersperse` / `List.scan` / `String.capitalize` / `String.indent` 型シグネチャ追加
- [x] F-7: このファイルを完了状態に更新
- [x] F-8: commit

---

## 完了条件

- `fav/self/stdlib/list_stdlib.fav` が `fav check` でエラーなし ✓
- `fav/self/stdlib/string_stdlib.fav` が `fav check` でエラーなし ✓
- `List.intersperse` / `String.capitalize` / `String.indent` が Favnir 実装経由 ✓
- `List.scan` が Rust 実装（closure 引数ありのため） ✓
- 既存テストが全件通る（1106 passing → 1113 passing）✓
- 新規統合テスト 7 件 ✓

---

## 実装ノート

- `infer_arg_tys` が arg_tys を逆順に構築するため `List.singleton(x)` (→ "List<String>") と
  `List.push(...)` (→ "List" bare) でEIf 型不一致になる
  → `List.singleton(x)` の代わりに `List.push(List.empty(), x)` を使用（両方 "List" bare）
- `stdlib_fav_runner.rs` は checker_fav_runner.rs と同じ OnceLock パターン
  — lib.rs に backend が含まれないため src/ 直下に配置、main.rs のみで宣言
- `List.scan` は closure 引数を受け取るため Rust（`VM::exec` 内 `self.call_value`）で実装
- Map 高レベル関数（merge, from_list, to_list, map_values 等）は既に vm.rs に実装済み → 対象外
