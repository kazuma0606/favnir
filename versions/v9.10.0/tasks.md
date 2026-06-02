# Favnir v9.10.0 Tasks

Date: 2026-06-02
Theme: `fav repl` — インタラクティブ REPL

---

## Phase A: `vm.rs` + `checker.fav` — `Debug.show_raw`

- [ ] A-1: `display_vmvalue(v: &VMValue) -> String` ヘルパーを `vm.rs` に追加
  - Int / Float / Str / Bool / Unit / List / Record / Variant / Closure の全ケース対応
  - variant タグは `"ok"` → `"Ok"` のように先頭大文字に正規化
- [ ] A-2: `"Debug.show_raw"` primitive を `vm.rs` に追加（`Unknown -> String`）
- [ ] A-3: `checker.fav` に `debug_fn` を追加し `builtin_ret_ty` / `ns_to_effect` に登録
- [ ] A-4: `cargo build` 通過確認

---

## Phase B: `driver.rs` — `cmd_repl` 実装

- [ ] B-1: `ReplSession` 構造体を追加（`definitions: String`、`def_names: Vec<String>`）
  - `new()` / `reset()` / `add_definition(src, name)` を実装
- [ ] B-2: `is_definition(line: &str) -> bool` を追加
  - `fn` / `public fn` / `stage` / `seq` / `type` / `effect` で始まる行を定義と判定
- [ ] B-3: `extract_def_name(line: &str) -> Option<String>` を追加
  - 定義行から名前トークンを取り出す
- [ ] B-4: `build_eval_source(session, expr) -> String` を追加
  - `<definitions>\nfn main() -> Unit = IO.println(Debug.show_raw(<expr>))\n`
- [ ] B-5: `handle_definition(line, session)` を追加
  - `check_source_str` で型チェック → OK なら `session.add_definition` して `defined: <name>` 表示
  - エラーなら表示してセッション変更なし
- [ ] B-6: `handle_expression(expr, session)` を追加
  - `build_eval_source` → `compile_src_str_to_bytes` → `run_fvc_bytes`
  - エラーは表示してセッション変更なし
- [ ] B-7: `extract_inferred_type(err_msg: &str) -> Option<String>` を追加
  - E0009 メッセージから `"inferred: <TYPE>"` を抽出
- [ ] B-8: `handle_type_cmd(expr, session)` を追加
  - `fn _type_probe_() -> ___PROBE___ = <expr>` をコンパイル → 型を抽出・表示
- [ ] B-9: `print_repl_help()` を追加
- [ ] B-10: `pub fn cmd_repl()` を追加（メインループ）
  - `stdin().read_line` でノーマル readline（外部クレートなし）
  - `:quit` / `:reset` / `:help` / `:env` / `:type` / 定義 / 式 を振り分け
  - EOF（read_line = 0 バイト）で正常終了
- [ ] B-11: `v9100_tests` モジュールを追加（3 件以上）
  - [ ] B-11a: `repl_eval_arithmetic` — 式のビルドソースが正しく生成される
  - [ ] B-11b: `repl_definition_accumulates` — 定義追加後に関数を呼ぶ式が評価できる
  - [ ] B-11c: `repl_error_recovery` — コンパイルエラー後にセッション定義が変化しない

---

## Phase C: `main.rs` — `repl` dispatch

- [ ] C-1: `cmd_repl` を use に追加
- [ ] C-2: `"repl"` match arm を追加（引数なし）
- [ ] C-3: `cargo build` 通過確認

---

## Phase D: `cli.fav` — `CmdRepl` ルーティング

- [ ] D-1: `CliCmd` に `| CmdRepl` を追加
- [ ] D-2: `parse_repl_cmd(args: List<String>) -> CliCmd` を追加
- [ ] D-3: `parse_named_cmd` に `else if cmd == "repl"` 分岐を追加
- [ ] D-4: `run_repl() -> Unit !IO` を追加（フォールバックメッセージを表示）
- [ ] D-5: `main` の match に `CmdRepl => run_repl()` を追加
- [ ] D-6: `run_help` に `repl` 行を追加
- [ ] D-7: `run_version` を `"9.10.0"` に更新

---

## Phase E: self-check + バージョン更新 + commit

- [ ] E-1: `cargo test v9100` — 3 件通過
- [ ] E-2: `cargo test checker_fav_wire_self_check` — 通過
- [ ] E-3: `cargo test bootstrap` — 通過
- [ ] E-4: `cargo test` — 全件通過（1217 件以上）
- [ ] E-5: `fav/Cargo.toml` version → `"9.10.0"`
- [ ] E-6: 本ファイル完了チェック
- [ ] E-7: `memory/MEMORY.md` に v9.10.0 完了を記録
- [ ] E-8: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `fav repl` で REPL が起動する | |
| 式を評価して結果を表示できる | |
| `fn` / `stage` 定義がセッションに累積される | |
| 累積した定義を後続の式から参照できる | |
| `:quit` で正常終了する | |
| `:reset` で定義がクリアされる | |
| `:env` でセッション定義を確認できる | |
| `:type <expr>` で型を確認できる（best-effort） | |
| コンパイルエラー後もセッションが継続する | |
| EOF（Ctrl+D）で正常終了する | |
| `cargo test v9100` — 3 件以上通過 | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |

---

## スコープ外（将来版へ延期）

- Tab 補完 / キー履歴（rustyline 等）— v9.11.0 LSP と合わせて検討
- マルチライン入力（`fn f(x:` で Enter → 継続）
- `fav repl --db <url>` DB 接続付き
- `:load file.fav` — ファイルをセッションに読み込み
- REPL ループの cli.fav セルフホスト化
