# Tasks: v49.6.0 — WASM / Python transpiler 互換確認

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3079 passed, 0 failed を確認（ベース確認）
- [x] `emit_python.rs` に `Stmt::Return` の実装が存在することを確認（L386 付近）
- [x] `wasm_codegen.rs` に `IRStmt::Return` の match arm が存在することを確認（L1013 付近）
- [x] `emit_python::emit_python_str` が `pub fn` として公開されていることを確認
- [x] `fav/src/driver.rs` に `v495000_tests` モジュールが存在することを確認（挿入位置の前提）

## T1 — `v496000_tests` 追加

- [x] `v496000_tests` モジュールを `v495000_tests` の直前に追加（2テスト）
  - [x] `python_emit_return_stmt`:
    - [x] `crate::emit_python::emit_python_str` に `return` 文入りソースを渡す
    - [x] 出力に `"return"` が含まれることを確認
    - [x] 注記: `return expr if condition` 構文はパーサーが受け付けないため `return x` のシンプルな形に修正
  - [x] `wasm_compat_return_stmt`:
    - [x] `include_str!("backend/wasm_codegen.rs")` でソース読み込み
    - [x] `"IRStmt::Return"` が含まれることを確認

## T2 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.6.0"`
- [x] `cargo test` 3081 passed, 0 failed（3079 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v49.6.0 エントリ追加（Python `return` 実装確認・WASM match arm 確認を明記）
- [x] `versions/current.md` を v49.6.0（3081 tests）に更新、進行中バージョンを `v49.7.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.6.0 実績を 3081 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T2 全 `[x]`）

---

> **注記**: 初回テストで `python_emit_return_stmt` FAILED — `return Result.err("negative") if x < 0` がパーサーエラー（`return ... if ...` 構文は実際には `if condition { return expr }` に展開される形のため `emit_python_str` 直接には通せない）。`return x` のシンプルな形に修正して通過。
> **注記**: `emit_python.rs` / `wasm_codegen.rs` への実コード変更はなし（確認のみ）
> **注記**: `cargo clean` はこのバージョンのスコープ外（v50.0.0 で実施）
