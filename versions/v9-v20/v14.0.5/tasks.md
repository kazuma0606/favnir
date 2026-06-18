# v14.0.5 Tasks — セルフホスト完全 capability-context 化

Date: 2026-06-11

---

## Phase A — Rust VM: AppCtx 自動注入

- [x] A-1: `fav/src/driver.rs` `exec_artifact_main_with_source` を修正
  - `artifact.functions[main_idx].param_count == 1` の場合に `vec![Value::Record(vec![])]` を initial_args として渡す
- [x] A-2: `exec_artifact_main_with_emits`（テスト用）も同様の判定を追加
- [x] A-3: `cargo test` でリグレッションなし確認（既存 main は param_count=0 のため影響なし）

---

## Phase B — lint.rs: `has_io_effect` 除外削除（Phase C/D/E 完了後）

- [x] B-1: `collect_ambient` 内の `has_io_effect` 除外行を削除
  ```rust
  // 削除: Item::FnDef(fd) if code == "E0023" && has_io_effect(&fd.effects) => {}
  ```
- [x] B-2: `has_io_effect` 関数が未使用になれば削除（他で使われていないか確認）
- [x] B-3: `cargo test` でリグレッションなし確認

---

## Phase C — `self/compiler.fav` 移行

- [x] C-1: `compile_file_quiet(path)` → `compile_file_quiet(ctx: CommonCtx, path: String)`
  - `IO.read_file_raw(path)` → `ctx.io.read_file_raw(path)`
- [x] C-2: `print_bytes(bytes)` → `print_bytes(ctx: CommonCtx, bytes: List<Int>)`
  - `IO.println(...)` → `ctx.io.println(...)`
  - 再帰呼び出し `print_bytes(List.drop(bytes, 1))` → `print_bytes(ctx, List.drop(bytes, 1))`
- [x] C-3: `main()` → `main(ctx: AppCtx)`
  - `IO.argv()` → `ctx.io.argv()`
  - `IO.println(...)` → `ctx.io.println(...)`
  - `compile_file_quiet(path)` → `compile_file_quiet(ctx, path)`
  - `print_bytes(bytes)` → `print_bytes(ctx, bytes)`
- [x] C-4: `fav check self/compiler.fav` で E0025 = 0, E0023 = 0 確認
- [x] C-5: `fav run self/compiler.fav` で動作確認（bootstrap テスト）

---

## Phase D — `self/cli.fav` 移行

- [x] D-1: 全 `run_*` 関数（17件）に `ctx: AppCtx` を第1引数として追加
  - `run_version(ctx)` / `run_help(ctx)` / `run_lint(ctx, path, warn_as_error)` etc.
- [x] D-2: 各 `run_*` 関数本体の `IO.*` → `ctx.io.*` 置換（110箇所）
  - `IO.println(...)` → `ctx.io.println(...)`
  - `IO.read_file_raw(...)` → `ctx.io.read_file_raw(...)`
  - `IO.write_stderr_raw(...)` → `ctx.io.write_stderr_raw(...)`
  - `IO.exit_raw(...)` → `ctx.io.exit_raw(...)`
  - `IO.argv()` → `ctx.io.argv()`
- [x] D-3: `main()` → `main(ctx: AppCtx)` + 全 `run_*(...)` 呼び出しに `ctx` を追加
- [x] D-4: `fav check self/cli.fav` で E0025 = 0, E0023 = 0 確認

---

## Phase E — E2E デモ `.fav` 移行

- [x] E-1: `infra/e2e-demo/airgap/src/analyze.fav`
  - `read_txn_csv(path)` → `read_txn_csv(ctx: CommonCtx, path: String)`
  - `main()` → `main(ctx: AppCtx)` + `read_txn_csv(ctx, path)`
  - `IO.*` → `ctx.io.*`（本体内）
- [x] E-2: `infra/e2e-demo/fav2py/src/pipeline.fav`
  - `load_csv_rows_json(path)` → `load_csv_rows_json(ctx: CommonCtx, path: String)`
  - `main()` → `main(ctx: AppCtx)` + `load_csv_rows_json(ctx, path)`
  - `IO.*` → `ctx.io.*`（本体内）

---

## Phase F — テスト更新

- [x] F-1: `v140000_tests::e0025_self_compiler_zero` の bootstrap 除外フィルターを削除
  - `non_bootstrap` フィルタリングを削除し `assert!(errors.is_empty(), ...)` に戻す
- [x] F-2: `v140000_tests::e0023_and_e0025_both_zero_compiler` の同様の除外フィルターを削除
- [x] F-3: `v140005_tests` モジュールを `driver.rs` に追加:
  - [x] `version_is_14_0_5`
  - [x] `compiler_fav_zero_e0025_no_exceptions` — フィルターなしで E0025 = 0
  - [x] `cli_fav_zero_e0025` — cli.fav も E0025 = 0
  - [x] `main_with_ctx_runs_via_vm` — `param_count=1` の main が VM 経由で動く
- [x] F-4: `cargo test v140005` 全件パス（4/4）

---

## Phase G — バージョンバンプ + 全テスト + コミット

- [x] G-1: `fav/Cargo.toml` → `version = "14.0.5"`
- [x] G-2: `cargo test` 全件パス確認
- [x] G-3: `git commit -m "feat: v14.0.5 — セルフホスト完全 capability-context 化"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `check_bang_notation(compiler.fav).is_empty()` が true（除外なし） | ✅ |
| `check_bang_notation(cli.fav).is_empty()` が true | ✅ |
| `check_bang_notation(checker.fav).is_empty()` が true（維持） | ✅ |
| `fn main(ctx: AppCtx) -> Bool` が `fav run` で動く | ✅ |
| `cargo test v140005` 全件パス（4/4） | ✅ |
| `cargo test` 全件パス | ✅ |
| `CARGO_PKG_VERSION == "14.0.5"` | ✅ |

---

## 実装ノート

- **実施順序の注意**: Phase B（`has_io_effect` 除外削除）は Phase C/D 完了後に行う。逆順にすると E0023 が大量発生してノイズが出る。
- **cli.fav の量**: `IO.` → `ctx.io.` は一括 sed 的置換が有効。ただし `Compiler.lint_source_raw(...)` 等は `IO.` を含まないため変更不要。
- **`param_count == 1` 判定の前提**: `main` が ctx 以外の単一引数を取るケースは現状存在しない。将来的には param 名/型の検査が望ましいが、現状は count で十分。
- **`has_io_effect` の他での使用**: 削除前に `grep has_io_effect` で他の使用箇所を確認する。
