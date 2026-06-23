# v23.1.0 — `Bytes` 型 タスク

## ステータス: COMPLETE

実装完了: 2026-06-21
テスト結果: 1890 passed / 0 failed（v231000_tests 5/5 PASS）

---

## タスク一覧

### T1: `fav/src/backend/heap_val.rs` — `HeapVal::Bytes(u64)` 追加

- [x] **事前確認**: `grep -n "PgPool\|ArrowBatch\|BigInt" fav/src/backend/heap_val.rs | head -10` で挿入位置確認
- [x] `HeapVal` 列挙型に `/// v23.1.0: 生バイト列 opaque handle\nBytes(u64),` を追加（`PgPool(u64)` の直後、`BigInt` の前）
- [x] `PartialEq` 実装に `(HeapVal::Bytes(a), HeapVal::Bytes(b)) => a == b,` を追加（`PgPool` アームの直後）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/backend/nan_val.rs` — VMValue::Bytes ↔ NanVal 変換追加

- [x] **事前確認**: `grep -n "PgPool\|ArrowBatch" fav/src/backend/nan_val.rs | head -10` で変換箇所を確認
- [x] `From<VMValue>` 変換に `VMValue::Bytes(id) => NanVal::from_heap(HeapVal::Bytes(id)),` を追加
- [x] `to_vmvalue()` の HeapVal マッチに `HeapVal::Bytes(id) => VMValue::Bytes(*id),` を追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/backend/vm.rs` — VMValue + BYTES_STORE + vm_call_builtin

- [x] **事前確認**: `grep -n "PgPool\|\"State\"\|NEXT_ARROW_ID\|\"State\\\.\|\"Arena\\\." fav/src/backend/vm.rs | head -20` で挿入位置を確認
- [x] `VMValue` 列挙型に `/// v23.1.0: 生バイト列 opaque handle\nBytes(u64),` を追加（`PgPool(u64)` 直後）
- [x] `VMValue::PartialEq` に `(VMValue::Bytes(a), VMValue::Bytes(b)) => a == b,` 追加
- [x] display / type_name 等のマッチアームすべてに `VMValue::Bytes` アームを追加
  - `Value::Str(format!("<bytes:{id}>"))` パターン
  - `"Bytes"` 文字列返しパターン
- [x] `BYTES_STORE` + `NEXT_BYTES_ID` thread-local と `bytes_new` / `bytes_get_arc` ヘルパーを追加（plan.md T3-4）
- [x] `vm_call_builtin` に Bytes 13 ハンドラを flat literal アームで追加（plan.md T3-5 のコードに従う。guard パターン不使用）
  - `Bytes.from_hex` / `Bytes.from_str` / `Bytes.len` / `Bytes.get` / `Bytes.slice` / `Bytes.concat`
  - `Bytes.to_utf8` / `Bytes.to_hex`
  - `Bytes.read_u16` / `Bytes.read_u24` / `Bytes.read_u32`
  - `Bytes.read_file` / `Bytes.write_file`（`#[cfg(not(target_arch = "wasm32"))]` 内）
- [x] `is_known_builtin_namespace` に `| "Bytes"   // v23.1.0` を追加（`"State"` の直後）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/middle/checker.rs` — namespace リスト更新

- [x] **事前確認**: `grep -n "\"Arena\"\|\"Bytes\"" fav/src/middle/checker.rs | head -5` で挿入位置確認
- [x] `"Arena",` の直後に `"Bytes",` を追加
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/middle/compiler.rs` — builtins リスト更新

- [x] **事前確認**: `grep -n "Arena\.stats\|\"Bytes\"\|ArrowBatch" fav/src/middle/compiler.rs | head -5` で挿入位置確認
- [x] `"Arena.stats",` の直後に Bytes 13 エントリを追加（plan.md T5-1 に従う）
- [x] `cargo check --bin fav` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/src/driver.rs` — `#[ignore]` + `v231000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_0_0\|mod v230000_tests\|mod v231000_tests" fav/src/driver.rs | head -5`
- [x] **T7-1（Cargo.toml バージョン更新）より前に実施**: `v230000_tests::version_is_23_0_0` に `#[ignore]` を追加
- [x] `v231000_tests` モジュールを `v230000_tests` の直後に追加（5 件、plan.md T6-2 のコードに従う）
  - `version_is_23_1_0`
  - `bytes_from_hex_to_hex_roundtrip`
  - `bytes_get_correct_byte`
  - `bytes_concat_increases_length`
  - `changelog_has_v23_1_0`
- [x] `cargo test v231000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1886 件以上合格）を確認

---

### T7: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意（T7-1 より前）**: T6-1（`v230000_tests::version_is_23_0_0` に `#[ignore]` 追加）が完了してから `Cargo.toml` を更新すること。

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリを確認
- [x] **注意**: T6-1（`#[ignore]` 追加）完了後に実施すること
- [x] `fav/Cargo.toml` の `version = "23.0.0"` → `"23.1.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.0.0 エントリの上）に v23.1.0 エントリを追加（plan.md T7-2）
- [x] `benchmarks/v23.1.0.json` を新規作成（plan.md T7-3）
- [x] `site/content/docs/runes/bytes.mdx` を新規作成（plan.md T7-4）
- [x] `cargo test v231000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1886 件以上合格）を再確認

---

