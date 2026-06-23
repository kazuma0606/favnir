# v23.7.0 — vm.fav Phase 4（stdlib・builtin 呼び出し）タスク

## ステータス: COMPLETE（2026-06-22）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "fn vm_execute\|fn vm_run" fav/self/vm.fav` — 現シグネチャが `(bytecode, stack, locals, pc)` であること
- [x] `grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav | wc -l` — 再帰呼び出し件数確認
- [x] `grep -n "version = " fav/Cargo.toml` — `"23.6.0"` であること
- [x] `grep -n "mod v236000_tests\|mod v237000_tests" fav/src/driver.rs | head -3` — v237000_tests 未存在確認
- [x] `grep -n "VMStr\|call_builtin\|vm_run_named" fav/self/vm.fav` — 全 0 件（未実装を確認）
- [x] `grep -n "LoadGlobal" fav/src/backend/codegen.rs | head -3` — `LoadGlobal = 0x12` を確認
- [x] `grep -rn "String\.length\|String\.trim" fav/self/ | head -10` — checker.fav で使用されており利用可能であること
- [x] `grep -n "Phase 3\|Phase 4" site/content/docs/tools/vm-fav.mdx | head -5` — Phase 3 追記済み・Phase 4 未存在を確認

---

### T1: `fav/self/vm.fav` — Phase 4 変更

- [x] **T1-1**: `VMVal` 型に `VMStr(String)` バリアントを追加（`VMUnit` の後）
- [x] **T1-2**: `vmval_to_string` に `VMStr(s) => f"VMStr({s})"` アームを追加
- [x] **T1-3**: `fn call_builtin(name: String, args: Int) -> Result<VMVal, String>` を `vmval_to_string` の直後に追加
  - `"Int.to_string"`: VMInt(n) → VMStr(f"{n}")
  - `"String.length"`: VMStr(s) → VMInt(String.length(s))
  - `"String.trim"`: VMStr(s) → VMStr(String.trim(s))
  - `"Math.abs"`: VMInt(n) → VMInt（if n < 0: 0-n else n）
  - `_`: Result.err(f"call_builtin: unknown builtin: {name}")
- [x] **T1-4**: `fn vm_execute` の定義行に `globals: Int` パラメータを追加（`locals` の後）
- [x] **T1-5**: Phase 1〜3 全再帰呼び出し `vm_execute(bytecode, stack, locals, dec.next_pc)` を `vm_execute(bytecode, stack, locals, globals, dec.next_pc)` に一括更新
  - 更新対象（約 20〜25 件）: ConstUnit / ConstTrue / ConstFalse / Const / Pop ok / Dup ok→push ok / Add / Sub / Mul / Eq / Ne / Lt / Le / Gt / Ge / And / Or / Div / Jump / JumpIfFalse true/false / LoadLocal ok→push ok / StoreLocal ok→set ok
- [x] **T1-6**: Phase 4 オペコードを `_` アームの直前に追加（2 件）
  - `LoadGlobal(idx)`: `Mut.get(globals, idx)` → push
  - `Call(argc)`: `match argc { 0 => ... 1 => ... _ => err }` — call_builtin ディスパッチ
- [x] **コメント更新**: `// LoadGlobal / Call / GetField は Phase 4 以降で実装予定` → `// GetField は Phase 5 以降で実装予定`
- [x] **T1-7**: `fn vm_run` に `bind globals <- Mut.map()` 追加 + `vm_execute` 呼び出しに `globals` を渡す
- [x] **T1-7**: `fn vm_run_named(bytecode: Bytes, globals: Int) -> Result<VMVal, String>` を `vm_run` の直後に追加
- [x] **事後確認**: `cargo check --bin fav` でコンパイルエラー 0
- [x] **事後確認**: `grep -n "vm_execute(bytecode, stack, locals, dec.next_pc)" fav/self/vm.fav` → 0 件（更新漏れなし）
- [x] **後方互換確認**: `cargo test v236000 --bin fav` — 5/5 PASS（vm_run の後方互換性確認）

---

### T2: `fav/src/driver.rs` — `v237000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_6_0\|#\[ignore\]" fav/src/driver.rs | head -10`
- [x] **T2-1（T3-1 より前に必須）**: `v236000_tests::version_is_23_6_0` に `#[ignore]` を追加
- [x] **T2-2**: `v237000_tests` モジュールを `v236000_tests` の直後に追加（5 件）
  - `version_is_23_7_0`
  - `vm_fav_phase4_compiles`
  - `vmstr_to_string_variant`（`vmval_to_string(VMStr("hello"))` → `"VMStr(hello)"`）
  - `execute_builtin_call`（hex `"120000012a0015010016"` + globals[0]="Int.to_string" → `"VMStr(42)"`）
  - `changelog_has_v23_7_0`
- [x] `cargo test v237000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1917 件以上合格）を確認

---

### T3: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意**: T2-1 の `#[ignore]` 追加完了後に Cargo.toml を更新すること。

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリ確認
- [x] `fav/Cargo.toml` の `version = "23.6.0"` → `"23.7.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.6.0 エントリの上）に v23.7.0 エントリを追加
- [x] `benchmarks/v23.7.0.json` を新規作成（`test_count` は最終 `cargo test --bin fav` 実行後に実際の件数で更新）
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 4 セクション追記 + フェーズ表更新
- [x] `cargo test v237000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし再確認

---

