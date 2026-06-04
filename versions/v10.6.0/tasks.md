# Favnir v10.6.0 Tasks

Date: 2026-06-04
Theme: Snowflake Rune 実装（runes/snowflake/）

---

## Phase A: Rune ファイル作成

- [x] A-1: `runes/snowflake/rune.toml` 作成
- [x] A-2: `runes/snowflake/snowflake.fav` 作成（エントリ、`use client.{ execute, query }` 再エクスポート）
- [x] A-3: `runes/snowflake/client.fav` 作成（`execute` / `query<T>` 実装）
- [x] A-4: `runes/snowflake/snowflake.test.fav` 作成（no_creds_is_err テスト 2 件）

---

## Phase B: Rust テスト追加

- [x] B-1: `driver.rs` 末尾に `v10600_tests` モジュール追加（1 件）
  - [x] B-1a: `snowflake_rune_test_file_passes`
    - SNOWFLAKE_ACCOUNT / SNOWFLAKE_PRIVATE_KEY を unset して `snowflake.test.fav` を実行
    - 全テスト PASS すること（no_creds_is_err が Err を返す）
- [x] B-2: `cargo test v10600` — 1 件通過

---

## Phase C: バージョン更新

- [x] C-1: `fav/Cargo.toml` version → `"10.6.0"`
- [x] C-2: `fav/self/cli.fav` の `run_version` → `"10.6.0"`

---

## Phase D: self-check + cargo test

- [x] D-1: `fav check --legacy-check self/compiler.fav` — エラーなし
- [x] D-2: `cargo test bootstrap` — 通過
- [x] D-3: `cargo test` — 全件通過（1272 件）

---

## Phase E: 完了処理

- [x] E-1: 本ファイル完了チェック
- [x] E-2: `memory/MEMORY.md` に v10.6.0 完了を記録
- [x] E-3: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `import rune "snowflake"` が通る | ✓ |
| `snowflake.execute` / `snowflake.query<T>` が型チェックを通る | ✓ |
| 資格情報なし環境で `snowflake.test.fav` 全件 PASS | ✓ |
| `cargo test bootstrap` 通過 | ✓ |
| `cargo test` 全件通過 | ✓ |