## テスト一覧（v231000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_1_0` | Cargo.toml に `version = "23.1.0"` が含まれる |
| `bytes_from_hex_to_hex_roundtrip` | `from_hex("414243")` → pipeline → `to_hex` → `"414243"` |
| `bytes_get_correct_byte` | `from_hex("ff00")` の `get(0)` → `255` |
| `bytes_concat_increases_length` | 2 bytes + 3 bytes を concat → len 5 |
| `changelog_has_v23_1_0` | CHANGELOG.md に `[v23.1.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `v230000_tests::version_is_23_0_0` に `#[ignore]` が追加済み（T7-1 より前）
- [x] `HeapVal::Bytes(u64)` が `heap_val.rs` に追加される
- [x] `nan_val.rs` の VMValue ↔ NanVal 変換に Bytes が追加される
- [x] `VMValue::Bytes(u64)` と `BYTES_STORE` / `NEXT_BYTES_ID` が `vm.rs` に追加される
- [x] `vm_call_builtin` に `"Bytes.*"` の 13 ハンドラが追加される（flat literal アーム形式、`#[cfg]` はアーム本体内ブロック）
- [x] `Bytes.concat` は `VMValue::Bytes` を直接返す（Result ラップなし）
- [x] `is_known_builtin_namespace` に `"Bytes"` が追加される
- [x] `checker.rs` の namespace リストに `"Bytes"` が追加される
- [x] `compiler.rs` の builtins リストに `"Bytes.*"` 13 エントリが追加される
- [x] `Bytes.read_file` / `Bytes.write_file` に `#[cfg(not(target_arch = "wasm32"))]` が付く
- [x] `cargo test v231000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1886 件以上合格）
- [x] `CHANGELOG.md` に v23.1.0 エントリ
- [x] `benchmarks/v23.1.0.json` 作成済み
- [x] `site/content/docs/runes/bytes.mdx` 作成済み

---

## 優先度

```
T1（heap_val.rs）  ← 最初（HeapVal::Bytes の定義。以降の依存元）
T2（nan_val.rs）   ← T1 完了後
T3（vm.rs）        ← T2 完了後（最大タスク）
T4（checker.rs）   ← T3 と並列可
T5（compiler.rs）  ← T3 と並列可
T6（driver.rs）    ← T3〜T5 完了後、T7-1 より前に #[ignore] を実施
T7（docs）         ← T6 完了後（#[ignore] 確認後にバージョン更新）
```

---

## コードレビュー指摘と対応（実装後に修正済み）

| # | ラベル | 内容 | 対応 |
|---|--------|------|------|
| 1 | [HIGH] | `Bytes.get` 負インデックスを無検査で `as usize` キャスト | `idx < 0` チェックを追加して早期 err_vm 返し |
| 2 | [MED] | `Bytes.read_file/write_file` パス traversal バリデーションなし | `Path::components().ParentDir` チェックを追加 |
| 3 | [MED] | `Bytes.concat` の `unwrap_or_default` がサイレント空配列 | `None => Err(...)` に変更し runtime error に昇格 |
| 4 | [MED] | `BYTES_STORE` にエントリ削除なくメモリリーク | 既知制限コメント追加（v25.x で GC 検討） |
| 5 | [MED] | file I/O テスト欠如（read_file/write_file 未カバー） | `bytes_slice_returns_subrange`/`bytes_to_utf8_decodes_ascii`/`bytes_read_u16_big_endian`/`bytes_read_write_file_roundtrip` を追加（計 9 件） |
| 6 | [LOW] | `Bytes.to_hex` invalid handle がサイレント空文字列 | `None => Err(...)` に変更し runtime error に昇格 |
| 7 | [LOW] | `Bytes.slice` の負値 `as usize` キャスト | `if start/end < 0 { 0usize }` クランプを追加 |
| 8 | [LOW] | `bytes.mdx` の `bind <- Bytes.from_str(...)` が誤り | `from_str` を直接ネストして使う例に修正 |
| 9 | [LOW] | `benchmarks/v23.1.0.json` test_count 不一致 | 1891 → 1894 に修正（テスト追加後の実測値）|

テスト最終結果: 1894 passed / 0 failed（v231000_tests 9/9 PASS）

---

## spec-reviewer 指摘と対応（実装前に修正済み）

| # | ラベル | 内容 | 対応 |
|---|--------|------|------|
| 1 | [HIGH] | テストアサーションが `VMValue::*` を使用（`exec_artifact_main` は `Value::*` を返す） | `crate::value::Value::*` に修正 |
| 2 | [HIGH] | `vm_call_builtin` match が guard パターン `n if n.starts_with("Bytes.")` 形式 | flat literal アームに書き直し |
| 3 | [HIGH] | `bytes_concat_increases_length` で `bind c <- Bytes.concat(a, b)` — concat は Result でない | `Bytes.len(Bytes.concat(a, b))` ネスト呼び出しに修正 |
| 4 | [MED] | `#[cfg]` をマッチアーム属性として使用（stable Rust 不可） | アーム本体内 `#[cfg]` ブロックに移動 |
| 5 | [MED] | `Bytes.concat` の返り値が `ok_vm(...)` ラップになっていた | `VMValue::Bytes(...)` 直接返しに修正 |
| 6 | [LOW] | plan.md T6-1 の注意が「T2 より前」だった | 「T7-1 より前」に修正 |
| 7 | [LOW] | tasks.md に `#[ignore]` 追加チェック項目なし | 完了条件に追加 |

## 実装時の知見

| 項目 | 内容 |
|---|---|
| compiler.rs namespace 登録 | `"Bytes"` namespace を `"ArrowBatch"` と同様に単体で登録。個別関数名は不要 |
| `|_|` パラメータ | stage 定義の lambda では `_` 単独は不可。`|s|` 等の named param を使う |
| GetField の動作 | `Bytes.from_hex` は `Global("Bytes")` + `GetField "from_hex"` → runtime に `"Bytes.from_hex"` 文字列を生成 |
