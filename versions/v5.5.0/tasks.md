# Favnir v5.5.0 タスクリスト — 標準ライブラリ型シグネチャ補完

作成日: 2026-05-20

---

## Phase A: List 型シグネチャ補完（checker.rs）

- [ ] A-1: `List.flat_map` の型シグネチャを checker.rs に追加
- [ ] A-2: `List.sort` の型シグネチャを checker.rs に追加
- [ ] A-3: `List.find` の型シグネチャを checker.rs に追加
- [ ] A-4: `List.any` / `List.all` の型シグネチャを checker.rs に追加
- [ ] A-5: `List.index_of` の型シグネチャを checker.rs に追加
- [ ] A-6: `List.zip` の型シグネチャを checker.rs に追加
- [ ] A-7: `List.range` の型シグネチャを checker.rs に追加
- [ ] A-8: `List.reverse` / `List.concat` の型シグネチャを checker.rs に追加
- [ ] A-9: `List.take` / `List.drop` の型シグネチャを checker.rs に追加
- [ ] A-10: `cargo test` 通過確認

---

## Phase B: String 型シグネチャ補完（checker.rs）

- [ ] B-1: `String.concat` の型シグネチャを checker.rs に追加
- [ ] B-2: `String.replace` の型シグネチャを checker.rs に追加
- [ ] B-3: `String.slice` の型シグネチャを checker.rs に追加
- [ ] B-4: `String.repeat` の型シグネチャを checker.rs に追加
- [ ] B-5: `String.char_at` の型シグネチャを checker.rs に追加（`-> Option<String>`）
- [ ] B-6: `String.to_int` の型シグネチャを checker.rs に追加（`-> Option<Int>`）
- [ ] B-7: `String.to_float` の型シグネチャを checker.rs に追加（`-> Option<Float>`）
- [ ] B-8: `String.from_chars` を vm.rs に実装（`List<String> -> String`）
- [ ] B-9: `String.from_chars` の型シグネチャを checker.rs に追加
- [ ] B-10: `cargo test` 通過確認

---

## Phase C: Option コンビネータ型シグネチャ補完（checker.rs）

- [ ] C-1: `Option.and_then` の型シグネチャを checker.rs に追加（`Option<T>, T -> Option<U> -> Option<U>`）
- [ ] C-2: `Option.or_else` の型シグネチャを checker.rs に追加
- [ ] C-3: `Option.is_some` / `Option.is_none` の型シグネチャを checker.rs に追加（`-> Bool`）
- [ ] C-4: `Option.to_result` の型シグネチャを checker.rs に追加
- [ ] C-5: `cargo test` 通過確認

---

## Phase D: Result コンビネータ型シグネチャ補完（checker.rs）

- [ ] D-1: `Result.and_then` の型シグネチャを checker.rs に追加（`Result<T,E>, T -> Result<U,E> -> Result<U,E>`）
- [ ] D-2: `Result.map_err` の型シグネチャを checker.rs に追加（`Result<T,E>, E -> F -> Result<T,F>`）
- [ ] D-3: `Result.is_ok` / `Result.is_err` の型シグネチャを checker.rs に追加（`-> Bool`）
- [ ] D-4: `Result.to_option` の型シグネチャを checker.rs に追加（`-> Option<T>`）
- [ ] D-5: `cargo test` 通過確認

---

## Phase E: Map 型シグネチャ強化（checker.rs）

- [ ] E-1: `Map.size` の型シグネチャを checker.rs に追加（`-> Int`）
- [ ] E-2: `Map.is_empty` の型シグネチャを checker.rs に追加（`-> Bool`）
- [ ] E-3: `Map.contains_key` の型シグネチャを checker.rs に追加（`-> Bool`）
- [ ] E-4: `Map.remove` の型シグネチャを checker.rs に追加（`-> Map<K,V>`）
- [ ] E-5: `Map.merge` の型シグネチャを checker.rs に追加（`-> Map<K,V>`）
- [ ] E-6: `Map.map_values` の型シグネチャを checker.rs に追加（`-> Map<K,U>`）
- [ ] E-7: `Map.filter_values` の型シグネチャを checker.rs に追加（`-> Map<K,V>`）
- [ ] E-8: `Map.to_list` の型シグネチャを checker.rs に追加（`-> List<{first:K, second:V}>`）
- [ ] E-9: `Map.from_list` の型シグネチャを checker.rs に追加
- [ ] E-10: `cargo test` 通過確認

---

## Phase F: vm.rs — 新規関数実装

- [ ] F-1: `Map.remove(m, key) -> Map<K,V>` を vm.rs に実装
- [ ] F-2: `Map.contains_key(m, key) -> Bool` を vm.rs に実装
- [ ] F-3: vm_stdlib_tests.rs に `test_map_remove` を追加
- [ ] F-4: vm_stdlib_tests.rs に `test_map_contains_key` を追加
- [ ] F-5: vm_stdlib_tests.rs に `test_string_from_chars` を追加
- [ ] F-6: `cargo test` 通過確認

---

## Phase G: 型チェックテスト追加

- [ ] G-1: checker.rs に `test_list_flat_map_type` を追加
- [ ] G-2: checker.rs に `test_list_sort_type` を追加
- [ ] G-3: checker.rs に `test_list_zip_type` を追加
- [ ] G-4: checker.rs に `test_option_and_then_type` を追加
- [ ] G-5: checker.rs に `test_result_and_then_type` を追加
- [ ] G-6: checker.rs に `test_result_map_err_type` を追加
- [ ] G-7: checker.rs に `test_map_remove_type` を追加
- [ ] G-8: checker.rs に `test_map_contains_key_type` を追加
- [ ] G-9: checker.rs に `test_string_from_chars_type` を追加

---

## Phase H: まとめ

- [ ] H-1: `cargo test` 全件通過（971 件 + 新規テスト）
- [ ] H-2: `versions/v5.5.0/tasks.md` にチェックを入れる
- [ ] H-3: `MEMORY.md` を更新
- [ ] H-4: `feat: stdlib type signatures + Map.remove / String.from_chars (v5.5.0)` でコミット
