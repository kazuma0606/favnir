# v23.5.0 — vm.fav Phase 2（スタックベース実行ループ）タスク

## ステータス: COMPLETE（2026-06-22）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "Mut\.push\|Mut\.pop\|Mut\.peek\|Mut\.len" fav/src/backend/vm.rs | head -10` — 全 primitive が実装済みであること
- [x] `grep -n "version = " fav/Cargo.toml` — `"23.4.0"` であること
- [x] `grep -n "mod v234000_tests\|mod v235000_tests" fav/src/driver.rs` — v235000_tests 未存在を確認
- [x] `grep -n 'Pat::Wildcard\|"_"\|bind.*_' fav/src/frontend/parser.rs | head -10` — `bind _ <-` の文法サポート確認

---

### T1: `fav/self/vm.fav` — Phase 2 追記

- [x] **事前確認**: `tail -10 fav/self/vm.fav` で現在の末尾を確認（`opcode_to_string` で終わること）
- [x] `type VMVal` を追記（VMInt / VMBool / VMUnit の 3 バリアント）
- [x] `fn vmval_to_string(v: VMVal) -> String` を追記
- [x] `fn vm_execute(bytecode: Bytes, stack: Int, pc: Int) -> Result<VMVal, String>` を追記
  - ConstUnit / ConstTrue / ConstFalse / Const(n) アーム
  - Pop / Dup / Return アーム
  - Add / Sub / Mul / Eq アーム
  - `_` アーム: `Result.err("vm_execute: unimplemented opcode")`
- [x] `fn vm_run(bytecode: Bytes) -> Result<VMVal, String>` を追記
- [x] `cargo check --bin fav` — コンパイルエラー 0 を確認

---

### T2: `fav/src/driver.rs` — `v235000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_4_0\|#\[ignore\]" fav/src/driver.rs | head -10`
- [x] **T3-1 より前に実施**: `v234000_tests::version_is_23_4_0` に `#[ignore]` を追加
- [x] `v235000_tests` モジュールを `v234000_tests` の直後に追加（5 件）
  - `version_is_23_5_0`
  - `vm_fav_phase2_compiles`
  - `execute_const_unit`（bytecode `"0216"` → `"VMUnit"`）
  - `execute_add`（bytecode `"0103000104002016"` → `"VMInt(7)"`）
  - `changelog_has_v23_5_0`
- [x] `cargo test v235000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1909 件以上合格）を確認

---

### T3: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意（T3-1 より前）**: T2-1 の `#[ignore]` 追加が完了してから `Cargo.toml` を更新すること。

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリを確認
- [x] `fav/Cargo.toml` の `version = "23.4.0"` → `"23.5.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.4.0 エントリの上）に v23.5.0 エントリを追加
- [x] `benchmarks/v23.5.0.json` を新規作成（test_count は実行後に確定）
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 2 セクション追記
- [x] `cargo test v235000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし再確認

---

## テスト一覧（v235000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_23_5_0` | Cargo.toml に `version = "23.5.0"` が含まれる |
| `vm_fav_phase2_compiles` | vm.fav を parse + build_artifact し、エラーなく完了 |
| `execute_const_unit` | hex `"0216"` → vm_run → vmval_to_string → `"VMUnit"` |
| `execute_add` | hex `"0103000104002016"` → vm_run → vmval_to_string → `"VMInt(7)"` |
| `changelog_has_v23_5_0` | CHANGELOG.md に `[v23.5.0]` が含まれる |

---

## 完了条件チェックリスト

- [x] `v234000_tests::version_is_23_4_0` に `#[ignore]` が追加済み（T3-1 より前）
- [x] `type VMVal` — 3 バリアント（VMInt / VMBool / VMUnit）が vm.fav に追加される
- [x] `fn vmval_to_string` が vm.fav に追加される
- [x] `fn vm_execute` — 11 オペコード対応が vm.fav に追加される
- [x] `fn vm_run` が vm.fav に追加される
- [x] `cargo test v235000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1909 件以上合格）
- [x] `CHANGELOG.md` に v23.5.0 エントリ
- [x] `benchmarks/v23.5.0.json` 作成済み
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 2 セクション追記済み

---

## 優先度

```
T0（事前確認）        ← 最初
T1（vm.fav 追記）     ← T0 完了後
T2-1（#[ignore]）     ← T3-1 より前（必須）
T2-2（tests）         ← T1 完了後
T3-1（version）       ← T2-1 完了後
T3-2〜4（docs）       ← T3-1 完了後
```

---

## 実装時の注意事項

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `bind _ <- Mut.push(stack, val)` | `_` が不可なら `bind _skip <-` で代替 |
| 2 | `stack: Int` と宣言 | MutList handle は Type::Unknown → Int に unify される |
| 3 | `Result.ok(...)` / `Result.err(...)` | bare `ok`/`err` は絶対使わない |
| 4 | `bind stack <- Mut.list()` | Mut.list() は直接 handle を返す（Result でない） |
| 5 | `Const(n)` = Phase 2 簡略 | u16 オペランドを整数値として push |
| 6 | `#[ignore]` 追加順序 | Cargo.toml 更新前に必ず追加すること |
| 7 | test helper | `run_source_get_output` は存在しない。`Lexer::new → Parser::new → build_artifact → exec_artifact_main → Value::Str` パターンを使う（v23.4.0 と同一） |

---

## 実装完了メモ（2026-06-22）

- 1913 tests pass（0 failures）
- v235000_tests: 5/5 PASS
- `bind _ <- Mut.push(stack, val)` は `Pattern::Wildcard` としてサポート済み（parser.rs line 2484）
- `stack: Int` 型宣言で MutList handle を受け渡し可能（Type::Unknown が Int に unify）
- `bind stack <- Mut.list()` は Result ではなく直接 handle を返す → bind で直接束縛
- spec-reviewer 指摘（8 件）対応: `run_source_get_output` 未存在 / `parser::parse` 誤パス / Cargo.toml include_str パス → plan.md 修正済み

## コードレビュー対応（2026-06-22）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [HIGH] | `Pop` アームが空スタックエラーを握り潰す | `bind _ <- Mut.pop` → `match top_r { err(e) => Result.err(...) ok(_) => ... }` に修正 |
| [HIGH] | 算術アームの二重 pop: b pop 失敗でも a が先に pop される | `bind b_r <- Mut.pop` → `match b_r { ok(b) => { bind a_r <- Mut.pop ... } }` に再構成 |
| [HIGH] | `version_is_23_4_0` テスト破損と言われたが `#[ignore]` は正しく適用済み | 対応不要（reviewer の確認漏れ） |
| [MED] | `Eq` の `_` アームが `"Eq: unsupported type"` で `Add` と不一致 | `"Eq: type error on a"` に統一 |
| [LOW] | `Dup` の push エラーを握り潰す | `bind push_r <- Mut.push` → `match push_r { err(e) => Result.err(...) ok(_) => ... }` に修正 |
| [LOW] | Phase 2 固有テスト（`vm_run`/`vm_execute` 存在確認）が欠如 | Phase 3 テスト追加時に対応予定 |
