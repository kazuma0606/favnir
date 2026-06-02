# Favnir v9.9.0 Tasks

Date: 2026-06-02
Theme: `fav profile` — パイプライン実行時間計測 + `fav watch` — ファイル監視

---

## Phase A: `vm.rs` — 新規 primitive 追加

- [x] A-1: `thread_local! PROFILE_RECORDS` を追加（`Vec<(String, i64)>`）
- [x] A-2: `"Env.profile_record_raw"` match arm を追加（name + ms を PROFILE_RECORDS に push）
- [x] A-3: `"Env.profile_dump_raw"` match arm を追加（JSON 文字列を返す）
- [x] A-4: `run_program` 先頭で PROFILE_RECORDS をクリア
- [x] A-5: `"IO.file_mtime_raw"` match arm を追加（`std::fs::metadata` → mtime ms）
- [x] A-6: `"IO.sleep_ms_raw"` match arm を追加（`std::thread::sleep`）
- [x] A-7: `cargo build` 通過確認

---

## Phase B: `compiler_fav_runner.rs` — `compile_profiled_str` 追加

- [x] B-1: `pub fn compile_profiled_str(src: &str) -> Result<Vec<u8>, String>` を追加
  — `doc_source_str` と同パターン、呼び出す compiler.fav 関数は `"compile_source_profiled"`
- [x] B-2: `cargo build` 通過確認

---

## Phase C: `compiler.fav` — profile 計測コード挿入

- [x] C-1: `collect_stage_names(items: List<Item>) -> List<String>` を追加（stage 名集合収集）
- [x] C-2: `instrument_stage_call(name: String, call_expr: Expr) -> Expr` を追加
  — t0/t1/result を束縛する `ELet` チェーンを構築
- [x] C-3: `instrument_expr(e: Expr, stage_names: List<String>) -> Expr` を追加（再帰トラバース）
- [x] C-4: `instrument_items(items: List<Item>, stage_names: List<String>) -> List<Item>` を追加
- [x] C-5: `public fn compile_source_profiled(src: String) -> Result<List<Int>, String>` を追加
  — `compile_source` のパイプラインに `instrument_items` ステップを挟む
- [x] C-6: `cargo test v990` 仮テストで動作確認

---

## Phase D: `driver.rs` — `cmd_profile` 追加

- [x] D-1: `fn render_profile_table(json: &str, fmt: &str)` を追加
  — `fmt == "json"` → そのまま println
  — `fmt == "table"` → serde_json でパースしてテーブル文字列を構築
- [x] D-2: `pub fn cmd_profile(path: &str, out_fmt: &str)` を追加
  — `compile_profiled_str` → `run_fvc_bytes` → `call_env_profile_dump` → `render_profile_table`
- [x] D-3: `v990_tests` モジュールを追加:
  - [x] D-3a: `test_profile_stage_names_collected` — 2-stage ソースで stage 名が収集されること
  - [x] D-3b: `test_profile_outputs_json` — `cmd_profile` が JSON フォーマットで出力すること
  - [x] D-3c: `test_profile_no_overhead` — 通常 `compile_source_str` の出力が profile 版と byte列で異なること（instrumentation が挿入されていること）
- [x] D-4: `cargo test v990` — 全件通過確認

---

## Phase E: `main.rs` — `profile` サブコマンド dispatch 追加

- [x] E-1: `cmd_profile` を use に追加
- [x] E-2: `"profile"` match arm を追加
  — `--out` フラグ（デフォルト `"table"`）
  — positional 引数（必須）
- [x] E-3: `cargo build` 通過確認

---

## Phase F: `cli.fav` — `CmdProfile` + `CmdWatch` 追加

- [x] F-1: `CliCmd` に `| CmdProfile(String, String)` を追加（path, out_fmt）
- [x] F-2: `CliCmd` に `| CmdWatch(String, String)` を追加（path, mode）
- [x] F-3: `parse_profile_cmd(args)` を追加（`--out` フラグ、デフォルト `"table"`）
- [x] F-4: `run_profile(path, out_fmt)` を追加
  — `Compiler.compile_source_profiled_raw` → 実行 → `Env.profile_dump_raw` → 整形出力
- [x] F-5: `parse_watch_mode(args)` を追加（`--check` / `--test` / デフォルト `"run"`）
- [x] F-6: `parse_watch_cmd(args)` を追加
- [x] F-7: `run_watch_action(path, mode)` を追加（mode に応じて check / run 呼び出し）
- [x] F-8: `watch_loop(path, mode, last_mtime)` を追加（500ms ポーリング + 再帰）
- [x] F-9: `run_watch(path, mode)` を追加（初回 mtime 取得 → `watch_loop`）
- [x] F-10: `parse_named_cmd` に `"profile"` / `"watch"` 分岐を追加
- [x] F-11: `main` の match に `CmdProfile(parts) => run_profile(parts._0, parts._1)` を追加
- [x] F-12: `main` の match に `CmdWatch(parts) => run_watch(parts._0, parts._1)` を追加
- [x] F-13: `run_help` に profile / watch の説明行を追加

---

## Phase G: self-check + Bootstrap + バージョン更新

- [x] G-1: `cargo test checker_fav_wire_self_check` — 通過
- [x] G-2: `cargo test bootstrap` — 通過
- [x] G-3: `cargo test` — 全件通過（1217 件）
- [x] G-4: `fav/Cargo.toml` version → `"9.9.0"`
- [x] G-5: `fav/self/cli.fav` バージョン → `"9.9.0"`
- [x] G-6: 本ファイル完了チェック
- [x] G-7: `memory/MEMORY.md` に v9.9.0 完了を記録
- [x] G-8: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `fav profile file.fav` で stage 別実行時間テーブルを表示 | ✓ |
| `--profile` なし（通常ビルド）にパフォーマンス影響なし | ✓ |
| `fav watch file.fav` がファイル変更を検出して自動再実行 | ✓ |
| エラー発生後も watch が停止しない | ✓ |
| `cargo test v990` — 3 件以上通過 | ✓ 3/3 |
| `cargo test checker_fav_wire_self_check` 通過 | ✓ |
| `cargo test bootstrap` 維持 | ✓ |

---

## スコープ外（v10.0.0 以降）

- `fav watch` の inotify / FSEvents 対応（ポーリング → イベント駆動）
- `fav profile --flamegraph` フレームグラフ出力
- プロファイル結果の永続化（`.fav-profile.json`）
- `fav bench`（繰り返し実行・統計）
