# Spec: v51.1.0 — `par` stage Tokio 並列実行基盤への置換 Phase 1

## 概要

`ast.rs` の `FlwStep::Par` は存在するが、現在の VM 実装は `IO.par_execute_raw`（`std::thread::spawn`）経由のスタブ相当。
本バージョンでは `ir.rs` に専用の `IRExpr::Par` variant を追加し、
`compiler.rs` のコンパイルパスを `IO.par_execute_raw` → `IRExpr::Par` に切り替え、
VM で `std::thread::spawn` + `VM::run_with_vmvalues` を用いた並列実行を直接ハンドルする。

> **ロードマップとの差異**: roadmap-v51.1-v52.0.md は「`tokio::spawn`」と記載しているが、
> VM の `eval` は同期関数であり `async fn` を必要とする `tokio::spawn` への直接移行は困難。
> 既存の `IO.par_execute_raw` と同パターン（`std::thread::spawn` + blocking）を採用し、
> tokio タスクプールへの移行は v51.2 以降（MergeMode 実装時）に繰り越す。

---

## 背景・現状確認

| 項目 | 現状 |
|---|---|
| `FlwStep::Par(Vec<String>)` in `ast.rs` | 実装済み ✅ |
| `compiler.rs` `FlwStep::Par` 分岐 | `IO.par_execute_raw(names, input)` へ変換（IR の Call として扱う） |
| VM `IO.par_execute_raw` | `std::thread::spawn` で並列実行 ✅ |
| `IRExpr::Par` variant | **未実装**（本バージョンで追加） |
| tokio 依存 | Cargo.toml に `tokio = { version = "1", features = ["full"] }` 登録済み ✅ |

---

## 成果物仕様

### 1. `ir.rs` — `IRExpr::Par` variant 追加

```rust
/// par [A, B] 並列 stage 実行 (v51.1.0)
/// stage_names: 並列実行する stage 名のリスト
/// input: 各 stage に渡す入力式
Par {
    stage_names: Vec<String>,
    input: Box<IRExpr>,
    ty: Type,
},
```

`IRExpr::ty()` の match arm に `IRExpr::Par { ty, .. } => ty` を追加。

### 2. `compiler.rs` — `FlwStep::Par` 分岐を `IRExpr::Par` emit に変更

現在の `IO.par_execute_raw` 呼び出し構築コードを削除し、以下を emit:

```rust
FlwStep::Par(names) => {
    IRExpr::Par {
        stage_names: names.clone(),
        input: Box::new(input),
        ty: Type::Unknown,
    }
}
```

### 3. `vm.rs` — `IRExpr::Par` ハンドラ追加

```rust
IRExpr::Par { stage_names, input, .. } => {
    let input_val = self.eval(artifact, input)?;
    // tokio の multi-thread runtime で各 stage を spawn し、結果を収集
    // fail-fast: 最初の Err が届いたら全タスクをキャンセルして Err を返す
    // 成功時: List<Value> として結果を返す
}
```

**実装方針**: `IO.par_execute_raw` と同パターン — `std::thread::spawn` で各 stage を並列実行し、
`VM::run_with_vmvalues` で stage 関数を呼び出す。
`wasm32` ターゲットでは `#[cfg(not(target_arch = "wasm32"))]` ガードで `Err` を返す（`IO.par_execute_raw` と同様）。
`IO.par_execute_raw` は compiler.fav 経由のパスが引き続き利用する可能性があるため vm.rs に残存させる（後方互換）。

### 4. 影響を受ける match 網羅性更新

新 `IRExpr::Par` variant を以下のファイルに追加:

| ファイル | 追加内容 |
|---|---|
| `ir.rs` | `ty()` match arm |
| `compiler.rs` | 既存 `FlwStep::Par` 分岐を置換（他の IRExpr match は不要） |
| `vm.rs` | 新ハンドラ |
| `backend/codegen.rs` | `IRExpr::Par` arm（`IO.par_execute_raw` 相当の codegen） |
| `backend/wasm_codegen.rs` | `IRExpr::Par` arm（`UnsupportedExpr` エラー）— 7 関数: `walk_closures_in_expr` / `scan_closure_bound_slots_walk` / `resolved_expr_type` / `collect_local_types` / `collect_expr_string_literals` / `walk_expr` / `compile_expr` |
| `backend/wasm_dce.rs` | `IRExpr::Par` arm（`input` の再帰スキャン） |

---

## テスト仕様

### `par_stage_runs_parallel`

2 つの stage を `par` で並列実行し、どちらも成功する場合に結果 List が返ることを確認。

```rust
// Favnir ソース例（stage 名のみ使用、par → List<Value>）
stage AddOne: Int -> Int = |n| { Result.ok(n + 1) }
stage AddTwo: Int -> Int = |n| { Result.ok(n + 2) }
seq P = par [AddOne, AddTwo]
public fn main() -> Int { 0 |> P }
// main が正常終了（パニックなし）することを assert
```

### `par_stage_error_propagation`

並列 stage の 1 つが `Err` を返す場合に fail-fast でエラーが伝播することを確認。

```rust
// Err を返す stage を含む par
stage Ok42: Int -> Int = |n| { Result.ok(42) }
stage Fail: Int -> Int = |n| { Result.err("boom") }
seq P = par [Ok42, Fail]
public fn main() -> Int { 0 |> P }
// VM::run が Err を返すことを assert
```

---

## バージョン要件

- `fav/Cargo.toml` version: `51.1.0`
- テスト数: 3113 → **3115**（純増 +2）
  - `v51100_tests` 3 件追加（`cargo_toml_version_is_51_1_0` + `par_stage_runs_parallel` + `par_stage_error_propagation`）
  - `v51000_tests::cargo_toml_version_is_51_0_0` 1 件削除（慣例）
  - 純増: +3 − 1 = **+2**

---

## 完了条件

- `par [A, B]` 構文が `IRExpr::Par` 経由でコンパイル・実行される
- `cargo test` 3115 tests passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v51100_tests` 3 件 pass:
  - `cargo_toml_version_is_51_1_0`
  - `par_stage_runs_parallel`
  - `par_stage_error_propagation`

---

## リスク・制約

- `IRExpr::Par` は新 variant なので `backend/codegen.rs`・`wasm_codegen.rs`・`wasm_dce.rs` の match が非網羅的になる → コンパイル時に検出されるため、追加しなければビルドが通らない。
- `wasm32` では `std::thread::spawn` が使えない → `#[cfg(not(target_arch = "wasm32"))]` ガードで `Err` を返す（`IO.par_execute_raw` の現行方針と統一）。
- `IO.par_execute_raw` は compiler.fav 経由の `par` 呼び出しが引き続き利用するため vm.rs に残す（後方互換）。
- `seq P = par [A, B]` の後に `|> Merge` なしでも VM は `List<Value>` を返す（テストは List 内の値確認を行わず、Ok/Err の判定のみ）。

---

## ロードマップ対応

roadmap-v51.1-v52.0.md v51.1.0 より:

> `ir.rs` に `Par` opcode を追加し、`compiler.rs` で `par [...]` → `Par` opcode を emit。
> VM で `tokio::spawn` を使い `Par` の各要素を並列実行・join。エラーは fail-fast。
