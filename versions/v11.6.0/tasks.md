# Favnir v11.6.0 Tasks

Date: 2026-06-06
Theme: `!Postgres` → psycopg2 Python トランスパイル

---

## Phase A — PyEmitter フラグ追加（emit_python.rs）

- [x] A-1: `struct PyEmitter` に `needs_psycopg2: bool` / `needs_pg_helpers: bool` 追加
- [x] A-2: `PyEmitter::new()` 初期化（`false`）
- [x] A-3: `copy_flags_from_sub` に 2 フィールドのコピー追加
- [x] A-4: `emit_imports` に `psycopg2` / `psycopg2.extras` / `os` import 追加
- [x] A-5: `emit_helpers_and_classes` に `emit_pg_helpers` 呼び出し追加

---

## Phase B — Postgres NS ディスパッチ（emit_call_expr）

- [x] B-1: `("Postgres", "execute_raw")` → `_pg_execute(...)` 変換追加
- [x] B-2: `("Postgres", "query_raw")` → `_pg_query(...)` 変換追加
- [x] B-3: `("Postgres", name)` フォールバック → `_pg_<name>(...)` 変換追加

---

## Phase C — `emit_pg_helpers` 実装

- [x] C-1: `emit_pg_helpers` メソッド追加（`_pg_connect` / `_pg_execute` / `_pg_query`）

---

## Phase D — driver.rs pyproject.toml 更新

- [x] D-1: `cmd_transpile` で `psycopg2-binary>=2.9` 依存を pyproject.toml に追加

---

## Phase E — テスト（6 件）

- [x] E-1: `v11600_tests` モジュール追加
  - [x] `transpile_postgres_execute_raw` — `Postgres.execute_raw` → `_pg_execute(...)`
  - [x] `transpile_postgres_query_raw` — `Postgres.query_raw` → `_pg_query(...)`
  - [x] `transpile_postgres_imports_psycopg2` — `import psycopg2` が生成される
  - [x] `transpile_postgres_pg_connect_helper` — `def _pg_connect()` が含まれる
  - [x] `transpile_postgres_pyproject_psycopg2_dep` — `psycopg2-binary` が pyproject.toml に含まれる
  - [x] `transpile_postgres_pipeline_smoke` — `!Postgres` パイプラインの Python 出力検証
- [x] E-2: `cargo test v11600` — 6 件通過
- [x] E-3: `cargo test --lib` — 705 件通過

---

## Phase F — バージョン更新 + コミット

- [x] F-1: `fav/Cargo.toml` version → `"11.6.0"`
- [x] F-2: `cargo build` で `Cargo.lock` 更新
- [ ] F-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `Postgres.execute_raw` → `_pg_execute(...)` 変換 | |
| `Postgres.query_raw` → `_pg_query(...)` 変換 | |
| `import psycopg2` が自動生成される | |
| `_pg_connect` / `_pg_execute` / `_pg_query` ヘルパー生成 | |
| pyproject.toml に `psycopg2-binary>=2.9` が追加される | |
| `cargo test v11600` 6 件通過 | |
| `cargo test --lib` 705 件以上通過 | |
