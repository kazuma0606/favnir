# Favnir v2.4.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.4.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.4.0` に更新

---

## Phase 1 — スタックトレース

### `src/backend/vm.rs`

- [x] `TraceFrame` 構造体を追加（`fn_name: String`, `line: u32`）
- [x] `VMError` に `stack_trace: Vec<TraceFrame>` フィールドを追加
  - [x] 既存の `fn_name: String` と `ip: usize` は後方互換のため残す
  - [x] `VMError` を手動構築している全箇所に `stack_trace: vec![]` を追加
- [x] `CallFrame` に `line: u32` フィールドを追加（初期値 `0`）
  - [x] `frames.push(CallFrame { ... })` の全箇所に `line: 0` を追加
- [x] `TrackLine` opcode ハンドラで `frame.line = line` を追加
  - [x] `COVERED_LINES` 更新は引き続き行う（削除しない）
- [x] `VM` 構造体に `source_file: String` フィールドを追加
  - [x] `VM::new` / `VM::new_with_db_path` で初期化（`String::new()` をデフォルトに）
  - [x] `vm.set_source_file(path: &str)` メソッドを追加（または引数で渡す）
- [x] `vm_error_from_frames` を全フレーム対応に修正（`build_stack_trace` として分離）
  - [x] `frames.iter().rev()` で全フレームから `TraceFrame` のリストを構築
  - [x] `VMError.stack_trace` に設定する
  - [x] `fn_name` は先頭フレーム（最新）の名前を設定する

### `src/driver.rs`

- [x] `format_runtime_error` 関数を追加し複数行スタックトレース形式に変更
  - [x] `"RuntimeError: {}"` を先頭行とする
  - [x] `e.stack_trace` の各フレームを `"  at {fn_name} ({source_file}:{line})"` 形式で出力
  - [x] `line == 0` の場合は `"  at {fn_name} ({source_file})"` にフォールバック
  - [x] `stack_trace` が空の場合は従来の `"vm error in {} @{}: {}"` にフォールバック
  - [x] `exec_artifact_main_with_source` に `source_file` 引数を追加し VM へ渡す

---

## Phase 2 — Unknown 型警告

### `src/middle/checker.rs`

- [x] `TypeWarning` / `FavWarning` 構造体を追加（`code: &'static str`, `message: String`, `span: Span`）
- [x] チェッカーに `warnings: Vec<FavWarning>` フィールドを追加
- [x] `check_stmt` の `Stmt::Bind` で、束縛変数の型が `Type::Unknown` のとき `W001` 警告を追加
  - [x] `W001`: `"type of '{name}' could not be resolved (Unknown)"`
- [x] チェッカーの公開 API で `warnings` を返すよう修正（`check_program` が `(Vec<TypeError>, Vec<FavWarning>)` を返す）

### `src/driver.rs`

- [x] `cmd_check` で警告を受け取り表示する
  - [x] `warning[W001]: ...` 形式で stderr に出力
  - [x] 警告があっても終了コードは `0`（エラーがなければ成功）

---

## Phase 3 — ignored テスト解消

### `src/backend/vm_legacy_coverage_tests.rs`

- [x] `legacy_vm_test_bind_record_destruct` の `#[ignore]` を削除
  - [x] テストが通ることを確認（v2.3.0 のコンパイラ対応で解消）
- [x] `legacy_vm_test_bind_variant_destruct` の対応
  - [x] `compile_stmt_into` に `Pattern::Variant` のアームを追加（`match` に脱糖）
  - [x] `#[ignore]` を削除し、テストが通ることを確認

---

## Phase 4 — テスト追加

### `src/backend/vm_stdlib_tests.rs`

- [x] `test_runtime_error_shows_stack_trace`: エラー時に `stack_trace[0].fn_name == "divide"`, `stack_trace[2].fn_name == "main"` を検証（fn_name 検証を内包）
- [x] `test_stack_trace_fn_names`: `test_runtime_error_shows_stack_trace` 内で fn_name 検証を実施（個別テストは統合済み）
- [x] `test_stack_trace_depth`: 4 段呼び出しで `stack_trace.len() == 4` を確認

### `src/middle/checker.rs`

- [x] `test_w001_unknown_type_bind`: 型が Unknown になる bind で W001 警告が出ることを確認

---

## Phase 5 — 最終確認・ドキュメント

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.3.0 の 579 → 584、ignored 0 件）
- [x] `#[ignore]` テストが 0 件であることを確認
- [x] `fav run` でエラー発生時にスタックトレースが表示されることを確認
- [x] `fav check` で Unknown 型変数に W001 が出ることを確認

### ドキュメント作成

- [x] `versions/v2.4.0/langspec.md` を作成
  - [x] スタックトレースの出力形式（`at fn (file:line)` 形式）
  - [x] `TraceFrame` と `VMError` の変更点
  - [x] W001 警告コードの説明・使用例
  - [x] ignored テスト 2 件の解消（`bind Pat <- expr` の variant 対応）
  - [x] v2.3.0 との互換性（完全上位互換）

---

## 完了条件チェック

- [x] ランタイムエラー時に 3 段以上のスタックトレースが表示される
- [x] スタックトレースが `at fn (file:line)` 形式である（line=0 時は `at fn (file)` にフォールバック）
- [x] `fav check` で Unknown 型変数に `warning[W001]` が表示される
- [x] W001 は警告のみで終了コードは 0
- [x] `legacy_vm_test_bind_record_destruct` が `#[ignore]` なしで通る
- [x] `legacy_vm_test_bind_variant_destruct` が `#[ignore]` なしで通る
- [x] `#[ignore]` テストが 0 件になる
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.4.0"`
- [x] `versions/v2.4.0/langspec.md` 作成済み