## テスト一覧（v237000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_7_0` | Cargo.toml に `version = "23.7.0"` | — |
| `vm_fav_phase4_compiles` | vm.fav を parse + build_artifact | エラーなし |
| `vmstr_to_string_variant` | `vmval_to_string(VMStr("hello"))` | `"VMStr(hello)"` |
| `execute_builtin_call` | hex `"120000012a0015010016"` + globals[0]="Int.to_string" → vm_run_named | `"VMStr(42)"` |
| `changelog_has_v23_7_0` | CHANGELOG.md に `[v23.7.0]` | — |

---

## 完了条件チェックリスト

- [x] `v236000_tests::version_is_23_6_0` に `#[ignore]` が追加済み（T3-1 より前）
- [x] `VMVal` に `VMStr(String)` バリアントが追加済み（4 バリアント）
- [x] `vmval_to_string` に `VMStr` アームが追加済み
- [x] `fn call_builtin` が追加済み（4 builtin）
- [x] `fn vm_execute` に `globals: Int` パラメータが追加済み
- [x] Phase 1〜3 全再帰呼び出しが `globals` を渡す（更新漏れ 0 件）
- [x] `LoadGlobal(idx)` / `Call(0)` / `Call(1)` が実装済み
- [x] `fn vm_run` が `globals <- Mut.map()` を生成
- [x] `fn vm_run_named` が追加済み
- [x] `cargo test v237000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1917 件以上合格）
- [x] `CHANGELOG.md` に v23.7.0 エントリ
- [x] `benchmarks/v23.7.0.json` 作成済み
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 4 セクション追記済み

---

## 優先度

```
T0（事前確認）         ← 最初（String.length 可用性を確認）
T1-1（VMStr 追加）     ← T0 完了後
T1-2（vmval_to_string）← T1-1 完了後
T1-3（call_builtin）   ← T1-2 完了後
T1-4（シグネチャ変更） ← T1-3 完了後
T1-5（再帰呼び出し更新）← T1-4 完了後（最多作業・漏れ注意）
T1-6（LoadGlobal/Call）← T1-5 完了後
T1-7（vm_run 更新）    ← T1-6 完了後
cargo check            ← T1 全完了後
T2-1（#[ignore]）      ← T3-1 より前（必須）
T2-2（tests）          ← T1 + cargo check 完了後
T3-1（version）        ← T2-1 完了後
T3-2〜4（docs）        ← T3-1 完了後
```

---

## 実装時の注意事項

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `match argc { 0 => ... 1 => ... _ => ... }` | 整数リテラル match は Favnir でサポート済み（v23.6.0 で `match bi { 0 => ... _ => ... }` 確認済み） |
| 2 | `Mut.get(globals, idx)` の idx: Int キー | MutMap は VMValue::Int(n) としてキー照合（locals と同じパターン）|
| 3 | `Call(1)` の引数順序 | pop once = 唯一の引数（Call(1) の場合は逆順問題なし） |
| 4 | `VMStr("Int.to_string")` をテスト Favnir コードで記述 | `VMStr` バリアントは vm.fav 由来。テストコードに vm_src を concat しているため使用可能 |
| 5 | `String.length` / `String.trim` が型エラーの場合 | `_ => Result.err("not implemented")` で一時無効化し、next step で checker.fav に追加 |
| 6 | `#[ignore]` 追加順序 | Cargo.toml 更新（T3-1）前に必ず T2-1 を完了させること |

---

## コードレビュー対応（2026-06-22）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [HIGH] | ロードマップの `call_builtin` シグネチャ（`List<VMValue>`）と実装（`Int` handle）の乖離 | spec.md「ロードマップとの関係」に乖離対応表を追加（opaque handle パターンへの変更理由を明記） |
| [HIGH] | ロードマップ v23.6 定義（CallFrame）と実際の実装（制御フロー）の乖離 | spec.md に「v23.5.0 の型チェッカー制約による再定義」を明記 |
| [HIGH] | `Const(n)` オペランドが直接値か定数プールインデックスか不明 | plan.md T0 に確認コマンド追加（既存テストで実証済みと注記） |
| [MED] | T0 grep コマンドが `checker.fav` を含んでいない | plan.md T0 を `-rn fav/self/` に修正 |
| [MED] | T1 事後確認に後方互換テストがなかった | `cargo test v236000` を T1 事後確認に追加 |
| [MED] | `Mut.set(globals, 0, ...)` のキー型リスク | plan.md リスク対応表に説明追加 |
| [BUG] | `vm_execute(bytecode, stack, locals, dec.next_pc + off)` の更新漏れ（Jump / JumpIfFalse false branch） | replace_all パターンが `+ off` を含む呼び出しにマッチしなかった。別途 replace_all で修正 |
| [BUG] | f-string `f"VMStr({s})"` で String が引用符付き出力 (`VMStr("hello")`) になる | `vmval_to_string` の VMStr アームを `String.concat("VMStr(", String.concat(s, ")"))` に変更 |
| [MED] | `Call(argc)` の引数積み順（LIFO）が将来の argc>=2 実装者に不明瞭 | `Call` アームの直前にスタック呼び出し規約コメントを追加（Phase 5: collect_args 方針を明記） |
| [LOW] | `Math.abs` の `Int.MIN_VALUE` オーバーフローにコメントなし | `"Math.abs"` アームの直前に既知制限コメントを追加 |
