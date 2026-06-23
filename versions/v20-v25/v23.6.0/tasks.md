# v23.6.0 — vm.fav Phase 3（制御フロー・ローカル変数）タスク

## ステータス: COMPLETE（2026-06-22）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "fn vm_execute\|fn vm_run" fav/self/vm.fav` — 現シグネチャ確認
- [x] `grep -n "vm_execute(bytecode, stack" fav/self/vm.fav | wc -l` — 再帰呼び出し件数確認
- [x] `grep -n "version = " fav/Cargo.toml` — `"23.5.0"` であること
- [x] `grep -n "mod v235000_tests\|mod v236000_tests" fav/src/driver.rs | head -3` — v236000_tests 未存在確認
- [x] `grep -n "AmpAmp\|PipePipe" fav/src/frontend/lexer.rs | head -3` — `&&`/`||` サポート確認
- [x] `ls site/content/docs/tools/vm-fav.mdx` — Phase 2 セクション追記済みファイルが存在すること

---

### T1: `fav/self/vm.fav` — Phase 3 変更

- [x] **T1-1**: `fn vm_execute` の定義行に `locals: Int` パラメータを追加
- [x] **T1-1**: Phase 2 全アームの再帰呼び出し `vm_execute(bytecode, stack, dec.next_pc)` を `vm_execute(bytecode, stack, locals, dec.next_pc)` に一括更新
  - 更新対象: ConstUnit / ConstTrue / ConstFalse / Const(n) / Pop ok アーム / Dup ok アーム / Add / Sub / Mul / Eq
- [x] **T1-1**: `fn vm_run` を `bind locals <- Mut.map()` + `vm_execute(bytecode, stack, locals, 0)` に更新
- [x] **T1-2**: Phase 3 オペコードを `_` アームの直前に追加（12 件）
  - Jump / JumpIfFalse
  - LoadLocal / StoreLocal
  - Ne / Lt / Le / Gt / Ge
  - And / Or
  - Div（ゼロ除算 match bi { 0 => err _ => ok } を含む）
- [x] **事後確認**: `cargo check --bin fav` でコンパイルエラー 0
- [x] **事後確認**: `grep -n "vm_execute(bytecode, stack, dec.next_pc)" fav/self/vm.fav` → 0 件（更新漏れなし）

---

### T2: `fav/src/driver.rs` — `v236000_tests` 追加

- [x] **事前確認**: `grep -n "fn version_is_23_5_0\|#\[ignore\]" fav/src/driver.rs | head -10`
- [x] **T2-1（T3-1 より前に必須）**: `v235000_tests::version_is_23_5_0` に `#[ignore]` を追加
- [x] **T2-2**: `v236000_tests` モジュールを `v235000_tests` の直後に追加（5 件）
  - `version_is_23_6_0`
  - `vm_fav_phase3_compiles`
  - `execute_locals`（hex `"012a0011000010000016"` → `"VMInt(42)"`）
  - `execute_jump`（hex `"0431060001010030030001020016"` → `"VMInt(2)"`）
  - `changelog_has_v23_6_0`
- [x] `cargo test v236000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1917 件以上合格）を確認

---

### T3: Cargo.toml + CHANGELOG + benchmarks + MDX

> **注意**: T2-1 の `#[ignore]` 追加完了後に Cargo.toml を更新すること。

- [x] **事前確認**: `grep "\[v23\." CHANGELOG.md | head -5` で先頭エントリ確認
- [x] `fav/Cargo.toml` の `version = "23.5.0"` → `"23.6.0"` に変更
- [x] `CHANGELOG.md` 先頭（v23.5.0 エントリの上）に v23.6.0 エントリを追加
- [x] `benchmarks/v23.6.0.json` を新規作成（test_count は実行後に確定）
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 3 セクション追記 + フェーズ表更新
- [x] `cargo test v236000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし再確認

---

## テスト一覧（v236000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_23_6_0` | Cargo.toml に `version = "23.6.0"` | — |
| `vm_fav_phase3_compiles` | vm.fav を parse + build_artifact | エラーなし |
| `execute_locals` | `"012a0011000010000016"` → vm_run | `"VMInt(42)"` |
| `execute_jump` | `"0431060001010030030001020016"` → vm_run | `"VMInt(2)"` |
| `changelog_has_v23_6_0` | CHANGELOG.md に `[v23.6.0]` | — |

---

## 完了条件チェックリスト

- [x] `v235000_tests::version_is_23_5_0` に `#[ignore]` が追加済み（T3-1 より前）
- [x] `fn vm_execute` が `locals: Int` パラメータを持つ
- [x] Phase 2 全アームの再帰呼び出しが `locals` を渡す
- [x] `fn vm_run` が `Mut.map()` でローカル変数マップを生成する
- [x] Jump / JumpIfFalse / LoadLocal / StoreLocal / Ne / Lt / Le / Gt / Ge / And / Or / Div（計 12 件）が追加される
- [x] `cargo test v236000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1917 件以上合格）
- [x] `CHANGELOG.md` に v23.6.0 エントリ
- [x] `benchmarks/v23.6.0.json` 作成済み
- [x] `site/content/docs/tools/vm-fav.mdx` に Phase 3 セクション追記済み

---

## 優先度

```
T0（事前確認）        ← 最初
T1-1（シグネチャ更新）← T0 完了後（Phase 2 全アーム更新を含む）
T1-2（Phase 3 追加）  ← T1-1 完了後
T2-1（#[ignore]）     ← T3-1 より前（必須）
T2-2（tests）         ← T1 完了後
T3-1（version）       ← T2-1 完了後
T3-2〜4（docs）       ← T3-1 完了後
```

---

## 実装時の注意事項

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `match b { true => ... false => ... }` | Favnir で bool リテラルパターンは使用可能（parser.rs:2511 `Pattern::Lit(Lit::Bool)`）。そのまま使う |

---

## コードレビュー対応（2026-06-22）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `Eq` アームの `Mut.push` 結果が `bind _ <-` でエラー未検査（Phase 3 の Ne〜Div と不整合） | `bind push_r <- Mut.push(...)` + `match push_r { err ... ok ... }` に修正 |
| [LOW] | `_ => Result.err("vm_execute: unimplemented opcode")` に意図コメントなし | `// LoadGlobal / Call / GetField は Phase 4 以降で実装予定` コメント追加 |
| 2 | `&&`/`||` の Favnir サポート | lexer.rs に AmpAmp/PipePipe が存在するため使用可能 |
| 3 | `Mut.set(locals, slot, val)` の Int キー | MutMap は VMValue キー → slot:Int が VMValue::Int(n) として正しく照合される |
| 4 | 再帰呼び出しの更新漏れ | `cargo check` がシグネチャ不一致でエラーを出す（Favnir の型チェッカーが検出） |
| 5 | `#[ignore]` 追加順序 | Cargo.toml 更新前に必ず追加すること |
| 6 | `Div` の `VMInt(0)` パターン | `VMInt(bi) => match bi { 0 => err _ => ... }` でゼロ除算処理 |
