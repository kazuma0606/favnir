# Tasks: v51.1.0 — `par` stage Tokio 並列実行基盤への置換 Phase 1

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3113 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）
- [x] `ir.rs` に `IRExpr::Par` が存在しないことを確認（追加対象）
- [x] `compiler.rs` の `FlwStep::Par` が現在 `IO.par_execute_raw` 呼び出しをビルドしていることを確認
- [x] `vm.rs` に `IO.par_execute_raw` ハンドラが存在することを確認（置換後も残存 — 後方互換）
- [x] `v51000_tests::cargo_toml_version_is_51_0_0` が存在することを確認（削除対象）

## T1 — `ir.rs` — `IRExpr::Par` 追加

- [x] `IRExpr` enum に `Par { stage_names: Vec<String>, input: Box<IRExpr>, ty: Type }` を追加
- [x] `IRExpr::ty()` match arm に `IRExpr::Par { ty, .. } => ty` を追加

## T2 — `compiler.rs` — `FlwStep::Par` 分岐を `IRExpr::Par` emit に変更

- [x] `FlwStep::Par(names)` 分岐の `IO.par_execute_raw` 構築コードを削除
- [x] `IRExpr::Par { stage_names: names.clone(), input: Box::new(input), ty: Type::Unknown }` を emit に変更

## T3 — `vm.rs` — `IRExpr::Par` ハンドラ追加

- [x] `IRExpr::Par { stage_names, input, .. }` アームを追加
- [x] `wasm32` 非サポートガード（`#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]`）を設定
- [x] 各 stage を `std::thread::spawn` + `VM::run` で並列実行
- [x] fail-fast: `Err` または panic が発生したら即座に `Err` を返す
- [x] 全成功時: `Value::List(results)` を返す

## T4 — match 網羅性の更新（3 ファイル）

- [x] `backend/codegen.rs` に `IRExpr::Par` arm 追加（`Err(CodegenError::Unsupported(...))` 相当）
- [x] `backend/wasm_codegen.rs` に `IRExpr::Par` arm 追加（実装: 5 関数に明示的 arm 追加。`scan_closure_bound_slots_walk` / `resolved_expr_type` は `_ => {}` / `_ => expr.ty().clone()` のキャッチオール arm があるため不要）
- [x] `backend/wasm_dce.rs` に `IRExpr::Par` arm 追加（`input` を再帰スキャン）
- [x] `cargo build` が通ることを確認（コンパイルエラーなし）

## T5 — `driver.rs` — `v51100_tests` 追加

- [x] `v51100_tests` モジュールを `v51000_tests` の直前に追加（3 件）:
  - [x] `cargo_toml_version_is_51_1_0`: version = "51.1.0" を assert
  - [x] `par_stage_runs_parallel`: 2-stage par が成功することを assert
  - [x] `par_stage_error_propagation`: Err stage を含む par が Err を返すことを assert
- [x] `v51000_tests::cargo_toml_version_is_51_0_0` を削除（他 5 件は保持）

## T6 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"51.1.0"`
- [x] `cargo test` 3115 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v51.1.0 エントリ追加
- [x] `versions/current.md` を v51.1.0（3115 tests）に更新
- [x] `versions/roadmap/roadmap-v51.1-v52.0.md` の v51.1.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
