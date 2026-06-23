# v23.3.0 — 可変コレクション `Mut<T>` タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/backend/heap_val.rs` — `HeapVal::MutList(u64)` / `HeapVal::MutMap(u64)` 追加

- [x] **事前確認**: `grep -n "Bytes\|BigInt\|PgPool" fav/src/backend/heap_val.rs | head -10` で挿入位置確認
- [x] `HeapVal::Bytes(u64)` の直後に `MutList(u64)` / `MutMap(u64)` を追加（plan.md T1）
- [x] `PartialEq` に `MutList` / `MutMap` アームを追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/backend/nan_val.rs` — `from_vmvalue` / `to_vmvalue` メソッドに MutList/MutMap 追加

- [x] **事前確認**: `grep -n "Bytes\|PgPool\|from_vmvalue\|to_vmvalue" fav/src/backend/nan_val.rs | head -10` で変換箇所確認
- [x] `from_vmvalue` メソッドに `VMValue::MutList(id)` / `VMValue::MutMap(id)` アームを追加（plan.md T2）
- [x] `to_vmvalue` メソッドに `HeapVal::MutList(id)` / `HeapVal::MutMap(id)` アームを追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/backend/vm.rs` — VMValue + STORE + vm_call_builtin

- [x] **事前確認**: `grep -n "VMValue::Bytes\|BYTES_STORE\|NEXT_BYTES_ID" fav/src/backend/vm.rs | head -10` で挿入位置確認
- [x] `VMValue` 列挙型に `MutList(u64)` / `MutMap(u64)` を追加（`Bytes(u64)` 直後）
- [x] `VMValue::PartialEq` / display / `vmvalue_type_name` / stringify の各マッチアームに MutList / MutMap を追加（plan.md T3-2）
- [x] `HeapVal` の type_name マッチに `MutList` / `MutMap` アームを追加
- [x] thread-local ストアを追加（plan.md T3-3 のコードに従う）
  - `MUT_LIST_STORE: RefCell<HashMap<u64, Vec<VMValue>>>` + `NEXT_MUT_LIST_ID: Cell<u64>` (初期値 0)
  - `MUT_MAP_STORE: RefCell<HashMap<u64, Vec<(VMValue, VMValue)>>>` + `NEXT_MUT_MAP_ID: Cell<u64>` (初期値 0)
  - `mut_list_new()` / `mut_map_new()` ヘルパー関数
- [x] `vm_call_builtin` に 10 ハンドラを flat literal アームで追加（plan.md T3-4 のコードに従う）
  - `"Mut.list"` / `"Mut.push"` / `"Mut.pop"` / `"Mut.peek"` / `"Mut.len"`
  - `"Mut.map"` / `"Mut.set"` / `"Mut.get"` / `"Mut.delete"` / `"Mut.has"`
  - `err_vm` には `err_vm(VMValue::Str("...".to_string()))` を渡す（String 直接渡し不可）
  - `Mut.push` / `Mut.set` / `Mut.delete` は `Ok(ok_vm(VMValue::Unit))` を返す
- [x] `is_known_builtin_namespace` に `| "Mut"     // v23.3.0` を追加（`"Bytes"` の直後）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/middle/checker.rs` — namespace リスト + builtin_ret_ty 更新

- [x] **事前確認**: `grep -n '"Bytes"\|// Mut\|// Int bit' fav/src/middle/checker.rs | head -5` で挿入位置確認
- [x] namespace リストに `"Mut",` を追加（`"Bytes"` の直後）
- [x] `builtin_ret_ty` に 5 エントリを追加（plan.md T4-2 のコードに従う）
  - `("Mut", "list") | ("Mut", "map") => Some(Type::Unknown)`
  - `("Mut", "push") | ("Mut", "set") | ("Mut", "delete") => Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))`
  - `("Mut", "pop") | ("Mut", "peek") | ("Mut", "get") => Some(Type::Result(Box::new(Type::Unknown), Box::new(Type::String)))`
  - `("Mut", "len") => Some(Type::Int)`
  - `("Mut", "has") => Some(Type::Bool)`
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/middle/compiler.rs` — builtins リスト更新

