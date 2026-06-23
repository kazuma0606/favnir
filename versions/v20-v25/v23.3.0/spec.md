# v23.3.0 — 可変コレクション `Mut<T>` 仕様書

## 概要

`Mut.list()` / `Mut.map()` を中心とした可変コレクション API を追加する。
vm.fav（v23.4〜v23.8）の実装に不可欠な「VM スタック」「ローカル変数テーブル」「dispatch テーブル」を
Favnir コードから直接操作できるようにする。

---

## 背景と動機

- v23.4 以降の vm.fav では `Mut.list()` でスタックを、`Mut.map()` でローカル変数と dispatch テーブルを実装する
- Favnir には現時点で「可変」な操作が存在しない（すべて不変値）
- `Mut<T>` を限定的に導入し、スコープ内での可変性のみを許可する

---

## 設計決定

| 項目 | 決定 |
|---|---|
| 内部表現 | opaque handle (`VMValue::MutList(u64)` / `VMValue::MutMap(u64)`) — Bytes/ArrowBatch と同じパターン |
| リストストレージ | `thread_local! { static MUT_LIST_STORE: RefCell<HashMap<u64, Vec<VMValue>>> }` |
| マップストレージ | `thread_local! { static MUT_MAP_STORE: RefCell<HashMap<u64, Vec<(VMValue, VMValue)>>> }` |
| ID カウンタ | `Cell<u64>`（`BYTES_STORE` / `NEXT_BYTES_ID` と同じパターン。初期値 0）|
| マップの実装 | `Vec<(VMValue, VMValue)>` による線形探索（vm.fav の dispatch テーブルサイズは小さいため十分）|
| 型チェック | `Type::Unknown` を返す（opaque handle。generics 型チェックは将来課題）|
| `Mut.push` / `Mut.set` / `Mut.delete` の戻り値 | `ok_vm(VMValue::Unit)` — Result<Unit, String> として返す（`bind` で受け取れる）|
| `Mut.pop` / `Mut.peek` / `Mut.get` の戻り値 | `ok_vm(val)` / `err_vm(VMValue::Str(msg))` — Result として返す |
| `Mut.has` の戻り値 | `VMValue::Bool(...)` を直接返す |
| `Mut.len` の戻り値 | `VMValue::Int(n)` を直接返す（list / map 両対応）|
| 線形型強制 | 今バージョンでは未実装（コメントのみ）。将来バージョンで検討 |
| 関数数 | 10 件（list 5 + map 5）|
| 関数値の MutMap 格納 | スコープ外（vm.fav の dispatch テーブル実装は v23.4 以降）|

---

## 追加関数（10 件）

### List 操作（5 件）

| 関数 | シグネチャ（Favnir） | 返り値 | 説明 |
|---|---|---|---|
| `Mut.list` | `() -> MutList` | `VMValue::MutList(id)` | 空の可変リストを生成 |
| `Mut.push` | `MutList, T -> Result<(), String>` | `ok_vm(Unit)` | リストの末尾に値を追加 |
| `Mut.pop` | `MutList -> Result<T, String>` | `ok_vm(val)` / `err_vm` | 末尾から値を取り出す |
| `Mut.peek` | `MutList -> Result<T, String>` | `ok_vm(val)` / `err_vm` | 末尾の値を参照（取り出さない）|
| `Mut.len` | `MutList \| MutMap -> Int` | `VMValue::Int(n)` | 要素数（list / map 両対応）|

### Map 操作（5 件）

| 関数 | シグネチャ（Favnir） | 返り値 | 説明 |
|---|---|---|---|
| `Mut.map` | `() -> MutMap` | `VMValue::MutMap(id)` | 空の可変マップを生成 |
| `Mut.set` | `MutMap, K, V -> Result<(), String>` | `ok_vm(Unit)` | キーに値をセット（既存は上書き）|
| `Mut.get` | `MutMap, K -> Result<V, String>` | `ok_vm(val)` / `err_vm` | キーで値を取得 |
| `Mut.delete` | `MutMap, K -> Result<(), String>` | `ok_vm(Unit)` | キーを削除（存在しなければ no-op）|
| `Mut.has` | `MutMap, K -> Bool` | `VMValue::Bool(...)` | キーが存在するか確認 |

---

## 使用例

```favnir
// スタック操作
bind stack <- Mut.list()
bind _p1 <- Mut.push(stack, 42)     // bind で Unit を受け取る（`_` 不可のため _p1 等の名前を使う）
bind _p2 <- Mut.push(stack, 99)
bind result <- Mut.pop(stack)        // ok(99)
bind n <- Mut.len(stack)             // 1（pop 後）

// ローカル変数テーブル
bind locals <- Mut.map()
bind _s1 <- Mut.set(locals, "x", 10)
bind x <- Mut.get(locals, "x")      // ok(10)
bind exists <- Mut.has(locals, "z") // false
bind _d1 <- Mut.delete(locals, "x")
bind n2 <- Mut.len(locals)          // 0
```

> **注意**: Favnir の `bind` では `_` を変数名として使えないため、
> `Mut.push` / `Mut.set` / `Mut.delete` の戻り値は `bind _p1 <- ...` のように
> 名前付き変数で受け取ること。

---

## 実装方針（概要）

| タスク | ファイル | 内容 |
|---|---|---|
| T1 | `heap_val.rs` | `HeapVal::MutList(u64)` / `HeapVal::MutMap(u64)` 追加 |
| T2 | `nan_val.rs` | `from_vmvalue` / `to_vmvalue` メソッドに MutList / MutMap アーム追加 |
| T3 | `vm.rs` | VMValue 追加・thread-local ストア・10 ハンドラ・`is_known_builtin_namespace` |
| T4 | `checker.rs` | namespace + `builtin_ret_ty` 更新 |
| T5 | `compiler.rs` | builtins リストに `"Mut"` 追加 |
| T6 | `driver.rs` | `#[ignore]` + `v233000_tests` （5 件）|
| T7 | docs | Cargo.toml / CHANGELOG / benchmarks / MDX |

> **実装順序制約**: T7-1（Cargo.toml バージョン更新）は T6 の `#[ignore]` 追加より後に行うこと。
> `version_is_23_2_0` テストが Cargo.toml の新バージョン文字列で失敗するため。

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

## スコープ外（今後のバージョン）

- `Mut<T>` の線形型強制（スコープ外持ち出しコンパイルエラー）→ 将来バージョン
- generics 型チェック（`Mut<List<Int>>` 等の型パラメータ検証）→ 将来バージョン
- `Mut.list<VMValue>()` の `<VMValue>` 型引数パース → v23.4 以降で検討
- ファーストクラス関数値の `MutMap` への格納（dispatch テーブル）→ v23.4 以降（vm.fav 実装時に確認）
- メモリリーク（GC 未実装）→ 既知制限、v25.x で対応予定
