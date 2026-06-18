# v14.1.5 Tasks — fav/examples/ ctx 構文への一括移行

Date: 2026-06-11

---

## 実装内容

### examples/ 書き換え（46 ファイル）

古い `fn main() -> Unit !Io` + `IO.println(...)` 形式から
新しい `fn main(ctx: AppCtx) -> Unit` + `ctx.io.println(...)` 形式へ移行。

- [x] basic/collect.fav
- [x] basic/generics.fav（multi-type generic 呼び出し → 関数分割で E0005 回避）
- [x] cap_sort.fav
- [x] cap_user.fav
- [x] csv_demo/src/main.fav
- [x] db_demo/src/main.fav（`match` ブロック後のセミコロン追加）
- [x] diff_demo/new.fav（stage から `!Io`/`!Db` 削除）
- [x] duckdb_demo/src/main.fav
- [x] env_demo/src/main.fav（全 rune 呼び出し後セミコロン追加）
- [x] features/coalesce_demo.fav
- [x] features/for_demo.fav
- [x] gen2_demo/src/main.fav（旧 `Io.println` → `ctx.io.println` + セミコロン）
- [x] gen_demo/main.fav
- [x] grpc_client_demo/src/main.fav
- [x] grpc_e2e_demo/src/client.fav
- [x] grpc_e2e_demo/src/main.fav（セミコロン追加）
- [x] grpc_server_demo/src/main.fav
- [x] http_demo/src/main.fav
- [x] incremental_demo/src/main.fav
- [x] infer_demo/src/main.fav
- [x] json_demo/src/main.fav
- [x] log_demo/src/main.fav（全 log.* 呼び出し後セミコロン追加）
- [x] multi_file/src/main.fav（project モード要: `cd examples/multi_file && fav check`）
- [x] parquet_demo/src/main.fav
- [x] pipeline/chain.fav
- [x] pipeline/csv_to_json.fav
- [x] pipeline/pipe_match.fav
- [x] pipeline/pipeline.fav
- [x] pipeline/stage_seq_demo.fav
- [x] proto_roundtrip_demo/src/main.fav
- [x] rune_multifile_demo/src/main.fav（`++` → `String.concat()` に変換）
- [x] schema_demo/main.fav（`--` → `//`、`let x: T =` → `bind x <-`）
- [x] stat_demo/src/main.fav
- [x] stream_demo/src/main.fav（セミコロン追加）
- [x] types/adt_match.fav
- [x] types/algebraic.fav
- [x] types/interface_auto.fav
- [x] types/interface_basic.fav
- [x] types/invariant_basic.fav
- [x] types/std_states.fav
- [x] types/type_alias_demo.fav
- [x] validate_demo/src/main.fav
- [x] visibility_errors/src/main.fav

### WASM examples（旧 `!Io` 構文維持）

WASM コードゲンが `public fn main() -> Unit !Io` を要求するため、
wasm/ 以下のファイルは旧構文のままとする。

- [x] wasm/hello_wasm.fav（新規作成 — WASM テスト用）
- [x] wasm/math_wasm.fav（旧構文に戻す）
- [x] wasm/string_wasm.fav（旧構文に戻す）
- [x] wasm/closures_wasm.fav（旧構文に戻す）

### Legacy example（旧 `!Effect` 構文のドキュメント）

- [x] pipeline/custom_effects.fav（コメントで legacy 旨を明記）

### バグ修正（pre-existing）

- [x] `ast_lower_checker.rs`: `BinOp::NullCoalesce` → `OpNullCoalesce`（旧 `OpOr` から修正）
  - `??` 演算子が E0002 "logical operator requires Bool operands" を出していたバグを修正
- [x] `self/checker.fav`: `infer_expr` に `ECollect` ケースを追加
  - `EBlock(expr, EBind(name, ECollect(...), body))` パターンで "non-exhaustive match" が発生していたバグを修正

### テスト対応

- [x] `src/driver.rs`: WASM テスト 3 件を `basic/hello.fav` → `wasm/hello_wasm.fav` に変更
  - `example_hello_wasm_build_and_exec`
  - `wasm_exec_bytes_rejects_db_path_with_w004`
  - `wasm_exec_bytes_info_returns_metadata`

---

## 既知の制限（pre-existing）

| ファイル | 状況 |
|---|---|
| csv_demo/src/main.fav | `csv.parse<User>(text)` ジェネリック rune 呼び出し → E0007（checker 非対応） |
| json_demo/src/main.fav | `json.decode<Config>(text)` ジェネリック rune 呼び出し → E0007（checker 非対応） |
| multi_file/ | プロジェクトモード（`fav check` from プロジェクトディレクトリ）で OK |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| 全 examples/ が `ctx: AppCtx` 構文へ移行（wasm 除く） | ✅ |
| WASM examples が旧 `!Io` 構文で動作 | ✅ |
| `cargo test` 全件パス（2283 passed / 0 failed） | ✅ |
| `fav fmt --check self/checker.fav` パス | ✅ |
| NullCoalesce バグ修正 | ✅ |
| ECollect バグ修正 | ✅ |
