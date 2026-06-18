# Favnir v9.1.0 Tasks

Date: 2026-05-31
Theme: stdlib 拡充 + マルチパラメータクロージャ + rvm 独立バイナリ
完了日: 2026-06-01  最終テスト数: 1162

---

## Phase A: `rvm` 独立バイナリ

- [x] A-1: `fav/src/vm.rs` に `pub const VM_VERSION: &str = "1.0.0"` を追加
- [x] A-2: `fav/Cargo.toml` に `[[bin]] name = "rvm" path = "src/bin/rvm.rs"` を追加
- [x] A-3: `fav/src/bin/rvm.rs` を新規作成
  — `--version` → `"Favnir VM 1.0.0"` を表示
  — `--help` → 使い方を表示
  — `--db <url>` → DB URL を渡す
  — `<file.fvc>` → `fav_core::exec_fvc_file` に委譲
- [x] A-4: `cargo build` — `rvm` バイナリが生成されること確認
- [x] A-5: `rvm --version` → `Favnir VM 1.0.0` ✓

**実装メモ**: `src/lineage.rs` を新設して `driver.rs` からリネージ関連コードを分離。
`lib.rs` に backend/checker/compiler/stdlib_fav_runner を公開して `rvm` バイナリが
`fav_core::` 経由でアクセスできるようにした。

---

## Phase B: stdlib 実装（Rust builtin として vm.rs に追加）

> 計画では `.fav` ファイル実装だったが、既存 vm.rs builtin パターンに合わせ
> Rust builtin として実装。型シグネチャは checker.rs / checker.fav に登録済み。

### B-1: List 追加関数

- [x] B-1-3: `List.group_by` — キーで分類 → `Map<String, List<A>>`（Rust builtin 追加）
- [x] B-1-4: `List.zip_with` — 2リストを f で合成（Rust builtin 追加）
- [x] B-1-1/2/5/6/7/8/9/10/11: `List.chunk` / `flat_map` / `take_while` / `drop_while` /
  `unique` / `count` / `sum` / `min` / `max` — 既存 Rust builtin で対応済み

### B-2: String 追加関数

- [x] B-2-3: `String.truncate(s, max, suffix)` — Rust builtin 追加
- [x] B-2-5: `String.trim_start` — Rust builtin 追加
- [x] B-2-6: `String.trim_end` — Rust builtin 追加
- [x] B-2-1/2/4/7: `pad_left` / `pad_right` / `repeat` / `replace` — 既存対応済み

### B-3: Map 追加関数

- [x] B-3-1: `Map.merge_with` — Rust builtin 追加
- [x] B-3-2: `Map.filter` — Rust builtin 追加
- [x] B-3-3/4/5: `map_values` / `from_list` / `to_list` — 既存対応済み

### B-4: Result / Option 追加関数

- [x] B-4-3: `Result.all` — Rust builtin 追加
- [x] B-4-1/2/4/5/6/7/8: `map_err` / `and_then` / `Option.*` — 既存対応済み

---

## Phase C: 型シグネチャ登録 + VM ディスパッチ

- [x] C-1: `fav/src/middle/checker.rs` — 新関数のシグネチャ追加
  (`List.zip_with` / `List.group_by` / `String.truncate` / `String.trim_start` /
   `String.trim_end` / `Map.filter` / `Map.merge_with` / `Result.all`)
- [x] C-2: `fav/self/checker.fav` — `builtin_ret_ty` に新関数の戻り型追加
  (`str_fn`: truncate/trim_start/trim_end、`map_fn`: filter、`res_fn`: all)
- [ ] C-3: `fav/src/stdlib_fav_runner.rs` — `call_map_stdlib` / `call_result_stdlib` 追加
  — Phase B を Rust builtin で実装したため不要。スキップ。
- [x] C-4: `fav/src/vm.rs` — 新関数を Rust builtin として `impl VM::call_builtin` に追加
- [x] C-5: `cargo build` — コンパイルエラーなし、警告なし

---

## Phase D: E0012 — 非ジェネリック関数引数数チェック

> v8.8.0 で `fn_to_scheme_str` が全関数を `"forall|..."` 形式でスキーム化済み。
> 非ジェネリック関数も E0008 で引数数チェックされるため E0012 は不要。
> 代わりに E0008 の非ジェネリックケースのテストを追加して確認。

