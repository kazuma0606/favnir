# v23.8.0 — vm.fav Phase 5（GetField・collect_args・hello.fav 実行）タスク

## ステータス: COMPLETE（2026-06-22）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "fn vm_execute\|fn vm_run\|fn vm_run_named\|fn call_builtin" fav/self/vm.fav` — シグネチャが `(bytecode, stack, locals, globals, pc)` であること
- [x] `grep -n "GetField\|collect_args\|vmval_display" fav/self/vm.fav` — 全 0 件（未実装）
- [x] `grep -n "version = " fav/Cargo.toml` — `"23.7.0"` であること
- [x] `grep -n "mod v237000_tests\|mod v238000_tests" fav/src/driver.rs | head -3` — v238000_tests 未存在確認
- [x] `grep -n "GetField" fav/src/backend/codegen.rs | head -5` — `GetField = 0x40` を確認
- [x] `grep -rn "String\.concat" fav/self/ | head -5` — checker.fav で使用されており利用可能であること
- [x] `grep -n "Phase 4\|Phase 5" site/content/docs/tools/vm-fav.mdx | head -5` — Phase 4 追記済み・Phase 5 未存在を確認

---

### T1: `fav/self/vm.fav` — Phase 5 変更

- [x] **T1-1**: `fn collect_args_rec` を `call_builtin` の直後・`vm_execute` の直前に追加
  - `match n { 0 => Result.ok(acc) _ => { bind v_r <- Mut.pop(stack) ... collect_args_rec(stack, n - 1, acc) } }`
- [x] **T1-2**: `fn collect_args` を `collect_args_rec` の直後に追加
  - `bind acc <- Mut.list()` + `collect_args_rec(stack, n, acc)`
- [x] **T1-3**: `fn vmval_display` を `vmval_to_string` の直後に追加
  - `VMInt(n) => f"{n}"` / `VMBool(b) => f"{b}"` / `VMUnit => ""` / `VMStr(s) => s`
- [x] **T1-4**: `call_builtin` の `_ =>` アームの直前に `"String.concat"` を追加
  - `Mut.pop(args)` → a、`Mut.pop(args)` → b、`String.concat(sa, sb)` → `VMStr`
- [x] **T1-5**: `GetField(idx)` ハンドラを実装
  - `// GetField は Phase 5 以降で実装予定` コメントを置換
  - pop VMStr(ns_name) + `Mut.get(globals, idx)` → VMStr(field_name) → push `VMStr(String.concat(ns_name, String.concat(".", field_name)))`
- [x] **T1-6**: `Call(argc)` ハンドラを `collect_args` 利用の汎用実装に置換
  - 既存 `match argc { 0 => ... 1 => ... _ => err }` を削除
  - `collect_args(stack, argc)` → `Mut.pop(stack)` (callee) → `call_builtin(name, args)` → push result
- [x] **コメント確認**: `"not yet supported in Phase 4"` 等の古いコメントが残っていないこと
- [x] **事後確認**: `cargo check --bin fav` でコンパイルエラー 0
- [x] **事後確認**: `grep -n "fn collect_args_rec\|fn collect_args\|fn vmval_display\|GetField" fav/self/vm.fav | head -10`
- [x] **後方互換確認**: `cargo test v237000 --bin fav` — 5/5 PASS

---

### T2: `fav/src/driver.rs` — `v238000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_7_0" fav/src/driver.rs | head -5`
- [x] **T2-1（T3-1 より前に必須）**: `v237000_tests::version_is_23_7_0` テスト関数を**削除**（`#[ignore]` 蓄積防止のため）
- [x] **T2-2**: `v238000_tests` モジュールを `v237000_tests` の直後に追加（6 件）
  - `version_is_23_8_0`
  - `vm_fav_phase5_compiles`
  - `execute_hello_via_vm`（hex `"12000016"` + globals[0]=VMStr("hello") → `vmval_display` = `"hello"`）
  - `execute_getfield_call`（hex `"12000040010012020015010016"` + globals={0:"String",1:"trim",2:" hi "} → `"hi"`）
  - `execute_string_concat`（hex `"12000040010012020012030015020016"` + 4 globals → `"hello world"`）
  - `changelog_has_v23_8_0`
- [x] `cargo test v238000 --bin fav` — 6/6 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1921 件以上合格）を確認

---

### T3: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意**: T2-1 の `version_is_23_7_0` 削除完了後に Cargo.toml を更新すること。

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリ確認
- [x] `fav/Cargo.toml` の `version = "23.7.0"` → `"23.8.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.7.0 エントリの上）に v23.8.0 エントリを追加
- [x] `benchmarks/v23.8.0.json` を新規作成（`test_count` は最終 `cargo test --bin fav` 実行後に実際の件数で更新）
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 5 セクション追記 + フェーズ表更新
- [x] `cargo test v238000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし再確認

