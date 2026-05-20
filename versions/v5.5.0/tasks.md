# Favnir v5.5.0 タスクリスト — 標準ライブラリ型シグネチャ補完

作成日: 2026-05-20

---

## Phase A: List 型シグネチャ補完（checker.rs）

- [x] A-1: `List.flat_map` の型シグネチャを checker.rs に追加
- [x] A-2: `List.sort` の型シグネチャを checker.rs に追加
- [x] A-3: `List.find` の型シグネチャを checker.rs に追加
- [x] A-4: `List.any` / `List.all` の型シグネチャを checker.rs に追加
- [x] A-5: `List.index_of` の型シグネチャを checker.rs に追加
- [x] A-6: `List.zip` の型シグネチャを checker.rs に追加
- [x] A-7: `List.range` の型シグネチャを checker.rs に追加
- [x] A-8: `List.reverse` / `List.concat` の型シグネチャを checker.rs に追加
- [x] A-9: `List.take` / `List.drop` の型シグネチャを checker.rs に追加
- [x] A-10: `cargo test` 通過確認

---

## Phase B: String 型シグネチャ補完（checker.rs）

- [x] B-1: `String.concat` の型シグネチャを checker.rs に追加
- [x] B-2: `String.replace` の型シグネチャを checker.rs に追加
- [x] B-3: `String.slice` の型シグネチャを checker.rs に追加
- [x] B-4: `String.repeat` の型シグネチャを checker.rs に追加
- [x] B-5: `String.char_at` の型シグネチャを checker.rs に追加（`-> Option<String>`）
- [x] B-6: `String.to_int` の型シグネチャを checker.rs に追加（`-> Option<Int>`）
- [x] B-7: `String.to_float` の型シグネチャを checker.rs に追加（`-> Option<Float>`）
- [x] B-8: `String.from_chars` を vm.rs に実装（`List<String> -> String`）
- [x] B-9: `String.from_chars` の型シグネチャを checker.rs に追加
- [x] B-10: `cargo test` 通過確認

---

## Phase C: Option コンビネータ型シグネチャ補完（checker.rs）

- [x] C-1: `Option.and_then` の型シグネチャを checker.rs に追加（`Option<T>, T -> Option<U> -> Option<U>`）
- [x] C-2: `Option.or_else` の型シグネチャを checker.rs に追加
- [x] C-3: `Option.is_some` / `Option.is_none` の型シグネチャを checker.rs に追加（`-> Bool`）
- [x] C-4: `Option.to_result` の型シグネチャを checker.rs に追加
- [x] C-5: `cargo test` 通過確認

---

## Phase D: Result コンビネータ型シグネチャ補完（checker.rs）

- [x] D-1: `Result.and_then` の型シグネチャを checker.rs に追加（`Result<T,E>, T -> Result<U,E> -> Result<U,E>`）
- [x] D-2: `Result.map_err` の型シグネチャを checker.rs に追加（`Result<T,E>, E -> F -> Result<T,F>`）
- [x] D-3: `Result.is_ok` / `Result.is_err` の型シグネチャを checker.rs に追加（`-> Bool`）
- [x] D-4: `Result.to_option` の型シグネチャを checker.rs に追加（`-> Option<T>`）
- [x] D-5: `cargo test` 通過確認

---

## Phase E: Map 型シグネチャ強化（checker.rs）

- [x] E-1: `Map.size` の型シグネチャを checker.rs に追加（`-> Int`）
- [x] E-2: `Map.is_empty` の型シグネチャを checker.rs に追加（`-> Bool`）
- [x] E-3: `Map.contains_key` の型シグネチャを checker.rs に追加（`-> Bool`）
- [x] E-4: `Map.remove` の型シグネチャを checker.rs に追加（`-> Map<K,V>`）
- [x] E-5: `Map.merge` の型シグネチャを checker.rs に追加（`-> Map<K,V>`）
- [x] E-6: `Map.map_values` の型シグネチャを checker.rs に追加（`-> Map<K,U>`）
- [x] E-7: `Map.filter_values` の型シグネチャを checker.rs に追加（`-> Map<K,V>`）
- [x] E-8: `Map.to_list` の型シグネチャを checker.rs に追加（`-> List<{first:K, second:V}>`）
- [x] E-9: `Map.from_list` の型シグネチャを checker.rs に追加
- [x] E-10: `cargo test` 通過確認

---

## Phase F: vm.rs — 新規関数実装

- [x] F-1: `Map.remove(m, key) -> Map<K,V>` を vm.rs に実装
- [x] F-2: `Map.contains_key(m, key) -> Bool` を vm.rs に実装
- [x] F-3: vm_stdlib_tests.rs に `test_map_remove` を追加
- [x] F-4: vm_stdlib_tests.rs に `test_map_contains_key` を追加
- [x] F-5: vm_stdlib_tests.rs に `test_string_from_chars` を追加
- [x] F-6: `cargo test` 通過確認

---

## Phase G: 型チェックテスト追加

- [x] G-1: checker.rs に `test_list_flat_map_type` を追加
- [x] G-2: checker.rs に `test_list_sort_type` を追加
- [x] G-3: checker.rs に `test_list_zip_type` を追加
- [x] G-4: checker.rs に `test_option_and_then_type` を追加
- [x] G-5: checker.rs に `test_result_and_then_type` を追加
- [x] G-6: checker.rs に `test_result_map_err_type` を追加
- [x] G-7: checker.rs に `test_map_remove_type` を追加
- [x] G-8: checker.rs に `test_map_contains_key_type` を追加
- [x] G-9: checker.rs に `test_string_from_chars_type` を追加

---

## Phase H: まとめ

- [x] H-1: `cargo test` 全件通過（982 件）
- [x] H-2: `versions/v5.5.0/tasks.md` にチェックを入れる
- [x] H-3: `MEMORY.md` を更新
- [x] H-4: `feat: stdlib type signatures + Map.remove / String.from_chars (v5.5.0)` でコミット
