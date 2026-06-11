# v14.0.5 Plan — セルフホスト完全 capability-context 化

Date: 2026-06-11

---

## Phase A — Rust VM: AppCtx 自動注入

**ファイル**: `fav/src/driver.rs`

`exec_artifact_main_with_source` を修正し、`main` の `param_count == 1` を検出して AppCtx を注入する:

```rust
fn exec_artifact_main_with_source(
    artifact: &FvcArtifact,
    db_path: Option<&str>,
    source_file: Option<&str>,
) -> Result<Value, String> {
    let main_idx = artifact
        .fn_idx_by_name("main")
        .ok_or_else(|| "error: artifact does not contain a `main` function".to_string())?;
    let display_source = source_file.unwrap_or("<artifact>");

    // param_count == 1 → main(ctx: AppCtx) として AppCtx を注入
    let initial_args = if artifact.functions[main_idx].param_count == 1 {
        vec![Value::Record(vec![])]
    } else {
        vec![]
    };

    VM::run_with_emits_db_path_and_source_file(
        artifact, main_idx, initial_args, db_path, source_file,
    )
    .map(|(value, _)| value)
    .map_err(|e| format_runtime_error(display_source, e))
}
```

同様に `exec_artifact_main_with_emits`（テスト用）も同じ判定を追加する。

---

## Phase B — lint.rs: `has_io_effect` 除外削除

**ファイル**: `fav/src/lint.rs`

`collect_ambient` 内の E0023 除外条件を削除する:

```rust
// 削除: Item::FnDef(fd) if code == "E0023" && has_io_effect(&fd.effects) => {}
// 理由: !IO 宣言がなくなるため、この除外は不要になる
```

`has_io_effect` 関数自体も未使用になるため削除する。

---

## Phase C — `self/compiler.fav` 移行

**ファイル**: `fav/self/compiler.fav`

### C-1: `compile_file_quiet` を ctx 受け取りに変更

```
// 前
fn compile_file_quiet(path: String) -> Result<Artifact, String> !IO {
    Result.and_then(IO.read_file_raw(path), ...)
}

// 後
fn compile_file_quiet(ctx: CommonCtx, path: String) -> Result<Artifact, String> {
    Result.and_then(ctx.io.read_file_raw(path), ...)
}
```

### C-2: `print_bytes` を ctx 受け取りに変更

```
// 前
fn print_bytes(bytes: List<Int>) -> Bool !IO {
    ...
    bind _ <- IO.println(Int.to_string(b));
    print_bytes(List.drop(bytes, 1))
}

// 後
fn print_bytes(ctx: CommonCtx, bytes: List<Int>) -> Bool {
    ...
    bind _ <- ctx.io.println(Int.to_string(b));
    print_bytes(ctx, List.drop(bytes, 1))
}
```

### C-3: `main` を AppCtx 受け取りに変更

```
// 前
public fn main() -> Bool !IO {
    bind args <- IO.argv();
    ...
    match compile_file_quiet(path) { ... print_bytes(bytes) }
}

// 後
public fn main(ctx: AppCtx) -> Bool {
    bind args <- ctx.io.argv();
    ...
    match compile_file_quiet(ctx, path) { ... print_bytes(ctx, bytes) }
}
```

---

## Phase D — `self/cli.fav` 移行

**ファイル**: `fav/self/cli.fav`

### D-1: 全 `run_*` 関数に `ctx: AppCtx` を追加

各 `run_*(...)` 関数の第1引数に `ctx: AppCtx` を追加し、`IO.*` → `ctx.io.*` に置換する。

```
// 前
fn run_version() -> Unit !IO {
    IO.println("favnir 11.2.0 (self-host CLI)")
}

// 後
fn run_version(ctx: AppCtx) -> Unit {
    ctx.io.println("favnir 11.2.0 (self-host CLI)")
}
```

### D-2: `main` を AppCtx 受け取りに変更

```
// 前
public fn main() -> Unit !IO {
    bind args <- IO.argv();
    ...
    run_version()
    run_check(path)
    ...
}

// 後
public fn main(ctx: AppCtx) -> Unit {
    bind args <- ctx.io.argv();
    ...
    run_version(ctx)
    run_check(ctx, path)
    ...
}
```

### D-3: `run_lint` / `run_fmt` / `run_check` 等の呼び出し箇所も ctx を伝播

全17関数の呼び出し箇所（`main` 内）を更新する。

---

## Phase E — E2E デモ `.fav` 移行

### E-1: `infra/e2e-demo/airgap/src/analyze.fav`

```
// 前
fn read_txn_csv(path: String) -> List<TxnRow> !IO { ... IO.read_file_raw(path) ... }
fn main() -> Result<Unit, String> !IO { ... }

// 後
fn read_txn_csv(ctx: CommonCtx, path: String) -> List<TxnRow> { ... ctx.io.read_file_raw(path) ... }
public fn main(ctx: AppCtx) -> Result<Unit, String> { ... read_txn_csv(ctx, path) ... }
```

### E-2: `infra/e2e-demo/fav2py/src/pipeline.fav`

```
// 前
fn load_csv_rows_json(path: String) -> String !IO { ... IO.read_file_raw(path) ... }
fn main() -> Result<Unit, String> !IO { ... }

// 後
fn load_csv_rows_json(ctx: CommonCtx, path: String) -> String { ... ctx.io.read_file_raw(path) ... }
public fn main(ctx: AppCtx) -> Result<Unit, String> { ... load_csv_rows_json(ctx, path) ... }
```

---

## Phase F — テスト更新

**ファイル**: `fav/src/driver.rs`

### F-1: `v140000_tests` の bootstrap 除外フィルター削除

`e0025_self_compiler_zero` と `e0023_and_e0025_both_zero_compiler` から
`compile_file_quiet` / `print_bytes` / `main` の除外フィルターを削除し、
`errors.is_empty()` のシンプルなアサーションに戻す。

### F-2: `v140005_tests` モジュール追加

```rust
mod v140005_tests {
    fn version_is_14_0_5()
    fn compiler_fav_zero_e0025_no_exceptions()   // フィルターなし
    fn cli_fav_zero_e0025()                       // cli.fav も E0025 ゼロ
    fn main_with_ctx_runs_via_vm()                // param_count=1 の main が動く
}
```

---

## Phase G — バージョンバンプ + テスト + コミット

1. `fav/Cargo.toml` → `version = "14.0.5"`
2. `cargo test v140005` 全件パス
3. `cargo test` 全件パス
4. `git commit -m "feat: v14.0.5 — セルフホスト完全 capability-context 化"`

---

## 実装順序

```
A (VM注入) ← 独立
B (lint.rs) ← 独立
C (compiler.fav) ← A 完了後に動作確認
D (cli.fav) ← A 完了後に動作確認
E (E2Eデモ) ← A 完了後
F (テスト) ← C,D,E 完了後
G (bump+commit) ← 全フェーズ完了後
```

A と B は並行実施可能。C〜E も実装自体は並行可能（動作確認は A 後）。

---

## リスク・注意点

1. **`cli.fav` の110箇所**: 機械的な置換だが量が多い。`IO.` → `ctx.io.` の一括変換後に `run_*` のシグネチャを追加する順序で行う。
2. **`has_io_effect` 除外削除後の E0023**: 移行前に lint.rs を変更すると既存の `!IO` 関数で E0023 が大量発生する。Phase C/D/E 完了後に Phase B を実施する。
3. **`exec_artifact_main_with_emits`**: テスト用関数も同様の ctx 注入が必要。`param_count == 1` 判定を追加する。
4. **既存の `main() -> Bool` テスト**: `param_count == 0` のため影響なし。
