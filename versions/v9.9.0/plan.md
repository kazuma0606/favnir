# Favnir v9.9.0 Plan

Date: 2026-06-02
Theme: `fav profile` + `fav watch`

---

## Phase A: vm.rs — 新規 primitive 追加

### A-1: `Env.profile_record_raw` / `Env.profile_dump_raw`

`vm.rs` にスレッドローカルなプロファイルテーブルを追加する。

```rust
thread_local! {
    static PROFILE_RECORDS: RefCell<Vec<(String, i64)>> = RefCell::new(Vec::new());
}
```

- `"Env.profile_record_raw"` — `(name: String, ms: Int)` を追記
- `"Env.profile_dump_raw"` — `Vec` を JSON 文字列にシリアライズして返す
  形式: `[{"name":"FetchOrders","ms":1203}, ...]`
- 各 VM 実行開始時に `PROFILE_RECORDS.with(|r| r.borrow_mut().clear())` でリセット
  — `run_program` 関数の先頭でリセット

### A-2: `IO.file_mtime_raw` / `IO.sleep_ms_raw`

vm.rs の IO セクションに追加:

```rust
"IO.file_mtime_raw" => {
    // std::fs::metadata(path)?.modified()? → unix ms
}
"IO.sleep_ms_raw" => {
    // std::thread::sleep(Duration::from_millis(ms))
}
```

両 primitive が既に実装済みなら A-2 はスキップ。

---

## Phase B: `compiler_fav_runner.rs` — `compile_profiled_str`

`doc_source_str` と同パターンで追加:

```rust
pub fn compile_profiled_str(src: &str) -> Result<Vec<u8>, String> {
    // calls "compile_source_profiled" fn in compiler.fav artifact
}
```

---

## Phase C: `compiler.fav` — profile 計測コード挿入

### C-1: `ProfileRecord` 型 + `instrument_stage_call`

```favnir
fn instrument_stage_call(name: String, call_expr: Expr) -> Expr {
    // LetExpr で t0/t1/result を束縛し profile_record を呼ぶ Expr を返す
}
```

`Expr` の `ECall` variant を認識して変換。

### C-2: `instrument_stage_calls_in_expr(e: Expr) -> Expr`

式全体を再帰トラバースして stage call を変換。
stage call の識別: `ECall` の callee が `EVar` で、かつ名前が環境の stage 集合に含まれる。

### C-3: `compile_source_profiled(src: String) -> Result<List<Int>, String>`

`compile_source` のパイプラインに `instrument_stage_calls` ステップを追加。

```favnir
public fn compile_source_profiled(src: String) -> Result<List<Int>, String> {
    // lex → parse → (instrument stage calls) → compile → serialize
}
```

---

## Phase D: `driver.rs` — `cmd_profile`

### D-1: `cmd_profile(path, out_fmt)` を追加

```rust
pub fn cmd_profile(path: &str, out_fmt: &str) {
    let src = std::fs::read_to_string(path)?;
    let bytes = compile_profiled_str(&src)?;
    run_fvc_bytes(&bytes, &[])?;
    let json = call_env_profile_dump()?;
    render_profile_table(json, out_fmt);
}
```

### D-2: `render_profile_table(json: &str, fmt: &str)`

JSON をパース → テーブル文字列を生成して `println!`。

---

## Phase E: `main.rs` — `profile` サブコマンド dispatch

```rust
"profile" => {
    let out = find_flag_value(&args, "--out", "table");
    let path = find_positional(&args).unwrap_or_else(|| usage());
    cmd_profile(&path, &out);
}
```

---

## Phase F: `cli.fav` — `CmdProfile` + `CmdWatch`

### F-1: `CmdProfile(String, String)` + `CmdWatch(String, String)` を `CliCmd` に追加

### F-2: `parse_profile_cmd` / `run_profile`

`parse_doc_cmd` と同パターン。`--out` フラグ（デフォルト `"table"`）。

`run_profile`: `Compiler.compile_source_profiled_raw` → 実行 → `Env.profile_dump_raw` → 整形表示

### F-3: `parse_watch_cmd` / `run_watch_action` / `watch_loop` / `run_watch`

```favnir
fn watch_loop(path: String, mode: String, last_mtime: Int) -> Unit !IO {
    bind _ <- IO.sleep_ms_raw(500)
    match IO.file_mtime_raw(path) {
        Err(_)    => watch_loop(path, mode, last_mtime)
        Ok(mtime) =>
            if mtime != last_mtime {
                bind _ <- run_watch_action(path, mode)
                watch_loop(path, mode, mtime)
            } else {
                watch_loop(path, mode, last_mtime)
            }
    }
}
```

`run_watch_action(path, mode)`: mode に応じて `Compiler.check_raw` / `Compiler.compile_source_raw` を呼ぶ。

### F-4: `parse_named_cmd` に `"profile"` / `"watch"` 分岐を追加

### F-5: `main` の match に `CmdProfile` / `CmdWatch` arm を追加

### F-6: `run_help` に profile / watch 説明を追加

---

## Phase G: テスト + バージョン更新

### G-1: `v990_tests` モジュールを `driver.rs` に追加（3 件以上）

- `test_profile_outputs_table` — 単純な 2-stage パイプラインで profile が JSON を返すこと
- `test_watch_detects_change` — mtime が変わったときに再実行トリガーが発火すること
- `test_profile_no_overhead_on_normal_compile` — `--profile` なしでコンパイル結果が変わらないこと

### G-2: セルフチェック

```
cargo test checker_fav_wire_self_check
cargo test bootstrap
cargo test
```

### G-3: バージョン更新

- `fav/Cargo.toml`: `"9.8.0"` → `"9.9.0"`
- `fav/self/cli.fav`: `"9.8.0"` → `"9.9.0"`
- `memory/MEMORY.md`: v9.9.0 完了記録

### G-4: commit

---

## 実装順序の依存関係

```
A (vm.rs primitives)
  └── B (compiler_fav_runner.rs)
        └── C (compiler.fav)
              └── D (driver.rs)
                    └── E (main.rs)

A (IO primitives)
  └── F (cli.fav — watch_loop)

D + F + G → commit
```

A と F（watch 部分）は並列実装可能。

---

## リスクと注意点

### profile: stage 識別

compiler.fav の `Expr` 型に `EStageCall` は存在しない（通常の `ECall`）。
`compile_source_profiled` はステージ定義の名前集合を事前収集し、`ECall` の callee 名でフィルタリングする。

### watch: 再帰呼び出しスタック

`watch_loop` はテール再帰。Favnir VM はテール再帰最適化を行わない場合があるため、
長時間 watch の場合はスタックオーバーフローのリスクがある。
対策: `IO.sleep_ms_raw` の後に VM ループに戻る形で実装するか、
watchループを `while` 的な繰り返しで実現できないか検討する。
→ 当面は最大 watch 時間に制限なし・スタック深度テストを追加して確認する。

### profile_dump JSON: Rust JSON パース

`render_profile_table` は serde_json を使う。既に依存済みのため追加不要。