- [x] D-1〜D-6: 既存 E0008 で対応済み（`nongeneric_wrong_arity_e0008` テストが証明）
  — v8.8.0 完了内容として確認済み

---

## Phase E: マルチパラメータクロージャ self-hosted 対応

> Rust AST は既に `Expr::Closure(Vec<String>, ...)` で複数パラメータ対応済み。
> ELambda 型変更は不要。カリー化デシュガーを 2 箇所に追加するアプローチを採用。

- [x] E-1: `fav/src/ast.rs` — `Expr::Closure(Vec<String>, ...)` 確認。変更不要。
- [x] E-2: `fav/self/compiler.fav` — `parse_lambda_rest` ヘルパーを追加
  — `|x, y| body` → `ELambda("x", ELambda("y", body))` へのカリー化デシュガー
- [x] E-3: `fav/self/checker.fav` — `ELambda(String, Expr)` のまま変更不要
  （カリー化済みのネスト ELambda を型推論が順に処理）
- [x] E-4: 実装は E-2 (`parse_lambda_rest`) で完了
- [x] E-5: `fav/src/middle/ast_lower_checker.rs` — `Closure(params, body)` を
  `params.len() > 1` の場合に右畳みカリー化して `ELambda` を生成
- [x] E-6: 動作確認
  — `|x| x + 1`（単引数）引き続き動作 ✓
  — `|x, y| x + y` が Favnir pipeline で動作 ✓（`multi_param_closure_self_hosted` テスト）
  — `List.zip_with(|x, y| x + y, ...)` は curried 呼び出し規約で動作 ✓

**注意**: `List.fold` / `List.scan` 等は flat 2-arg 呼び出し規約のため、
self-hosted pipeline で `|acc, x|` を使う場合は curried 形式 `|acc| |x|` を推奨。
Rust pipeline では `|acc, x|` が直接動作する。

---

## Phase F: テスト追加

- [x] F-1: `rvm_version_constant` — `VM_VERSION == "1.0.0"` アサーション ✓
- [x] F-2: `stdlib_v91_list_tests` — List 新関数テスト 9 件 (zip_with / group_by / sum /
  min / max / unique / take_while / drop_while / flat_map)
- [x] F-3: `stdlib_v91_string_tests` — String テスト (truncate / trim_start / trim_end /
  pad_left / pad_right / repeat)
- [x] F-4/F-5: 非ジェネリック E0008 テストは v8.8.0 で追加済み。スキップ。
- [x] F-6: `multi_param_closure_map` / `multi_param_closure_self_hosted` ✓

**追加テストモジュール**:
- `stdlib_v91_tests` (9 件): zip_with / group_by / truncate / trim / result_all / map_filter / merge_with
- `stdlib_v91_extra_tests` (13 件): sum / min / max / unique / take_while / drop_while / flat_map / pad_left / pad_right / repeat / option_map / option_unwrap_or
- `rvm_tests` (2 件): version_constant / exec_fvc_file

---

## Phase G: 最終確認・ドキュメント

- [x] G-1: `cargo test rvm` — 2 件通過 ✓
- [x] G-2: `cargo test stdlib_v91` — 9 件通過 ✓
- [x] G-3: E0008 非ジェネリックテスト確認済み ✓
- [x] G-4: `cargo test multi_param_closure` — 2 件通過 ✓
- [x] G-5: `cargo test` — **1162 件通過**（目標 1160 達成）✓
- [x] G-6: `rvm --version` → `Favnir VM 1.0.0` ✓
- [x] G-7: `rvm_exec_fvc_file` テストで .fvc 実行を確認 ✓
- [x] G-8: MEMORY.md 更新済み ✓
- [ ] G-9: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| stdlib 全関数が `cargo test` で動作する | ✓ 1162 件 |
| 各関数の型シグネチャが `checker.fav` / `checker.rs` に登録されている | ✓ |
| 非ジェネリック fn 引数数不一致で E0008 検出 | ✓ (v8.8.0 済み) |
| `|x, y|` が Favnir pipeline (compiler.fav) で動作する | ✓ |
| `List.zip_with(|x| \|y\| x+y, ...)` が動作する | ✓ |
| `rvm --version` → `Favnir VM 1.0.0` | ✓ |
| `rvm file.fvc` が bytecode を実行できる | ✓ |
| `cargo test` 全件通過（1160 件以上） | ✓ 1162 件 |
