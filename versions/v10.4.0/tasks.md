# Favnir v10.4.0 Tasks

Date: 2026-06-04
Theme: checker.fav 更新 — Snowflake 型チェック対応

---

## Phase A: checker.fav 更新

- [x] A-1: `snowflake_fn(fname: String) -> String` を `llm_fn` の直後に追加
  - `"execute_raw"` → `"Result"`、`"query_raw"` → `"Result"`
- [x] A-2: `builtin_ret_ty` に `"Snowflake"` 分岐を追加（`"Llm"` の直後、`"Debug"` の前）
  - `snowflake_fn(fname)` を呼ぶ
- [x] A-3: `ns_to_effect` に `"Snowflake" -> "Snowflake"` を追加（`"Llm"` の直後、`"Debug"` の前）

---

## Phase B: テスト追加

- [x] B-1: `driver.rs` 末尾に `v10400_tests` モジュール追加（2 件）
  - [x] B-1a: `snowflake_effect_checker_fav_missing` — checker.fav 経由で E0003 が出ること
  - [x] B-1b: `snowflake_effect_checker_fav_ok` — checker.fav 経由でエラーなし
- [x] B-2: `cargo test v10400` — 2 件通過

---

## Phase C: self-check + cargo test

- [x] C-1: `fav fmt --check self/checker.fav` — 差分なし（必要なら `fav fmt` で整形）
- [x] C-2: `fav check self/checker.fav` — エラーなし
- [x] C-3: `cargo test checker_fav_wire_self_check` — 通過
- [x] C-4: `cargo test` — 全件通過（目標 1269 件）

---

## Phase D: 完了処理

- [x] D-1: `fav/Cargo.toml` version → `"10.4.0"`
- [x] D-2: `fav/self/cli.fav` の `run_version` → `"10.4.0"`
- [x] D-3: 本ファイル完了チェック
- [x] D-4: `memory/MEMORY.md` に v10.4.0 完了を記録
- [x] D-5: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `checker.fav` に `snowflake_fn` / `builtin_ret_ty` / `ns_to_effect` 追加済み | |
| `fav check` 経由で `!Snowflake` 未宣言の fn に E0003 が出る | |
| `fav check self/checker.fav` エラーなし | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test` 全件通過 | |
