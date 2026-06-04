# Favnir v10.5.0 Tasks

Date: 2026-06-04
Theme: Snowflake × Favnir pipeline — E2E コンパイル確認

---

## 調査結果メモ

ロードマップ記載の「compiler.fav の NS リスト追加」は実装不要：
- `compiler.fav` に NS ホワイトリストなし（`CVName` で名前をそのまま emit）
- `compiler.rs` には v10.2.0 で `"Snowflake"` 追加済み
- `vm.rs` には v10.2.0 で `Snowflake.*` primitive 追加済み

本バージョンはテスト追加とバージョン更新のみ。

---

## Phase A: テスト追加

- [x] A-1: `driver.rs` 末尾に `v10500_tests` モジュール追加（2 件）
  - [x] A-1a: `snowflake_compiles_with_favnir_pipeline`
    - `compile_src_str_to_bytes` で `Snowflake.execute_raw` を含む fn がコンパイルできること
  - [x] A-1b: `snowflake_query_compiles_with_favnir_pipeline`
    - `compile_src_str_to_bytes` で `Snowflake.query_raw` を含む fn がコンパイルできること
- [x] A-2: `cargo test v10500` — 2 件通過

---

## Phase B: バージョン更新

- [x] B-1: `fav/Cargo.toml` version → `"10.5.0"`
- [x] B-2: `fav/self/cli.fav` の `run_version` → `"10.5.0"`

---

## Phase C: self-check + cargo test

- [x] C-1: `fav check --legacy-check self/compiler.fav` — エラーなし
- [x] C-2: `cargo test bootstrap` — 通過
- [x] C-3: `cargo test` — 全件通過（目標 1271 件）

---

## Phase D: 完了処理

- [x] D-1: 本ファイル完了チェック
- [x] D-2: `memory/MEMORY.md` に v10.5.0 完了を記録
- [x] D-3: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| Favnir pipeline で `Snowflake.execute_raw` / `query_raw` を含む fn がコンパイルできる | |
| `cargo test bootstrap` 通過 | |
| `cargo test` 全件通過 | |
