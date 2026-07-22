# Plan: v51.1.0 — `par` stage Tokio 並列実行基盤への置換 Phase 1

## 作業ステップ

### Step 1: `ir.rs` — `IRExpr::Par` variant 追加

**ファイル**: `fav/src/middle/ir.rs`

1a. `IRExpr` enum の末尾（`RecordSpread` の後）に追加:

```rust
/// par [A, B] 並列 stage 実行 (v51.1.0)
Par {
    stage_names: Vec<String>,
    input: Box<IRExpr>,
    ty: Type,
},
```

1b. `IRExpr::ty()` の match arm に追加（`RecordSpread` の後）:

```rust
| IRExpr::Par { ty, .. } => ty,
```

### Step 2: `compiler.rs` — `FlwStep::Par` 分岐を `IRExpr::Par` emit に置換

**ファイル**: `fav/src/middle/compiler.rs`（761〜789 行付近）

現在のコード（`IO.par_execute_raw` 呼び出し構築）を以下に置換:

```rust
FlwStep::Par(names) => {
    IRExpr::Par {
        stage_names: names.clone(),
        input: Box::new(input),
        ty: Type::Unknown,
    }
}
```

### Step 3: `vm.rs` — `IRExpr::Par` ハンドラ追加

**ファイル**: `fav/src/backend/vm.rs`

`IRExpr::RecordSpread` ハンドラの後に追加:

```rust
IRExpr::Par { stage_names, input, .. } => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let input_vmval = self.eval_to_vmvalue(artifact, input)?;
        let artifact_clone = artifact.clone();
        let db_path = self.db_path.clone();
        let handles: Vec<_> = stage_names
            .iter()
            .map(|name| {
                let fn_idx = artifact_clone.fn_idx_by_name(name)
                    .ok_or_else(|| format!("par: stage '{}' not found", name));
                let a = artifact_clone.clone();
                let v = input_vmval.clone();
                let db = db_path.clone();
                let n = name.clone();
                std::thread::spawn(move || -> Result<VMValue, String> {
                    let idx = fn_idx?;
                    VM::run_with_vmvalues(&a, idx, vec![v], db, None)
                        .map(|(val, _)| val)
                        .map_err(|e| format!("par: stage '{}': {}", n, e.message))
                })
            })
            .collect();
        let mut results = Vec::new();
        for handle in handles {
            match handle.join() {
                Ok(Ok(v)) => results.push(v.into_value()),
                Ok(Err(e)) => return Err(self.error(artifact, &e)),
                Err(_) => return Err(self.error(artifact, "IRExpr::Par: a stage panicked")),
            }
        }
        Ok(Value::List(results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        return Err(self.error(artifact, "IRExpr::Par is not supported on wasm32"));
    }
}
```

> **注記**: `IO.par_execute_raw` と同パターン（`std::thread::spawn` + `VM::run_with_vmvalues`）を採用。
> `eval_to_vmvalue` / `VMValue::into_value` は vm.rs 既存の内部 API — 実装時に正確なメソッド名を確認すること。
> `tokio::spawn` への移行は v51.2 以降で検討。

### Step 4: `backend/codegen.rs` — `IRExpr::Par` arm 追加

**ファイル**: `fav/src/backend/codegen.rs`

`IRExpr::RecordSpread` arm の後に追加（ネイティブコード生成は `IO.par_execute_raw` 相当、
または `todo!()` でスタブ — ただし `#[allow(unreachable_patterns)]` は使わず明示的に追加）:

```rust
IRExpr::Par { stage_names, input, ty } => {
    // Par は VM で直接処理されるため、codegen では IO.par_execute_raw 相当の呼び出しを生成
    // ネイティブ emit パスはまだ使用されないため、Err を返すだけで十分
    return Err(CodegenError::Unsupported("IRExpr::Par is handled by VM, not codegen".to_string()));
}
```

### Step 5: `backend/wasm_codegen.rs` — `IRExpr::Par` arm 追加

**ファイル**: `fav/src/backend/wasm_codegen.rs`

IRExpr を match する以下の **7 関数**に `IRExpr::Par` arm を追加する:

| 関数名 | 追加内容 |
|---|---|
| `walk_closures_in_expr`（248行） | `input` を再帰 walk |
| `scan_closure_bound_slots_walk`（329行） | `input` を再帰 scan |
| `resolved_expr_type`（446行） | `Type::Unknown` を返す |
| `collect_local_types`（595行） | `input` を再帰 |
| `collect_expr_string_literals`（705行） | `input` を再帰 |
| `walk_expr`（801行） | `input` を再帰 walk |
| `compile_expr`（1027行） | `UnsupportedExpr` エラーを返す |

```rust
// walk 系関数への追加パターン（input を再帰）:
IRExpr::Par { input, .. } => { 関数名(input, ...); }

// compile_expr への追加:
IRExpr::Par { .. } => Err(WasmCodegenError::UnsupportedExpr(
    "par is not supported in WASM codegen".to_string()
)),
```