- [x] **事前確認**: `grep -n '"Bytes"\|"Arena\.stats"' fav/src/middle/compiler.rs | head -5` で挿入位置確認
- [x] `"Bytes",` の直後に `"Mut",` を追加（plan.md T5）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/src/driver.rs` — `#[ignore]` + `v233000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_2_0\|mod v232000_tests\|mod v233000_tests" fav/src/driver.rs | head -5`
- [x] **T7-1（Cargo.toml バージョン更新）より前に実施**: `v232000_tests::version_is_23_2_0` に `#[ignore]` を追加
- [x] `v233000_tests` モジュールを `v232000_tests` の直後に追加（5 件、plan.md T6-2 のコードに従う）
  - `version_is_23_3_0`
  - `mut_list_push_pop_correct`（`bind _p1 <- Mut.push(...)` パターン）
  - `mut_list_len_after_push`
  - `mut_map_set_get_correct`（`bind _s1 <- Mut.set(...)` パターン）
  - `changelog_has_v23_3_0`
- [x] `cargo test v233000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1898 件以上合格）を確認

---

### T7: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意（T7-1 より前）**: T6-1 の `#[ignore]` 追加が完了してから `Cargo.toml` を更新すること。

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリを確認
- [x] `fav/Cargo.toml` の `version = "23.2.0"` → `"23.3.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.2.0 エントリの上）に v23.3.0 エントリを追加（plan.md T7-2）
- [x] `benchmarks/v23.3.0.json` を新規作成（plan.md T7-3）
- [x] `site/content/docs/runes/mut.mdx` を新規作成（plan.md T7-4）
- [x] `cargo test v233000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1898 件以上合格）を再確認

---

## テスト一覧（v233000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_3_0` | Cargo.toml に `version = "23.3.0"` が含まれる |
| `mut_list_push_pop_correct` | push(42), push(99), pop → ok(99) |
| `mut_list_len_after_push` | push × 2 → len = 2 |
| `mut_map_set_get_correct` | set("key", 42) → get("key") → ok(42) |
| `changelog_has_v23_3_0` | CHANGELOG.md に `[v23.3.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `v232000_tests::version_is_23_2_0` に `#[ignore]` が追加済み（T7-1 より前）
- [x] `HeapVal::MutList(u64)` / `HeapVal::MutMap(u64)` が `heap_val.rs` に追加される
- [x] `nan_val.rs` の `from_vmvalue` / `to_vmvalue` メソッドに MutList / MutMap が追加される
- [x] `VMValue::MutList(u64)` / `VMValue::MutMap(u64)` が `vm.rs` に追加される
- [x] `MUT_LIST_STORE` / `MUT_MAP_STORE` thread-local が `vm.rs` に追加される（`Cell<u64>` カウンタ、二重 RefCell なし）
- [x] `vm_call_builtin` に `"Mut.*"` の 10 ハンドラが追加される（flat literal アーム、`into_iter()` パターン）
- [x] `err_vm` は `VMValue::Str("...".to_string())` で呼ぶ（`String` 直接渡し不可）
- [x] `Mut.push` / `Mut.set` / `Mut.delete` は `Ok(ok_vm(VMValue::Unit))` を返す
- [x] `Mut.pop` / `Mut.peek` / `Mut.get` は `Result<VMValue, String>` を返す（`MUT_*_STORE.with(|s| { ... })` の戻り値がそのまま `Result<VMValue, String>`）
- [x] `Mut.has` / `Mut.len` は `Ok(VMValue::Bool(...))` / `Ok(VMValue::Int(...))` を返す
- [x] `is_known_builtin_namespace` に `"Mut"` が追加される
- [x] `checker.rs` の namespace リストに `"Mut"` が追加される
- [x] `checker.rs` の `builtin_ret_ty` に 5 エントリ（10 関数分）が追加される
- [x] `compiler.rs` の builtins リストに `"Mut"` が追加される
- [x] `cargo test v233000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1898 件以上合格）
- [x] `CHANGELOG.md` に v23.3.0 エントリ
- [x] `benchmarks/v23.3.0.json` 作成済み
- [x] `site/content/docs/runes/mut.mdx` 作成済み

---

## 優先度

```
T1（heap_val.rs）  ← 最初（MutList/MutMap の定義。以降の依存元）
T2（nan_val.rs）   ← T1 完了後
T3（vm.rs）        ← T2 完了後（最大タスク）
T4（checker.rs）   ← T3 と並列可
T5（compiler.rs）  ← T3 と並列可
T6（driver.rs）    ← T3〜T5 完了後、T7-1 より前に #[ignore] を実施
T7（docs）         ← T6 完了後（#[ignore] 確認後にバージョン更新）
```

## 実装時の注意事項（spec-reviewer 指摘対応済み）

| # | 内容 | 対応 |
|---|---|---|
| 1 | `err_vm(String)` 引数型ミス | `err_vm(VMValue::Str("...".to_string()))` に修正 |
| 2 | spec/plan 間のストア型定義不一致 | `RefCell<HashMap<u64, Vec<...>>>` (二重 RefCell なし) に統一 |
| 3 | "From<VMValue> に追加"という誤表現 | `from_vmvalue` / `to_vmvalue` メソッドに追加と修正 |
| 4 | 二重 Result の曖昧さ | `MUT_*_STORE.with(|s| { ... })` が `Result<VMValue, String>` を返す構造を明示 |
| 5 | `Mut.push(...)` 単独呼び出し構文未確認 | `bind _p1 <- Mut.push(...)` パターンに統一（Mut.push は `ok_vm(Unit)` を返す） |
| 6 | ロードマップ付記（関数値 Map 格納）への対応 | スコープ外として spec.md に明示（v23.4 以降）|
| 7 | `Cell` vs `RefCell` カウンタ不統一 | `Cell<u64>` に統一（BYTES_STORE パターンと同じ） |
| 8 | `#[ignore]` 追加順序の理由が spec.md に未記載 | 実装方針セクションに順序制約の理由を追記 |