---

## テスト一覧（v238000_tests、6 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_8_0` | Cargo.toml に `version = "23.8.0"` | — |
| `vm_fav_phase5_compiles` | vm.fav を parse + build_artifact | エラーなし |
| `execute_hello_via_vm` | hex `"12000016"` + globals[0]=VMStr("hello") → `vmval_display` | `"hello"` |
| `execute_getfield_call` | hex `"12000040010012020015010016"` + 3 globals → `vmval_display` | `"hi"` |
| `execute_string_concat` | hex `"12000040010012020012030015020016"` + 4 globals → `vmval_display` | `"hello world"` |
| `changelog_has_v23_8_0` | CHANGELOG.md に `[v23.8.0]` | — |

---

## 完了条件チェックリスト

- [x] `fn collect_args_rec` が追加される
- [x] `fn collect_args` が追加される
- [x] `fn vmval_display` が追加される
- [x] `call_builtin` に `"String.concat"` が追加される
- [x] `GetField(idx)` が実装される（namespace + "." + field_name）
- [x] `Call(argc)` ハンドラが `collect_args` を使った単一実装に置換される
- [x] `v237000_tests::version_is_23_7_0` が削除済み（`#[ignore]` 蓄積防止のため、T3-1 より前）
- [x] `cargo test v238000 --bin fav` — 6/6 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1921 件以上合格）
- [x] `CHANGELOG.md` に v23.8.0 エントリ
- [x] `benchmarks/v23.8.0.json` 作成済み
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 5 セクション追記済み

---

## 優先度

```
T0（事前確認）         ← 最初（GetField=0x40, String.concat 可用性を確認）
T1-1（collect_args_rec）← T0 完了後
T1-2（collect_args）   ← T1-1 完了後
T1-3（vmval_display）  ← T1-2 完了後（vm_execute より前の位置に追加）
T1-4（String.concat）  ← T1-3 完了後（call_builtin への追記）
T1-5（GetField）       ← T1-4 完了後（_ アーム直前のコメントを置換）
T1-6（Call 汎用化）    ← T1-5 完了後（既存 match argc 全体を置換）
cargo check            ← T1 全完了後
T2-1（version_is_23_7_0 削除）← T3-1 より前（必須）
T2-2（tests）          ← T1 + cargo check 完了後
T3-1（version）        ← T2-1 完了後
T3-2〜4（docs）        ← T3-1 完了後
```

---

## 実装時の注意事項

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `collect_args_rec` の `match n { 0 => ... _ => ... }` | 整数リテラル match は Favnir でサポート済み（Div アームの `match bi { 0 => ... }` で確認済み） |
| 2 | `GetField` の文字列連結に f-string 不可 | `String.concat(ns_name, String.concat(".", field_name))` を使う（f-string は String 型に引用符を付けるため） |
| 3 | `Call(argc)` 汎用化後の collect_args の argc=0 動作 | `collect_args(stack, 0)` は `collect_args_rec(stack, 0, acc)` → `Result.ok(acc)`（空リスト）→ callee pop → call_builtin(name, []) で正常動作 |
| 4 | `vmval_display` の `VMStr(s) => s` | f-string でなく直接 `s` を返す（引用符なし。vmval_to_string の `String.concat("VMStr(", ...)` と異なる） |
| 5 | `bind _ <- Mut.push(acc, v)` の sequencing | `Mut.push` は `Result<Unit, String>` 返却。`bind _ <-` は unwrap しない（Favnir の仕様）。MutList ハンドルが常に有効なため実際には失敗しない。コメントで明記 |
| 6 | `version_is_23_7_0` 削除順序 | Cargo.toml 更新（T3-1）前に必ず T2-1 を完了させること |

---

## コードレビュー対応（2026-06-22）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `collect_args_rec` の `bind _ <- Mut.push(acc, v)` がエラー非伝播である旨の注釈がない | `collect_args_rec` 内にコメント追記（Favnir の `bind _` は unwrap しない仕様、MutList は常に有効） |
| [LOW] | `Unknown(b)` opcode が `_ =>` に落ちてバイト番号がエラーメッセージに含まれない | デバッグ品質の問題。機能影響なし。スキップ（Phase 6 で整理予定） |
| [LOW] | `Add`/`Sub`/`Mul` と比較演算の push エラーハンドリングが非対称 | 既存コードのスタイル問題。機能影響なし。スキップ |