### Step 6: `backend/wasm_dce.rs` — `IRExpr::Par` arm 追加

**ファイル**: `fav/src/backend/wasm_dce.rs`

`IRExpr::RecordSpread` arm の後に追加（`input` を再帰スキャン）:

```rust
IRExpr::Par { input, .. } => {
    self.collect_refs_expr(input);
}
```

### Step 7: `driver.rs` — `v51100_tests` 追加・`cargo_toml_version_is_51_0_0` 削除

**対象**: `fav/src/driver.rs`（`v51000_tests` の直前）

3 件追加:

```rust
// -- v51100_tests (v51.1.0) -- par stage Tokio 並列実行 Phase 1 --
#[cfg(test)]
mod v51100_tests {
    use super::build_artifact;
    use crate::backend::vm::VM;
    use crate::frontend::parser::Parser;

    fn run_par(source: &str) -> Result<crate::value::Value, String> {
        let program = Parser::parse_str(source, "test_par.fav").expect("parse");
        let artifact = build_artifact(&program);
        let main_idx = artifact.fn_idx_by_name("main").expect("main");
        // VM::run(artifact, fn_idx, args: Vec<Value>) -> Result<Value, VMError>
        VM::run(&artifact, main_idx, vec![])
            .map_err(|e| e.message)
    }

    #[test]
    fn cargo_toml_version_is_51_1_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"51.1.0\""),
            "Cargo.toml version should be 51.1.0");
    }

    #[test]
    fn par_stage_runs_parallel() {
        let source = r#"
stage AddOne: Int -> Int = |n| { Result.ok(n + 1) }
stage AddTwo: Int -> Int = |n| { Result.ok(n + 2) }
seq P = par [AddOne, AddTwo]
public fn main() -> Int { 0 |> P }
"#;
        // par returns List<Int>; pipeline result may be the list itself
        // assert the pipeline runs without error
        let result = run_par(source);
        assert!(result.is_ok(), "par stage should run successfully, got: {:?}", result.err());
    }

    #[test]
    fn par_stage_error_propagation() {
        let source = r#"
stage Ok42: Int -> Int = |n| { Result.ok(42) }
stage Fail: Int -> Int = |n| { Result.err("boom") }
seq P = par [Ok42, Fail]
public fn main() -> Int { 0 |> P }
"#;
        let result = run_par(source);
        assert!(result.is_err(), "par with failing stage should propagate error");
    }
}
```

`v51000_tests::cargo_toml_version_is_51_0_0` を削除（他 5 件は保持）。

### Step 8: `Cargo.toml` バージョン更新

`fav/Cargo.toml`: `version = "51.0.0"` → `version = "51.1.0"`

### Step 9: テスト・Lint 確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -5
```

期待: 3115 tests passed, 0 failed

---

## ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/ir.rs` | `IRExpr::Par` variant 追加 + `ty()` arm |
| `fav/src/middle/compiler.rs` | `FlwStep::Par` → `IRExpr::Par` emit に変更 |
| `fav/src/backend/vm.rs` | `IRExpr::Par` ハンドラ追加 |
| `fav/src/backend/codegen.rs` | `IRExpr::Par` arm 追加（Err 返却） |
| `fav/src/backend/wasm_codegen.rs` | `IRExpr::Par` arm 追加（UnsupportedExpr） |
| `fav/src/backend/wasm_dce.rs` | `IRExpr::Par` arm 追加（input 再帰） |
| `fav/src/driver.rs` | v51100_tests 追加 + cargo_toml_version_is_51_0_0 削除 |
| `fav/Cargo.toml` | version → `51.1.0` |
| `CHANGELOG.md` | v51.1.0 エントリ追加 |
| `versions/current.md` | v51.1.0 更新 |
| `versions/roadmap/roadmap-v51.1-v52.0.md` | v51.1.0 実績欄更新 |
| `versions/v50-v55/v51.1.0/tasks.md` | COMPLETE に更新 |

---

## リスク・注意点

- `IRExpr::Par` を追加すると `codegen.rs`・`wasm_codegen.rs`・`wasm_dce.rs` の既存 match が non-exhaustive になり **コンパイルエラー**になる。これは意図的で、追加漏れを防ぐ仕組み。全ファイルへの追加完了後に `cargo build` が通ることを確認すること。
- VM の `par` 実装で `VM::run` の正確なシグネチャを確認すること（引数: `artifact`, `fn_idx`, `args: Vec<Value>`, `event: Option<...>`, `path: Option<&str>` → 戻り値: `Ok((Value, Vec<EmitVal>, Vec<String>))`)。
- `seq P = par [A, B]` の後に stage が続く場合（例: `|> Merge`）、`par` の戻り値 `List<Value>` が次の stage の入力になる。テストでは `|> Merge` なしのシンプルな構成を使う。
- `par` が `seq` の最終ステップになる場合、SeqStageCheck が最後の stage にのみ発火する（v50.7.0 の制約と同様）。テストは結果の Ok/Err のみを確認する。