---

## 実装完了メモ（2026-06-22）

- **テスト**: 5/5 v233000_tests PASS、1902 tests 全通過（0 failures）
- **重要な修正点**: テストコードで `let lst = Mut.list()` は不可（Favnir に `let` キーワードなし）→ `bind lst <- Mut.list()` に変更
- **`Mut.list()` / `Mut.map()`**: Result でなく直接値を返す → `bind lst <- Mut.list()` でバインド可
- **`Mut.pop()` / `Mut.get()` の Result 展開**: テスト内で `match pop_result { ok(v) => ... err(_) => ... }` パターンを使用
- **stringify 関数**: `VMValue::MutList(id) => format!("<mut_list:{}>", id)` / `MutMap` アームを追加（cargo check で検出）

## コードレビュー対応（2026-06-22）

| # | 指摘 | 対応 |
|---|---|---|
| [HIGH]-1 | `Mut.push`/`Mut.set`/`Mut.delete` の `ok_vm` 二重ラップ設計の非対称性 | `Mut.push` にコメント追加（checker 型は `Result<Unit, String>`、`bind _p <- Mut.push(...)` が標準パターン） |
| [HIGH]-2 | `borrow_mut` 入れ子リスク | thread-local ブロックに安全性注記コメント追加 |
| [MED]-1 | `Type::Unknown` コメントなし | checker.rs に説明コメント追加（ArrowBatch と同パターン） |
| [MED]-2 | `Mut.delete` キー不在仕様未明記 | コメント追加（冪等削除、キー不在でも `ok(unit)` を返す） |
| [MED]-3 | ID カウンタ 0 始まり説明なし | コメント追加（NEXT_BYTES_ID と同パターン、DB_NEXT_ID の 1 始まりとは異なる） |
| [MED]-4 | エラーパステスト不足 | テスト 3 件追加（pop 空リスト Err / get 不在キー Err / has 正常 Bool） |
| [LOW] | stringify と to_value の記法不一致（`mut_list` vs `mut-list`） | stringify を `<mut-list:>` / `<mut-map:>` に統一（ハイフン記法） |
| [LOW] | 線形探索コメントなし | thread-local コメントに `O(n)` 制限を明記 |

最終テスト結果: **1905 passed, 0 failed**（v233000_tests 8/8 PASS）
